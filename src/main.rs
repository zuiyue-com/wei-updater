use std::fs;
use std::path::Path;

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

    // 检查数据是否下载完毕
    // 下载完成后，写入 .wei/status.dat 2 重启所有daemon
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
