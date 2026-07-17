use std::process::Command;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let bin_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let archive_name = if target_os == "windows" {
        "pi-windows-x64.zip"
    } else {
        "pi-archive.tar.gz"
    };
    let archive_path = bin_dir.join(archive_name);

    if !archive_path.exists() {
        let tag = get_latest_tag(&target_os);
        println!("cargo:warning=Detected latest Pi CLI version: {}", tag);

        let url = match target_os.as_str() {
            "windows" => format!("https://github.com/earendil-works/pi/releases/download/{}/pi-windows-x64.zip", tag),
            "linux" => format!("https://github.com/earendil-works/pi/releases/download/{}/pi-linux-x64.tar.gz", tag),
            "macos" => format!("https://github.com/earendil-works/pi/releases/download/{}/pi-darwin-x64.tar.gz", tag),
            _ => panic!("Unsupported target OS: {}", target_os),
        };

        println!("cargo:warning=Downloading Pi CLI archive from {}...", url);
        
        let download_status = Command::new("curl")
            .args(&["-L", "-o", archive_path.to_str().unwrap(), &url])
            .status()
            .expect("failed to execute curl to download pi binary");
            
        if !download_status.success() {
            panic!("failed to download Pi binary using curl");
        }
    }

    println!("cargo:rustc-env=PI_ARCHIVE_PATH={}", archive_path.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn get_latest_tag(target_os: &str) -> String {
    let fallback = "v0.80.10".to_string();

    let output_result = if target_os == "windows" {
        Command::new("powershell")
            .args(&[
                "-Command",
                "(Invoke-RestMethod -Uri 'https://api.github.com/repos/earendil-works/pi/releases/latest').tag_name",
            ])
            .output()
    } else {
        Command::new("sh")
            .args(&[
                "-c",
                "curl -s https://api.github.com/repos/earendil-works/pi/releases/latest | grep -m1 '\"tag_name\":' | cut -d'\"' -f4",
            ])
            .output()
    };

    match output_result {
        Ok(out) if out.status.success() => {
            let tag = String::from_utf8(out.stdout).unwrap().trim().to_string();
            if tag.is_empty() {
                fallback
            } else {
                tag
            }
        }
        _ => fallback,
    }
}
