use std::fs;
use serde_yaml::Value;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[macro_use]
extern crate wei_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_windows::init();
    
    wei_env::bin_init("wei-updater");
    let instance = wei_single::SingleInstance::new("wei-updater").unwrap();
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
        // if name != online_version.as_str() &&
        //    name != format!("{}.torrent", online_version).as_str() {
        if name.contains(&online_version) == false {
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
    info!("Start updater.");

    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };


    let download_dat_path = format!("download.dat");
    let download_dat = fs::read_to_string(&download_dat_path).expect("Something went wrong reading the file");
    let download_url = download_dat.trim();

    let product_dat_path = format!("product.dat");
    let product_dat = fs::read_to_string(&product_dat_path).expect("Something went wrong reading the file");
    let product = product_dat.trim().to_lowercase();

    let url = format!("{}{}/{}/version.dat", download_url, product, os);
    info!("{}", url);
    let local_version = fs::read_to_string("./version.dat").unwrap().trim().to_string();
    let mut online_version;

    loop {

        online_version = reqwest::get(&url).await?.text().await?;

        // online_version 的值 是 0.2.3
        // local_version 的值 是 0.2.5
        // 把这两个值 转换成数字，然后比较大小，第一个位置的0乘于10000,第二个位置的2乘于100,第三个位置的3乘于1
        // 0.2.3 => 0 * 10000 + 2 * 100 + 3 * 1 = 203
        // 0.2.5 => 0 * 10000 + 2 * 100 + 5 * 1 = 205
        // 0.2.3 < 0.2.5

        info!("online_version: {}", online_version);
        info!("local_version: {}", local_version);

        online_version = online_version.trim().to_string();

        let online_version_num = parse_version(&online_version)?;
        let local_version_num = parse_version(&local_version)?;

        info!("online_version_num: {}", online_version_num);
        info!("local_version_num: {}", local_version_num);

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

    let path = std::env::current_dir()?.join("new");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    // 列出同版本的文件，如果有，则删除
    let data = wei_run::run("wei-download", vec!["list", &(online_version.clone() + ".tar.xz")]);
    let data = match data {
        Ok(c) => c,
        Err(_) => "".to_string()
    };

    let data:serde_json::Value = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(_) => serde_json::json!({"code": 500})
    };

    let gid = match data["data"]["gid"].as_str() {
        Some(c) => c,
        None => ""
    };

    if gid != "" {
        info!("already exists gid: {}", gid);
        wei_run::run("wei-download", vec!["delete", gid])?;
        std::process::exit(0);
    }
    
    info!("add torrent to wei-download");
    let gid = match wei_run::run("wei-download", vec!["add", &torrent, path.display().to_string().as_str()]) {
        Ok(c) => {
            let data: serde_json::Value = serde_json::from_str(&c)?;
            match data["data"]["result"].as_str() {
                Some(c) => c.to_string(),
                None => {
                    info!("add torrent error: {}", c);
                    update_failed().await?;
                    "".to_string()
                }
            }
        },
        Err(e) => {
            info!("wei-download error: {}", e);
            update_failed().await?;
            serde_json::json!({"code": 500}).to_string()
        }
    };

    // 检查数据是否下载完毕, 错误次过多，直接退出
    let mut times_error = 0;
    let mut times_i = 0;
    loop {
        times_i += 1;

        // 17280 * 5 = 86400, 86400 / 3600 = 24h
        // 如果 24h 内没有下载完成，就清除没下载完的数据，然后退出
        if times_i > 17280 {
            clear_undownload_version(online_version.clone())?;
            info!("download timeout, clear undownload version and exit");
            update_failed().await?;
        }
        
        info!("loop check list");
        let cmd = match wei_run::run(
            "wei-download", 
            vec![
                "list_id",
                &gid
            ]
        ) {
            Ok(c) => c,
            Err(e) => {
                info!("wei-download error: {}", e);
                update_failed().await?;
                serde_json::json!({"code": 500}).to_string()
            }
        };

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

        if v["completed_length"].as_str().unwrap() == v["total_length"].as_str().unwrap() &&
           v["completed_length"].as_str().unwrap() != "0" &&
           v["total_length"].as_str().unwrap() != "0" {
            finished = true;
        }

        // 把path_new和v["dir"]放进Path()里面，然后比较
        let path_online_version = path.join(&online_version);
        let path_online_version = path_online_version.display().to_string();
        let path_online_version = path_online_version.replace("\\", "/");
        let path_online_version = format!("{}.tar.xz", path_online_version);

        let dir = match v["dir"].as_str() {
            Some(c) => c,
            None => {
                info!("dir is null");
                update_failed().await?;
                ""
            }
        };
        let path_download = std::path::Path::new(dir);
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

            info!("set location: {}", path.display().to_string());
            update_failed().await?;
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
            update_failed().await?;
            serde_json::json!({"data": {"check": false}}).to_string()
        }
    };

    info!("check data: {}", data);

    let data: serde_json::Value = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(e) => {
            info!("check error: {}, data: {}", e, data);
            wei_run::run("wei-download", vec!["delete", &gid])?;
            update_failed().await?;
            serde_json::json!({"data": {"check": false}})
        }
    };
    let data = match data["data"]["check"].as_bool() {
        Some(c) => c,
        None => {
            info!("data check error: {}", data);
            update_failed().await?;
            false
        }
    };
    if data == false {
        wei_run::run("wei-download", vec!["delete", &gid])?;
        info!("check error, delete download data");
        update_failed().await?;
    }

    info!("Check finished.");

    decompress(online_version.clone()).await?;

    // 下载完成后，写入 .wei/status.dat 0 重启所有daemon
    wei_env::stop();

    // 读取name.dat,里面是一个字符串
    let name = match std::fs::read_to_string("name.dat") {
        Ok(c) => c,
        Err(_) => "Wei".to_string()
    };

    // 升级期间要有一个提示框提示用户，正在升级。
    #[cfg(target_os = "windows")]
    {
        use winrt_notification::{Duration, Sound, Toast};
        Toast::new(Toast::POWERSHELL_APP_ID)
        .title(&name)
        .text1("新版本已成功下载并正在进行更新，请避免重启软件。更新完毕，软件会自动重启。")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short).show()?;
    }

    // 关闭kill.dat里面的进程
    #[cfg(target_os = "windows")]
    {
        kill()?;
        check_process("wei-updater");
    }
    
    // 等待wei-task关闭，才进一步操作
    // loop {
        
        // if wei_env::task_status() == "0" {
        //     break;
        // }
        // info!("wait wei-task close, now task_status: {}", wei_env::task_status());
        // std::thread::sleep(std::time::Duration::from_secs(10));
    // }

    // 清除旧版本，保留online_version
    clear_version(online_version.clone())?;

    info!("run copy files");
    #[cfg(target_os = "windows")]
    std::process::Command::new("powershell")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-File").arg("wei-updater.ps1")
        .arg("-arg1").arg(online_version.clone())
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW).spawn()?;
    
    #[cfg(not(target_os = "windows"))]
    {
        //获取当前目录上一层的目录
        // let current_dir = std::env::current_dir()?;
        // let current_dir = current_dir.parent().unwrap();
        // let current_dir = current_dir.display().to_string();

        // std::process::Command::new("sh")
        // .arg("-c")
        // .arg("./wei-updater.sh")
        // .arg(&online_version.clone())
        // .arg(current_dir)
        // .spawn()?;

        copy_and_run(online_version.clone())?;
    }


    info!("updater success!");
    
    Ok(())
}

pub async fn update_failed() -> Result<(), Box<dyn std::error::Error>> {
    // 记录失败次数
    let mut failed_times = wei_env::read(
        &format!("{}updater-failed.dat", wei_env::home_dir()?), 
        "times"
    )?;

    if failed_times == "" {
        failed_times = "0".to_string();
    }

    info!("failed_times: {}", failed_times);

    let mut failed_times = failed_times.parse::<u32>()? + 1;

    // 如果失败次数大于5次，读取uuid，读取日志，上报失败原因
    if failed_times > 5 {
        let uuid = wei_env::dir_uuid();
        let uuid = std::path::Path::new(&uuid);
        let uuid = match std::fs::read_to_string(&uuid) {
            Ok(c) => c,
            Err(_) => "uuid 文件不存在".to_string()
        };

        // 获取当前应用程序的路径
        let exe_path = std::env::current_exe()?;
        // 获取exe文件名
        let exe_name = match exe_path.file_name() {
            Some(c) => match c.to_str() {
                Some(c) => c,
                None => "wei-updater.exe"
            },
            None => "wei-updater.exe"
        };

        let info = format!("{}{}.log.txt", wei_env::home_dir()?, exe_name);
        let info = match std::fs::read_to_string(info) {
            Ok(c) => c,
            Err(_) => "日志文件不存在".to_string()
        };

        let url = match std::fs::read_to_string("./server.dat") {
            Ok(c) => c,
            Err(_) => "https://www.zuiyue.com".to_string()
        };
        
        let url = format!("{}/clientapi.php", url);
        let post = serde_json::json!({
            "modac": "log",
            "uuid": uuid,
            "info": info
        });

        let client = reqwest::Client::new();
        match client.post(&url).json(&post).send().await {
            Ok(_) => {},
            Err(err) => {
                info!("上报失败: {}", err);
            }
        }

        failed_times = 0;

        wei_env::write(
            &format!("{}updater-failed.dat", wei_env::home_dir()?), 
            "times",
            &failed_times.to_string()
        )?;

        info!("sleep 1 day");
        // 如果大于5次以上，就休息一天，再重置失败次数。
        std::thread::sleep(std::time::Duration::from_secs(86400));
    }

    wei_env::write(
        &format!("{}updater-failed.dat", wei_env::home_dir()?), 
        "times",
        &failed_times.to_string()
    )?;

    std::process::exit(0);
}

async fn decompress(online_version: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start decompress file");
    let xz_file = "new/".to_owned() + online_version.as_str() + ".tar.xz";
    info!("xz_file: {}", xz_file);

    let xz_file_path = std::path::Path::new(&xz_file);
    if xz_file_path.exists() { 
        wei_file::xz_decompress(&xz_file)?;
    } else {
        update_failed().await?;
    }

    let dir = "new/".to_owned() + online_version.as_str();
    let dir_path = std::path::Path::new(&dir);
    if dir_path.exists() {
        // wei_file::tar_decompress(&dir)?;
    } else {
        update_failed().await?;
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn copy_and_run(online_version: String) -> Result<(), Box<dyn std::error::Error>> {
    // 复制new / online-version 到当前目录
    info!("copy new file to main dir");

    let current_dir = std::env::current_dir()?;
    let current_dir = current_dir.parent().unwrap();
    let current_dir = current_dir.display().to_string();

    let new = current_dir.clone() + "/data/new/" + online_version.as_str();
    match copy_files(new, current_dir + "/") {
        Ok(_) => {},
        Err(e) => {
            info!("copy_files error: {}", e);
        }
    };

    wei_run::command("systemctl", vec!["restart", "wei.service"])?;

    // 打印工作目录
    // std::env::set_current_dir("../")?;
    // wei_run::run_async("wei", vec![])?;
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
        
        info!("check_status online_version: {}", online_version);
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

    #[cfg(target_os = "windows")] {
        wei_run::psrun("wei-daemon-close.ps1", "")?;
        wei_run::kill("wei.exe")?;
    }

    #[cfg(not(target_os = "windows"))]
    wei_run::kill("wei")?;

    println!("{:?}", files);

    for f in files {
        let name = f.file_name().unwrap().to_str().unwrap();
        if name.starts_with("wei-updater") {
            continue;
        }
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
    // let content = std::fs::read_to_string("./kill.dat")?;
    // let map: serde_yaml::Value = serde_yaml::from_str(&content)?;
    // if let serde_yaml::Value::Mapping(m) = map.clone() {
    //     for (k, _) in m {
    //         let name = k.as_str().unwrap();
    //         info!("kill: {}", name);
    //         wei_run::kill(name).unwrap();
    //     }
    // }
    Ok(())
}

#[cfg(target_os = "windows")]
fn check_process(exclude: &str) {
    let mut i = 0;
    loop {
        i += 1;
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-Process | Where-Object {{ ($_.Name -like '*wei*' -or $_.Name -like '*wei-*') -and $_.Name -notlike '*{}*' }}", exclude))
                .output()
                .expect("Failed to execute command")
        } else {
            std::process::Command::new("bash")
                .arg("-c")
                .arg(format!("pgrep -l 'wei' | grep -v {}", exclude))
                .output()
                .expect("Failed to execute command")
        };

        if i > 10 {
            info!("kill process timeout");
            break;
        }

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
use std::io;
use std::path::Path;
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
            let dest = to.join(path.file_name().unwrap());
            fs::remove_file(&dest).unwrap_or(());
            match fs::copy(&path, dest) {
                Ok(_) => {},
                Err(e) => {
                    info!("copy file error: {}, path: {}, to: {}", e, path.display(), to.display());
                }
            }
        } else if path.is_dir() {
            copy_files(&path, &to.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}
