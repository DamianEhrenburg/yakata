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

pub fn is_safe_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("steam://")
}

pub fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{b:02X}"));
            }
        }
    }
    encoded
}

pub fn open_url(url: &str) -> Result<(), io::Error> {
    if !is_safe_url(url) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid URL scheme: only http, https, and steam are allowed",
        ));
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;

        let wide_url: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
        let wide_op: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();

        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                wide_op.as_ptr(),
                wide_url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };

        if (result as isize) > 32 {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "ShellExecuteW failed with code {result:?}"
            )))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }
}

pub fn extract_tag_from_redirect_location(location: &str) -> Option<&str> {
    let (_before, tag_part) = location.split_once("/releases/tag/")?;
    let tag = tag_part
        .split('?')
        .next()?
        .split('#')
        .next()?
        .trim_end_matches('/')
        .trim();
    if tag.is_empty() {
        None
    } else {
        Some(tag)
    }
}

pub async fn check_latest_github_release() -> Option<String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(concat!("yakata/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let response = match client
        .head("https://github.com/DamianEhrenburg/yakata/releases/latest")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to check for updates: {err}");
            return None;
        }
    };

    let location_header = response.headers().get(reqwest::header::LOCATION)?;
    let location = location_header.to_str().ok()?;
    let tag = extract_tag_from_redirect_location(location)?;

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

    #[test]
    fn test_is_safe_url() {
        assert!(is_safe_url("https://vndb.org/v17"));
        assert!(is_safe_url("http://example.com"));
        assert!(is_safe_url("steam://rungameid/12345"));
        assert!(!is_safe_url("file:///C:/Windows/notepad.exe"));
        assert!(!is_safe_url("javascript:alert(1)"));
        assert!(!is_safe_url("cmd.exe /c calc"));
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode("Hello World"), "Hello+World");
        assert_eq!(
            percent_encode("Fate/stay night & hollow"),
            "Fate%2Fstay+night+%26+hollow"
        );
        assert_eq!(percent_encode("桜"), "%E6%A1%9C");
    }

    #[test]
    fn test_extract_tag_from_redirect_location() {
        assert_eq!(
            extract_tag_from_redirect_location(
                "https://github.com/DamianEhrenburg/yakata/releases/tag/v0.3.2"
            ),
            Some("v0.3.2")
        );
        assert_eq!(
            extract_tag_from_redirect_location(
                "/DamianEhrenburg/yakata/releases/tag/v0.3.2?foo=1"
            ),
            Some("v0.3.2")
        );
        assert_eq!(
            extract_tag_from_redirect_location(
                "https://github.com/DamianEhrenburg/yakata/releases"
            ),
            None
        );
        assert_eq!(extract_tag_from_redirect_location(""), None);
    }
}
