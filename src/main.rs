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

    let path = std::env::current_dir()?;
    let target_path = Path::new("./src/main.rs");
    if target_path.exists() {
        let path = path.join("test/data");
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        std::env::set_current_dir(&path)?;
    } 

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
    let online_version = reqwest::get(&url).await?.text().await?;
    
    if online_version == local_version {
        println!("No new version");
        return Ok(());
    }

    let torrent = format!("http://download.zuiyue.com/{}/{}.torrent", os, online_version);

    // 使用qbittorrent下载数据
    let path = std::env::current_dir()?.join("new");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
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
    loop {
        // tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

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

        if v["progress"].as_str().unwrap() == "1" {
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

            println!("set hash location: {}", path.display().to_string());
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

    println!("Download finished");
    // 下载完成后，写入 .wei/status.dat 0 重启所有daemon
    // 不能设置状态为 2，因为wei-updater.exe 如果没有在后续操作把状态设置为 0，那么wei.exe就会无法开启
    wei_env::stop();

    // 升级期间要有一个提示框提示用户，正在升级。
    if os == "windows" {
        use winrt_notification::{Duration, Sound, Toast};
        Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Wei")
        .text1("新版本已成功下载并正在进行更新，请避免重启软件。")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short).show()?;
    }

    // 等待wei.exe关闭
    // 等待所有wei-*.exe关闭, 除了 wei-updater.exe 不关闭
    // 从当前 online-version 目录中，复制所有文件到当前目录
    check_process("wei-updater");
    // 复制new / online-version 到当前目录
    let new = "new/".to_owned() + online_version.as_str();

    copy_files(new, "..".to_string()).expect("Failed to copy files");
    if cfg!(target_os = "windows") {
        run_exe("../wei.exe");
    } else {
        run_exe("../wei");
    }
    
    // 完成所有操作，重新执行wei.exe

    // wei.exe 获取版本号，获取md5列表，获取文件列表，以上不对应，则从版本号目录中复制文件到当前目录
    // 如果复制的文件和md5不对应，则直接从服务器上面下载文件
    // 如果不存在md5列表，则直接从服务器上面下载文件
    // wei-updater.exe 应该把所有曾经release过的版本都放到wei-release对应的系统下面
    // wei-run.exe 也是做同样的操作，如果所有路径都找不到，则从服务器上面下载文件

    // wei.exe 会在运行的时候检测基础文件，如果有缺少的文件，则会自动下载最新版本的
    // 他会首先找new目录下面的最新的版本进行对比，如果有就复制过来。
    // 如果什么都没有，则从远程对应系统里面的latest下载所有最新的文件和应用程序。
    // 核心目标是只有一个程序也能把其它程序下载全。

    // 读取编写好的version.dat并自动完成更新到线上布署

    Ok(())
}

async fn build() -> Result<(), Box<dyn std::error::Error>> {
    // update trackers
    // let response = reqwest::get("https://gitea.com/XIU2/TrackersListCollection/raw/branch/master/all.txt").await?;
    // let trackers = response.text().await?;

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };

    let version = std::fs::read_to_string("./version.dat").unwrap();
    let version = version.trim();

    let src = "./version.dat";
    let dest_dir = format!("../wei-release/{}/{}/data", os.clone(), version.clone());
    let dest_file = format!("../wei-release/{}/{}/data/version.dat", os.clone(), version.clone());
    if !Path::new(&dest_dir).exists() {
        fs::create_dir_all(&dest_dir)?;
    }
    fs::copy(src, &dest_file).unwrap();
    let dest_file = format!("../wei-release/{}/version.dat", os);
    fs::copy(src, &dest_file).unwrap();

    let content = std::fs::read_to_string("../../build.dat")?;
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

    // let mut cmd = std::process::Command::new("./transmission/transmission-create");
    // cmd.arg("-o");
    // cmd.arg(format!("../wei-release/{}/{}.torrent", os.clone(), version.clone()));
    // trackers.lines().filter(|line| !line.trim().is_empty()).for_each(|tracker| {
    //     cmd.arg("-t");
    //     cmd.arg(tracker.trim());
    // });
    // cmd.arg("-s");
    // cmd.arg("8192");
    // cmd.arg(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    // cmd.arg("-c");
    // cmd.arg("wei_".to_owned() + version);
    // cmd.current_dir("../wei-release");

    // 显示结果    
    // let output = cmd.output().unwrap();
    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

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
            println!("Process exists. Waiting...");
        } else {
            println!("Process not found. Exiting...");
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
        fs::create_dir_all(&to)?;
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::copy(&path, to.join(path.file_name().unwrap()))?;
        } else if path.is_dir() {
            copy_files(&path, &to.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}

fn run_exe<P: AsRef<Path>>(exe_path: P) {
    Command::new(exe_path.as_ref())
        .spawn()
        .expect("Failed to run the exe");
}
