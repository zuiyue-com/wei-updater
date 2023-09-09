use std::fs;
use std::path::Path;
use serde_yaml::Value;
use std::process::Command;

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_env::bin_init("wei-updater");
    use single_instance::SingleInstance;
    let instance = SingleInstance::new("wei-updater").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    let args: Vec<String> = std::env::args().collect();

    let mut command = "";
    if args.len() >= 2 {
        command = &args[1];
    }

    match command {
        "build" => {
            build().await?;
        },
        _ => {
            run().await?;
        }
    }

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 当前目录加载./version.dat,对比线上版本
    let os = std::env::consts::OS;
    let url = format!("http://download.zuiyue.com/{}/version.dat", os);
    let local_version = fs::read_to_string("./version.dat").unwrap();
    // 使用reqwest获取线上版本

    loop {
        if wei_env::status() == "0" {
            return Ok(());
        }

        let online_version = reqwest::get(&url).await?.text().await?;
    
        if online_version == local_version {
            info!("No new version");
        } else {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }


    
    info!("new version: {}", online_version);

    let torrent = format!("http://download.zuiyue.com/{}/{}.torrent", os, online_version);
    info!("torrent: {}", torrent);

    // 使用qbittorrent下载数据
    let path = std::env::current_dir()?.join("new");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    info!("add torrent to qbittorrent");
    wei_run::run(
        "wei-qbittorrent", 
        vec![
            "add".to_owned(),
            torrent,
            path.display().to_string()
        ]
    )?;

    // 检查数据是否下载完毕, 错误次过多，直接退出
    let mut times_error = 0;
    let mut hashes;
    loop {
        // tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        info!("loop check list");
        let cmd = wei_run::run(
            "wei-qbittorrent", 
            vec![
                "list".to_owned(),
                online_version.clone()
            ]
        )?;

        let mut finished = false;

        let v: Value = serde_json::from_str(&cmd)?;
        
        if v["code"].as_str() != Some("200") {
            times_error += 1;
            if times_error > 5 {
                error!("error: {}", cmd);
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error")));
            }
            continue;
        }

        let v = &v["data"];
        hashes = v["hash"].as_str().unwrap().to_owned().clone();

        info!("progress: {:?}", v);

        if v["progress"].as_f64().unwrap() == 1.0 {
            finished = true;
        }

        // 出现 pausedDL | pausedUP 状态，需要重新开启下载
        let state = v["state"].as_str().unwrap();

        // 如果保存的路径不对，需要重新设置路径
        if v["save_path"].as_str().unwrap().replace("\\\\", "\\") != path.display().to_string() {
            wei_run::run(
                "wei-qbittorrent", 
                vec![
                    "set-location".to_owned(),
                    v["hash"].as_str().unwrap().to_owned(),
                    path.display().to_string()
                ]
            )?;

            info!("set hash location: {}", path.display().to_string());
        }

        match state {
            "pausedDL" | "pausedUP" => {
                wei_run::run(
                    "wei-qbittorrent", 
                    vec![
                        "resume".to_owned(),
                        v["hash"].as_str().unwrap().to_owned()
                    ]
                )?;
            },
            "unknown" | "missingFiles" => {
                wei_run::run(
                    "wei-qbittorrent", 
                    vec![
                        "del".to_owned(),
                        v["hash"].as_str().unwrap().to_owned()
                    ]
                )?;
                break;
            },
            _ => {}
        }

        if finished {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    info!("Download finished.");
    // 下载完成后，写入 .wei/status.dat 0 重启所有daemon
    wei_env::stop();

    wei_run::run(
        "wei-qbittorrent", 
        vec![
            "recheck".to_owned(),
            hashes
        ]
    )?;

    // 升级期间要有一个提示框提示用户，正在升级。
    if os == "windows" {
        use winrt_notification::{Duration, Sound, Toast};
        Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Wei")
        .text1("新版本已成功下载并正在进行更新，请避免重启软件。")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short).show()?;
    }

    // 等待所有wei-*.exe关闭, 除了 wei-updater.exe 不关闭
    // 从当前 online-version 目录中，复制所有文件到当前目录
    check_process("wei-updater");
    
    // 读取 kill.dat, 这个是一个serde_yml的列表。循环读取他，并关闭对应key的进程
    let content = std::fs::read_to_string("./kill.dat")?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;
    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, _) in m {
            let name = k.as_str().unwrap();
            info!("kill: {}", name);
            wei_run::kill(name).unwrap();
        }
    }

    // 复制new / online-version 到当前目录
    info!("copy new file to main dir");
    let new = "new/".to_owned() + online_version.as_str();
    copy_files(new, "..".to_string()).expect("Failed to copy files");
        
    // 打印工作目录
    std::env::set_current_dir("../")?;
    wei_run::run_async("wei", vec![])?;

    info!("updater success!");
    
    Ok(())
}

async fn build() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://gitea.com/XIU2/TrackersListCollection/raw/branch/master/all.txt").await?;
    let trackers = response.text().await?;

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };
    
    let contents = fs::read_to_string("../wei/Cargo.toml")
        .expect("Something went wrong reading the file");

    let value = contents.parse::<toml::Value>().unwrap();
    let package = value["package"].clone();
    let version = package["version"].to_string().replace("\"", "");
    
    // 写入 version.dat
    let mut file = File::create("./version.dat")?;
    file.write_all(version.as_bytes())?;
    println!("version:{}", version);
    let src = "./version.dat";
    let dest_dir = format!("../wei-release/{}/{}/data", os.clone(), version.clone());
    let dest_file = format!("../wei-release/{}/{}/data/version.dat", os.clone(), version.clone());
    if !Path::new(&dest_dir).exists() {
        fs::create_dir_all(&dest_dir)?;
    }
    fs::copy(src, &dest_file).unwrap();
    let dest_file = format!("../wei-release/{}/version.dat", os);
    fs::copy(src, &dest_file).unwrap();

    let content = std::fs::read_to_string("./build.dat")?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, v) in m {
            let name = k.as_str().unwrap();
            println!("build: {}", name);

            let mut cmd = std::process::Command::new("git");
            cmd.arg("pull");
            cmd.current_dir(format!("../{}", name));
            cmd.output().unwrap();

            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            cmd.arg("--release");
            cmd.current_dir(format!("../{}", name));
            cmd.output().unwrap();

            let suffix = match os {
                "windows" => ".exe",
                _ => ""
            };
            let src = format!("../{}/target/release/{}{}", name, name, suffix.clone());
            let dest_file = format!("../wei-release/{}/{}{}{}", os.clone(), version.clone(), v.as_str().unwrap(), suffix);
            println!("copy: {} -> {}", src, dest_file);
            fs::copy(src, &dest_file).unwrap();
        }
    }

    // copy wei.ico
    std::fs::copy(
        format!("../wei/res/wei.ico"),
        format!("../wei-release/{}/{}/data/wei.ico", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy daemon.dat
    std::fs::copy(
        format!("./daemon.dat"),
        format!("../wei-release/{}/{}/data/daemon.dat", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy daemon.dat
    std::fs::copy(
        format!("./kill.dat"),
        format!("../wei-release/{}/{}/data/kill.dat", os.clone(), version.clone())
    ).expect("Failed to copy files");

    // copy qbittorrent
    copy_files(
        format!("../wei-release/{}/qbittorrent", os.clone()),
        format!("../wei-release/{}/{}/data/qbittorrent", os.clone(), version.clone())
    ).expect("Failed to copy files");

    let checksum_dir = std::path::PathBuf::from(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    let mut checksum_file = File::create(format!("../wei-release/{}/{}/data/checksum.dat", os.clone(), version.clone()))?;
    write_checksums(&checksum_dir, &mut checksum_file, &checksum_dir).expect("Failed to write checksums");

    let from = format!("../wei-release/{}/{}", os.clone(), version.clone());
    let to = format!("../wei-release/{}/latest", os.clone());
    copy_files(from, to).expect("Failed to copy files");

    // make torrent
    let mut cmd = std::process::Command::new("../wei-release/windows/transmission/transmission-create");
    cmd.arg("-o");
    cmd.arg(format!("../wei-release/{}/{}.torrent", os.clone(), version.clone()));
    trackers.lines().filter(|line| !line.trim().is_empty()).for_each(|tracker| {
        cmd.arg("-t");
        cmd.arg(tracker.trim());
    });
    cmd.arg("-s");
    cmd.arg("8192");
    cmd.arg(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    cmd.arg("-c");
    cmd.arg(version.clone());
    cmd.current_dir("../wei-release");
    let output = cmd.output().unwrap();
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // git update
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add");
    cmd.arg("*");
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    let mut cmd = std::process::Command::new("git");
    cmd.arg("commit");
    cmd.arg("-am");
    cmd.arg(version);
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    let mut cmd = std::process::Command::new("git");
    cmd.arg("push");
    cmd.current_dir("../wei-release");
    cmd.output().unwrap();

    Ok(())
}


fn check_process(exclude: &str) {
    loop {
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-Process | Where-Object {{ ($_.Name -like '*wei*' -or $_.Name -like '*wei-*') -and $_.Name -notlike '*{}*' }}", exclude))
                .output()
                .expect("Failed to execute command")
        } else {
            Command::new("bash")
                .arg("-c")
                .arg(format!("pgrep -f 'wei' || pgrep -f 'wei-' | grep -v {}", exclude))
                .output()
                .expect("Failed to execute command")
        };

        if !output.stdout.is_empty() {
            info!("Process exists. Waiting...");
        } else {
            info!("Process not found. Exiting...");
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

use std::io;

fn copy_files<P: AsRef<Path>>(from: P, to: P) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    
    if !to.exists() {
        match fs::create_dir_all(&to) {
            Ok(_) => {},
            Err(e) => {
                error!("create dir error: {}", e);
            }
        }
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match fs::copy(&path, to.join(path.file_name().unwrap())) {
                Ok(_) => {},
                Err(e) => {
                    info!("copy file error: {}", e);                    
                }
            }
        } else if path.is_dir() {
            copy_files(&path, &to.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}

use std::fs::{File};
use std::io::{Write, Read};
use sha2::{Sha256, Digest};

fn calculate_sha256<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let mut file = File::open(file_path.as_ref())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

fn write_checksums<P: AsRef<Path>>(dir: P
    , checksum_file: &mut File, prefix: &Path) -> io::Result<()> {
    let dir = dir.as_ref();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(prefix).unwrap().to_path_buf();
            let sha256 = calculate_sha256(&path)?;
            writeln!(checksum_file, "{}|||{}", relative_path.display(), sha256)?;
        } else if path.is_dir() {
            write_checksums(&path, checksum_file, prefix)?;
        }
    }

    Ok(())
}