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
            build()?;          
        },
        _ => {
        }
    }

    Ok(())
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "ubuntu",
        _ => "ubuntu"
    };

    let content = std::fs::read_to_string("./build.dat")?;
    let map: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Mapping(m) = map.clone() {
        for (k, _) in m {
            let name = k.as_str().expect("process is not string");
            println!("build: {}", name);
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            cmd.arg("--release");
            cmd.current_dir(format!("../{}", name));
            let output = cmd.output().expect("failed to execute process");
            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    // let mut cmd = Command::new("cargo");
    // cmd.arg("build");
    // cmd.arg("--release");
    // cmd.arg("--target-dir");
    // cmd.arg("target");
    // cmd.arg("--target");
    // cmd.arg(os);
    // cmd.arg("--bin");
    // cmd.arg("wei-updater");

    // let output = cmd.output().expect("failed to execute process");
    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
