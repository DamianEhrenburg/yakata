use std::io;
use std::path::Path;
use std::process::Command;
///opens a folder
pub fn open_folder(path: &Path) -> Result<(), io::Error> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("explorer");
        command.arg(path);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    command.spawn()?;
    Ok(())
}
///finds the absolute path of a command
pub fn find_host_command(command_name: String) -> Option<String> {
    let output = if std::env::var_os("FLATPAK_ID").is_some() {
        Command::new("flatpak-spawn")
            .args([
                "--host",
                "sh",
                "-c",
                "command -v -- \"$1\"",
                "sh",
                &command_name,
            ])
            .output()
    } else {
        Command::new("sh")
            .args(["-lc", "command -v -- \"$1\"", "sh", &command_name])
            .output()
    }
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}
///returns whether umu launcher is found or not
#[cfg(target_os = "linux")]
pub fn umu_launcher_is_available() -> bool {
    if std::env::var_os("FLATPAK_ID").is_some() {
        Path::new("/app/bin/umu-run").exists() || find_host_command("umu-run".to_string()).is_some()
    } else {
        find_host_command("umu-run".to_string()).is_some()
    }
}

#[cfg(not(target_os = "linux"))]
pub fn umu_launcher_is_available() -> bool {
    false
}
///return true when a path is coming from /run/flatpak/doc/
pub fn is_flatpak_document_portal_path(path: &str) -> bool {
    if path.starts_with("/run/flatpak/doc/") {
        return true;
    }
    let Some(path_after_run_user) = path.strip_prefix("/run/user/") else {
        return false;
    };
    let Some((_user_id, path_after_user_id)) = path_after_run_user.split_once('/') else {
        return false;
    };
    path_after_user_id.starts_with("doc/")
}

pub fn open_url(url: &str) -> Result<(), io::Error> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    } else {
        Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub async fn check_latest_github_release() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("yakata-app")
        .build()
        .ok()?;

    let response = client
        .get("https://api.github.com/repos/DamianEhrenburg/yakata/releases/latest")
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<GithubRelease>()
        .await
        .ok()?;

    let tag = response.tag_name.trim();
    let remote_ver_str = tag
        .strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag);
    let remote_ver = parse_semver(remote_ver_str)?;
    let local_ver = parse_semver(env!("CARGO_PKG_VERSION"))?;

    if remote_ver > local_ver {
        Some(tag.to_string())
    } else {
        None
    }
}

fn parse_semver(v: &str) -> Option<(u32, u32, u32)> {
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.split('-').next()?.parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_semver("1.2.3-beta"), Some((1, 2, 3)));
        assert!(parse_semver("0.3.0") > parse_semver("0.2.1"));
        assert_eq!(parse_semver("invalid"), None);
    }
}
