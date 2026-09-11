use std::collections::BTreeMap;
use std::io::Read;

/// Extract the repo tarball and collect every dir holding a SKILL file.
///
/// One tarball download replaces the old per-directory contents-API walk,
/// so discovery costs zero API quota. Scoped to source.path when given.
pub fn discover_skills(
    client: &GitHubClient,
    source: &SkillSource,
) -> Result<Vec<DiscoveredSkill>, InstallError> {
    let (tarball, _branch) = client.download_tarball(source).map_err(InstallError::Fetch)?;
    let mut tree = extract_tarball(&tarball, source).map_err(InstallError::Fetch)?;
    let scope = normalize_scope(source.path.as_deref());
    if let Some(scope) = &scope {
        tree.retain(|path, _| path == scope || path.starts_with(&format!("{scope}/")));
    }
    let mut by_dir: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    for (path, bytes) in tree {
        let (dir, file) = match path.rsplit_once('/') {
            Some((d, f)) => (d.to_string(), f.to_string()),
            None => (String::new(), path),
        };
        by_dir.entry(dir).or_default().push((file, bytes));
    }
    let mut skills = Vec::new();
    for (dir, files) in &by_dir {
        let Some(skill_md) = files.iter().find(|(name, _)| name == "SKILL.md") else {
            continue;
        };
        let (name, description) =
            parse_frontmatter(&skill_md.1).map_err(|reason| InstallError::InvalidFrontmatter {
                skill_path: join_repo_path(dir, "SKILL.md"),
                reason,
            })?;
        let prefix = if dir.is_empty() { String::new() } else { format!("{dir}/") };
        let mut skill_files: Vec<FetchedFile> = files
            .iter()
            .map(|(name, bytes)| FetchedFile {
                rel_path: name.clone(),
                bytes: bytes.clone(),
            })
            .collect();
        for (other_dir, other_files) in &by_dir {
            if other_dir != dir && other_dir.starts_with(&prefix) {
                let rel_dir = &other_dir[prefix.len()..];
                for (name, bytes) in other_files {
                    skill_files.push(FetchedFile {
                        rel_path: format!("{rel_dir}/{name}"),
                        bytes: bytes.clone(),
                    });
                }
            }
        }
        skill_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        skills.push(DiscoveredSkill {
            name,
            description,
            skill_path: join_repo_path(dir, "SKILL.md"),
            repo_dir: dir.clone(),
            files: skill_files,
        });
    }
    if skills.is_empty() {
        return Err(InstallError::NoSkillsFound { source: source.clone() });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Strip the top tarball dir to repo-relative paths, capped for safety.
fn extract_tarball(tarball: &[u8], source: &SkillSource) -> Result<Vec<(String, Vec<u8>)>, FetchError> {
    let url = tarball_url(source, "tarball");
    let decoder = flate2::read::GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(decoder);
    let mut out = Vec::new();
    let mut total_bytes: u64 = 0;
    let entries = archive.entries().map_err(|e| FetchError::BadResponse {
        url: url.clone(),
        reason: format!("unreadable tarball: {e}"),
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("unreadable tarball entry: {e}"),
        })?;
        if out.len() >= 10_000 {
            break;
        }
        let path = entry.path().map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("bad tarball path: {e}"),
        })?;
        let mut parts = path.components();
        let _top = parts.next();
        let rel: std::path::PathBuf = parts.collect();
        if rel.as_os_str().is_empty() {
            continue;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("unreadable tarball entry: {e}"),
        })?;
        total_bytes += bytes.len() as u64;
        if total_bytes > 200_000_000 {
            return Err(FetchError::BadResponse {
                url: url.clone(),
                reason: "tarball exceeds 200 MB".to_string(),
            });
        }
        out.push((rel_str, bytes));
    }
    Ok(out)
}

fn normalize_scope(path: Option<&str>) -> Option<String> {
    path.map(|p| p.trim_matches('/').to_string()).filter(|p| !p.is_empty())
}

fn join_repo_path(dir: &str, file: &str) -> String {
    if dir.is_empty() {
        file.to_string()
    } else {
        format!("{dir}/{file}")
    }
}
