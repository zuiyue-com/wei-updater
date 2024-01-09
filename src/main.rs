#[cfg(target_os = "windows")]
static DATA_1: &'static [u8] = include_bytes!("../../wei-release/windows/qbittorrent/qbittorrent.exe");

use std::fs;
use serde_yaml::Value;
use std::os::windows::process::CommandExt;

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    if std::env::args().collect::<Vec<_>>().len() > 1000 {
        println!("{:?}", DATA_1);
    }
    
    wei_env::bin_init("wei-updater");
    use single_instance::SingleInstance;
    let instance = SingleInstance::new("wei-updater").unwrap();
    if !instance.is_single() {     
        std::process::exit(1);
    };

    run().await?;

    Ok(())
}

fn clear_version(online_version: String) -> Result<(), Box<dyn std::error::Error>> {

    // 清除旧版本，保留online_version
    let data = std::path::Path::new("./new").read_dir().unwrap();
    for entry in data {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        info!("name: {}, online_version: {}", name, online_version);
        if name != online_version.as_str() &&
           name != format!("{}.torrent", online_version).as_str() {
            info!("delete: {}", name);
            match fs::remove_dir_all(path.clone()) {
                Ok(_) => {},
                Err(_) => {}
            };
            match fs::remove_file(path) {
                Ok(_) => {},
                Err(_) => {}
            };
        }
    }

    let data = wei_run::run("wei-download", vec!["list_all"])?;
    let v: serde_json::Value = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(_) => {
            return Ok(());
        }
    };
    let data = match v["data"].as_object() {
        Some(c) => c,
        None => {
            return Ok(());
        }
    };

    let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    
    for (key, value) in data {
        let name = value["name"].as_str().unwrap();
        if re.is_match(name) && name != online_version {
            println!("delete {}", name);
            wei_run::run("wei-download", vec!["delete", key])?;
        }
    }

    Ok(())
}

fn clear_undownload_version(version: String) -> Result<(), Box<dyn std::error::Error>> {
    let data = wei_run::run("wei-download", vec!["list_all"])?;
    let data: serde_json::Value = serde_json::from_str(&data).unwrap();
    let data = data["data"].as_object().unwrap();
    
    for (key, value) in data {
        if value["completed_length"].as_str().unwrap() == "0" &&
           value["name"].as_str().unwrap() == version.as_str(){
            wei_run::run("wei-download", vec!["delete", key])?;
        }
    }

    Ok(())
}

fn parse_version(version: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let parts: Vec<u32> = version
        .split('.')
        .map(|part| part.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    
    // 确保版本号格式正确
    if parts.len() != 3 {
        return Err("Version should have three parts".into());
    }
    
    // 计算版本号的数值表示，假设格式始终为 major.minor.patch
    Ok(parts[0] * 10000 + parts[1] * 100 + parts[2])
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

        // online_version 的值 是 0.2.3
        // local_version 的值 是 0.2.5
        // 把这两个值 转换成数字，然后比较大小，第一个位置的0乘于10000,第二个位置的2乘于100,第三个位置的3乘于1
        // 0.2.3 => 0 * 10000 + 2 * 100 + 3 * 1 = 203
        // 0.2.5 => 0 * 10000 + 2 * 100 + 5 * 1 = 205
        // 0.2.3 < 0.2.5
        let online_version_num = parse_version(&online_version)?;
        let local_version_num = parse_version(&local_version)?;

        if local_version_num >= online_version_num {
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
    
    info!("add torrent to wei-download");
    wei_run::run("wei-download", vec!["add", &torrent, path.display().to_string().as_str()])?;

    // 检查数据是否下载完毕, 错误次过多，直接退出
    let mut times_error = 0;
    let mut times_i = 0;
    let mut gid;
    loop {
        times_i += 1;

        // 17280 * 5 = 86400, 86400 / 3600 = 24h
        // 如果 24h 内没有下载完成，就清除没下载完的数据，然后退出
        if times_i > 17280 {
            clear_undownload_version(online_version.clone())?;
            info!("download timeout, clear undownload version and exit");
            std::process::exit(1);
        }
        
        info!("loop check list");
        let cmd = wei_run::run(
            "wei-download", 
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
                error!("run wei-download error: {}", cmd);
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error")));
            }
            continue;
        }

        let v = &v["data"];

        info!("progress: {}/{}", v["completed_length"].as_str().unwrap(), v["total_length"].as_str().unwrap());

        if v["completed_length"].as_str().unwrap() == v["total_length"].as_str().unwrap() {
            finished = true;
        }

        gid = v["gid"].as_str().unwrap().to_string();

        // 把path_new和v["dir"]放进Path()里面，然后比较
        let path_online_version = path.join(&online_version);
        let path_online_version = path_online_version.display().to_string();
        let path_online_version = path_online_version.replace("\\", "/");

        let path_download = std::path::Path::new(v["dir"].as_str().unwrap());
        let path_download = path_download.display().to_string();
        let path_download = path_download.replace("\\", "/");

        info!("path_online_version: {}", path_online_version);
        info!("path_download: {}", path_download);

        if path_online_version != path_download {

            wei_run::run("wei-download", vec!["delete", &gid])?;

            // wei_run::run(
            //     "wei-download", 
            //     vec![
            //         "set_location",
            //         &gid,
            //         path.display().to_string().as_str()
            //     ]
            // )?;

            // info!("set location: {}", path.display().to_string());
            std::process::exit(0);
        }

        if finished {
            break;
        }

        check_status(online_version.clone())?;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }



    info!("Download finished.");

    info!("check gid: {}", gid);

    let data = match wei_run::run("wei-download", vec!["check", &gid]) {
        Ok(c) => c,
        Err(e) => {
            info!("wei-download error: {}", e);
            wei_run::run("wei-download", vec!["delete", &gid])?;
            std::process::exit(1);
        }
    };

    info!("check data: {}", data);

    let data: serde_json::Value = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(e) => {
            info!("check error: {}, data: {}", e, data);
            wei_run::run("wei-download", vec!["delete", &gid])?;
            std::process::exit(1);
        }
    };
    let data = match data["data"]["check"].as_bool() {
        Some(c) => c,
        None => {
            info!("data check error: {}", data);
            // wei_run::run("wei-download", vec!["delete", &gid])?;
            std::process::exit(1);
        }
    };
    if data == false {
        wei_run::run("wei-download", vec!["delete", &gid])?;
        info!("check error, delete download data");
        std::process::exit(1);
    }

    info!("Check finished.");

    // 下载完成后，写入 .wei/status.dat 0 重启所有daemon
    wei_env::stop();

    // 读取name.dat,里面是一个字符串
    let name = match std::fs::read_to_string("name.dat") {
        Ok(c) => c,
        Err(_) => "Wei".to_string()
    };

    // 升级期间要有一个提示框提示用户，正在升级。
    if os == "windows" {
        use winrt_notification::{Duration, Sound, Toast};
        Toast::new(Toast::POWERSHELL_APP_ID)
        .title(&name)
        .text1("新版本已成功下载并正在进行更新，请避免重启软件。更新完毕，软件会自动重启。")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short).show()?;
    }

    // 关闭kill.dat里面的进程
    kill()?;
    check_process("wei-updater");

    
    // 等待wei-task关闭，才进一步操作
    // loop {
        
        // if wei_env::task_status() == "0" {
        //     break;
        // }
        // info!("wait wei-task close, now task_status: {}", wei_env::task_status());
        // std::thread::sleep(std::time::Duration::from_secs(10));
    // }

    info!("run copy files");
    #[cfg(target_os = "windows")]
    std::process::Command::new("powershell")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-File").arg("wei-updater.ps1")
        .arg("-arg1").arg(online_version.clone())
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).spawn()?;
    
    #[cfg(not(target_os = "windows"))]
    copy_and_run(online_version)?;

    // 清除旧版本，保留online_version
    clear_version(online_version.clone())?;

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

// 列出当前目录下面所有的wei-开头的exe，然后关闭他们
fn kill_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    files.retain(|x| x.is_file());

    for f in files {
        let name = f.file_name().unwrap().to_str().unwrap();
        if name.starts_with("wei-") {
            info!("kill: {}", name);
            wei_run::kill(name).unwrap();
        }
    }

    Ok(())
}

fn kill() -> Result<(), Box<dyn std::error::Error>> {
    kill_all()?;
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

fn check_process(exclude: &str) {
    loop {
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-Process | Where-Object {{ ($_.Name -like '*wei*' -or $_.Name -like '*wei-*') -and $_.Name -notlike '*{}*' }}", exclude))
                .output()
                .expect("Failed to execute command")
        } else {
            std::process::Command::new("bash")
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
