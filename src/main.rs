use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_env::bin_init("wei-updater");
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
        }
    }

    Ok(())
}

async fn build() -> Result<(), Box<dyn std::error::Error>> {
    // update trackers
    let response = reqwest::get("https://cf.trackerslist.com/best.txt").await?;
    let trackers = response.text().await?;

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

    let mut cmd = std::process::Command::new("./transmission/transmission-create");
    cmd.arg("-o");
    cmd.arg(format!("../wei-release/{}/{}.torrent", os.clone(), version.clone()));
    trackers.lines().filter(|line| !line.trim().is_empty()).for_each(|tracker| {
        cmd.arg("-t");
        cmd.arg(tracker.trim());
    });
    cmd.arg("-s");
    cmd.arg("2048");
    cmd.arg(format!("../wei-release/{}/{}", os.clone(), version.clone()));
    cmd.arg("-c");
    cmd.arg("wei_".to_owned() + version);
    cmd.current_dir("../wei-release");
    
    // 输出执行的所有命令和参数
    println!("!!! cmd: {:?}", cmd);

    // 显示结果    
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
