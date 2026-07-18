use std::fs;
use std::process::Command;

// Embed the raw ZIP/Tarball archive of Goose
pub const GOOSE_ARCHIVE: &[u8] = include_bytes!(env!("GOOSE_ARCHIVE_PATH"));

pub fn find_goose_binary(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(p) = find_goose_binary(&path) {
                    return Some(p);
                }
            } else {
                let name = path.file_name()?.to_string_lossy();
                if name == "goose" || name == "goose.exe" {
                    return Some(path);
                }
            }
        }
    }
    None
}

pub fn ensure_goose_extracted() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // 1. Get user home directory and define ~/.hirn/bin/
    let home = dirs::home_dir().ok_or("Could not find user home directory")?;
    let hirn_dir = home.join(".hirn");
    let bin_dir = hirn_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    // 2. Define goose executable path
    let goose_name = if cfg!(windows) { "goose.exe" } else { "goose" };
    let goose_path = bin_dir.join(goose_name);

    // 3. Extract the embedded Goose archive if not present
    if !goose_path.exists() {
        let _ = fs::remove_file(bin_dir.join(".help_cache"));
        let _ = fs::remove_file(bin_dir.join(".version_cache"));

        // Create a temporary directory for extraction
        let temp_extract_dir = bin_dir.join("temp_extract");
        let _ = fs::remove_dir_all(&temp_extract_dir);
        fs::create_dir_all(&temp_extract_dir)?;

        let is_windows = cfg!(windows);
        if is_windows {
            // Extract ZIP from memory directly using `zip` crate
            let reader = std::io::Cursor::new(GOOSE_ARCHIVE);
            let mut archive = zip::ZipArchive::new(reader)?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = match file.enclosed_name() {
                    Some(path) => temp_extract_dir.join(path),
                    None => continue,
                };
                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p)?;
                        }
                    }
                    let mut outfile = fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }
        } else {
            // Non-windows uses tarball
            let archive_name = "goose_archive.tmp.tar.gz";
            let archive_path = bin_dir.join(archive_name);
            fs::write(&archive_path, GOOSE_ARCHIVE)?;
            let status = Command::new("tar")
                .args(&[
                    "-xzf",
                    archive_path.to_str().unwrap(),
                    "-C",
                    temp_extract_dir.to_str().unwrap(),
                ])
                .status()?;
            if !status.success() {
                return Err("Failed to extract Goose Tarball archive using tar".into());
            }
            let _ = fs::remove_file(archive_path)?;
        }

        // Find the goose/goose.exe binary in the extracted files
        if let Some(found_bin) = find_goose_binary(&temp_extract_dir) {
            fs::copy(&found_bin, &goose_path)?;
            // Set executable permission on Unix-like platforms
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&goose_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&goose_path, perms)?;
            }
        } else {
            let _ = fs::remove_dir_all(&temp_extract_dir);
            return Err("Goose binary not found in extracted archive".into());
        }

        // Clean up temporary extraction directory
        let _ = fs::remove_dir_all(&temp_extract_dir);
    }

    Ok(goose_path)
}
