use std::process::Command;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let bin_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let archive_name = match target_os.as_str() {
        "windows" => "goose-x86_64-pc-windows-msvc.zip",
        "linux" => "goose-x86_64-unknown-linux-gnu.tar.gz",
        "macos" => {
            if target_arch == "aarch64" {
                "goose-aarch64-apple-darwin.tar.gz"
            } else {
                "goose-x86_64-apple-darwin.tar.gz"
            }
        }
        _ => panic!("Unsupported target OS: {}", target_os),
    };
    let archive_path = bin_dir.join(archive_name);

    if !archive_path.exists() {
        let tag = get_latest_tag(&target_os);
        println!("cargo:warning=Detected latest Goose version: {}", tag);

        let url = match target_os.as_str() {
            "windows" => format!("https://github.com/aaif-goose/goose/releases/download/{}/goose-x86_64-pc-windows-msvc.zip", tag),
            "linux" => format!("https://github.com/aaif-goose/goose/releases/download/{}/goose-x86_64-unknown-linux-gnu.tar.gz", tag),
            "macos" => {
                if target_arch == "aarch64" {
                    format!("https://github.com/aaif-goose/goose/releases/download/{}/goose-aarch64-apple-darwin.tar.gz", tag)
                } else {
                    format!("https://github.com/aaif-goose/goose/releases/download/{}/goose-x86_64-apple-darwin.tar.gz", tag)
                }
            }
            _ => panic!("Unsupported target OS: {}", target_os),
        };

        println!("cargo:warning=Downloading Goose archive from {}...", url);
        
        let download_status = Command::new("curl")
            .args(&["-L", "-o", archive_path.to_str().unwrap(), &url])
            .status()
            .expect("failed to execute curl to download goose binary");
            
        if !download_status.success() {
            panic!("failed to download Goose binary using curl");
        }
    }

    println!("cargo:rustc-env=GOOSE_ARCHIVE_PATH={}", archive_path.display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn get_latest_tag(target_os: &str) -> String {
    let fallback = "v1.43.0".to_string();

    let output_result = if target_os == "windows" {
        Command::new("powershell")
            .args(&[
                "-Command",
                "(Invoke-RestMethod -Uri 'https://api.github.com/repos/aaif-goose/goose/releases/latest').tag_name",
            ])
            .output()
    } else {
        Command::new("sh")
            .args(&[
                "-c",
                "curl -s https://api.github.com/repos/aaif-goose/goose/releases/latest | grep -m1 '\"tag_name\":' | cut -d'\"' -f4",
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
