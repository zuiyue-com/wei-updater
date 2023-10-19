static DATA_1: &'static [u8] = include_bytes!("../../wei-test/r");

use std::fs;
use serde_yaml::Value;
use std::os::windows::process::CommandExt;

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_env::bin_init("wei-updater");
    use single_instance::SingleInstance;
    let instance = SingleInstance::new("wei-updater").unwrap();
    if !instance.is_single() { 
        println!("{:?}", DATA_1);
        std::process::exit(1);
    };

    run().await?;

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let os = std::env::consts::OS;

    let download_dat_path = format!("download.dat");
    let download_dat = fs::read_to_string(&download_dat_path).expect("Something went wrong reading the file");
    let download_url = download_dat.trim();

    let product_dat_path = format!("product.dat");
    let product_dat = fs::read_to_string(&product_dat_path).expect("Something went wrong reading the file");
    let product = product_dat.trim().to_lowercase();

    let url = format!("{}{}/{}/version.dat", download_url, product, os);
    info!("{}", url);
    let local_version = fs::read_to_string("./version.dat").unwrap();
    let mut online_version;

    loop {

        online_version = reqwest::get(&url).await?.text().await?;
    
        if online_version == local_version {
            info!("No new version");
        } else {
            break;
        }

        check_status(online_version.clone())?;
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    info!("new version: {}", online_version);

    let torrent = format!("{}{}/{}/{}.torrent", download_url, product, os, online_version);
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
            "add",
            torrent.as_str(),
            path.display().to_string().as_str()
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
                "list",
                online_version.clone().as_str()
            ]
        )?;

        let mut finished = false;

        let v: Value = serde_json::from_str(&cmd)?;
        
        if v["code"] != 200 {
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
                    "set-location",
                    v["hash"].as_str().unwrap(),
                    path.display().to_string().as_str()
                ]
            )?;

            info!("set hash location: {}", path.display().to_string());
        }

        match state {
            "pausedDL" | "pausedUP" => {
                wei_run::run(
                    "wei-qbittorrent", 
                    vec![
                        "resume",
                        v["hash"].as_str().unwrap()
                    ]
                )?;
            },
            "unknown" | "missingFiles" => {
                wei_run::run(
                    "wei-qbittorrent", 
                    vec![
                        "del",
                        v["hash"].as_str().unwrap()
                    ]
                )?;
                break;
            },
            _ => {}
        }

        if finished {
            break;
        }

        check_status(online_version.clone())?;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    info!("Download finished.");
    // 下载完成后，写入 .wei/status.dat 0 重启所有daemon
    wei_env::stop();

    wei_run::run(
        "wei-qbittorrent", 
        vec![
            "recheck",
            hashes.as_str()
        ]
    )?;

    // 升级期间要有一个提示框提示用户，正在升级。
    if os == "windows" {
        use winrt_notification::{Duration, Sound, Toast};
        Toast::new(Toast::POWERSHELL_APP_ID)
        .title("Wei")
        .text1("新版本已成功下载并正在进行更新，请避免重启软件。更新完毕，软件会自动重启。")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short).show()?;
    }

    // 关闭kill.dat里面的进程
    kill()?;
    // 等待wei-task关闭，才进一步操作
    loop {
        if wei_env::task_status() == "0" {
            break;
        }
        info!("wait wei-task close, now task_status: {}", wei_env::task_status());
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    info!("run copy files");
    #[cfg(target_os = "windows")]
    std::process::Command::new("powershell")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-File").arg("wei-updater.ps1")
        .arg("-arg1").arg(online_version.clone())
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).spawn()?;
    
    #[cfg(not(target_os = "windows"))]
    copy_and_run(online_version)?;

    info!("updater success!");
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn copy_and_run(online_version: String) -> Result<(), Box<dyn std::error::Error>> {
    // 复制new / online-version 到当前目录
    info!("copy new file to main dir");
    let new = "new/".to_owned() + online_version.as_str();
    copy_files(new, "..".to_string()).expect("Failed to copy files");

    // 打印工作目录
    std::env::set_current_dir("../")?;
    wei_run::run_async("wei", vec![])?;
    Ok(())
}

fn check_status(online_version: String) -> Result<(), Box<dyn std::error::Error>> {
    if wei_env::status() == "0" {
        kill()?;
        #[cfg(target_os = "windows")]
        std::process::Command::new("powershell")
            .arg("-ExecutionPolicy").arg("Bypass")
            .arg("-File").arg("wei-daemon-close.ps1")
            .arg("-arg1").arg(online_version)
            .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).spawn()?;
        
        std::process::exit(0);
    }

    Ok(())
}

fn kill() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

// fn check_process(exclude: &str) {
//     loop {
//         let output = if cfg!(target_os = "windows") {
//             Command::new("powershell")
//                 .arg("-Command")
//                 .arg(format!("Get-Process | Where-Object {{ ($_.Name -like '*wei*' -or $_.Name -like '*wei-*') -and $_.Name -notlike '*{}*' }}", exclude))
//                 .output()
//                 .expect("Failed to execute command")
//         } else {
//             Command::new("bash")
//                 .arg("-c")
//                 .arg(format!("pgrep -f 'wei' || pgrep -f 'wei-' | grep -v {}", exclude))
//                 .output()
//                 .expect("Failed to execute command")
//         };

//         if !output.stdout.is_empty() {
//             info!("Process exists. Waiting...");
//         } else {
//             info!("Process not found. Exiting...");
//             break;
//         }
//         std::thread::sleep(std::time::Duration::from_secs(10));
//     }
// }

#[cfg(not(target_os = "windows"))]
fn copy_files<P: AsRef<Path>>(from: P, to: P) -> io::Result<()> {
    use std::io;
    use std::path::Path;
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
