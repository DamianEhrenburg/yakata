use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_TICK_DELTA: Duration = Duration::from_secs(30);
const STARTUP_GRACE: Duration = Duration::from_secs(10);
const STEAM_STARTUP_GRACE: Duration = Duration::from_secs(35);

pub fn wait_for_game_exit(
    mut child: Option<Child>,
    game_folder: Option<PathBuf>,
    is_steam: bool,
) -> Result<u64, io::Error> {
    let startup_grace = if is_steam {
        STEAM_STARTUP_GRACE
    } else {
        STARTUP_GRACE
    };
    let folder = game_folder
        .map(|f| normalize_path(&f.to_string_lossy()))
        .unwrap_or_default();

    let started_at = Instant::now();
    let mut last_tick = Instant::now();
    let mut accumulated_seconds: u64 = 0;
    let mut seen_game_process = false;

    loop {
        thread::sleep(POLL_INTERVAL);

        let now = Instant::now();
        let delta = now.duration_since(last_tick);
        last_tick = now;

        let child_running = if let Some(ref mut c) = child {
            match c.try_wait()? {
                Some(_) => {
                    child = None;
                    false
                }
                None => true,
            }
        } else {
            false
        };

        let folder_running = if !folder.is_empty() {
            game_process_running(&folder)
        } else {
            false
        };

        if folder_running {
            seen_game_process = true;
        }

        let is_running = child_running || folder_running;

        if is_running {
            let tick_secs = if delta > MAX_TICK_DELTA {
                POLL_INTERVAL.as_secs()
            } else {
                delta.as_secs()
            };
            accumulated_seconds = accumulated_seconds.saturating_add(tick_secs);
        } else if seen_game_process || started_at.elapsed() >= startup_grace {
            break;
        }
    }

    Ok(accumulated_seconds)
}

fn normalize_path(path: &str) -> String {
    let mut path = path.trim().replace('\\', "/");
    if cfg!(target_os = "windows") {
        path = path.to_lowercase();
    } else if let Some(unix_path) = path.strip_prefix("Z:").or_else(|| path.strip_prefix("z:")) {
        path = unix_path.to_string();
    }
    if path.is_empty() {
        return path;
    }
    if !path.ends_with('/') {
        path.push('/');
    }
    path
}

fn is_inside_folder(path: &str, folder: &str) -> bool {
    normalize_path(path).starts_with(folder)
}

#[cfg(target_os = "windows")]
fn game_process_running(folder: &str) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    let own_process_id = std::process::id();
    let mut found = false;
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut has_entry = Process32FirstW(snapshot, &mut entry) != 0;
        while has_entry && !found {
            if entry.th32ProcessID != own_process_id {
                let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                if !process.is_null() && process != INVALID_HANDLE_VALUE {
                    let mut buffer = [0u16; 1024];
                    let mut length = buffer.len() as u32;
                    if QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buffer.as_mut_ptr(), &mut length) != 0 {
                        let exe_path = String::from_utf16_lossy(&buffer[..length as usize]);
                        found = is_inside_folder(&exe_path, folder);
                    }
                    CloseHandle(process);
                }
            }
            has_entry = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
    }
    found
}

#[cfg(not(target_os = "windows"))]
fn game_process_running(folder: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    let own_process_id = std::process::id().to_string();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(process_id) = file_name.to_str() else {
            continue;
        };
        if !process_id.bytes().all(|byte| byte.is_ascii_digit()) || process_id == own_process_id {
            continue;
        }
        let process_directory = entry.path();
        if let Ok(exe_path) = std::fs::read_link(process_directory.join("exe")) {
            if is_inside_folder(&exe_path.to_string_lossy(), folder) {
                return true;
            }
        }
        if let Ok(command_line) = std::fs::read(process_directory.join("cmdline")) {
            let in_folder = command_line
                .split(|byte| *byte == 0)
                .filter(|argument| !argument.is_empty())
                .any(|argument| is_inside_folder(&String::from_utf8_lossy(argument), folder));
            if in_folder {
                return true;
            }
        }
    }
    false
}

pub fn game_folder_for(game_folder: Option<&str>, executable_path: &str) -> Option<PathBuf> {
    game_folder
        .filter(|folder| !folder.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| Path::new(executable_path).parent().map(Path::to_path_buf))
        .filter(|folder| !folder.as_os_str().is_empty())
}

#[cfg(target_os = "windows")]
fn read_steam_registry_path() -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{
        RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ,
    };

    let read_key = |hkey: HKEY, subkey: &str, value: &str| -> Option<PathBuf> {
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();
        let value_wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
        let mut buf = [0u16; 512];
        let mut buf_bytes = (buf.len() * std::mem::size_of::<u16>()) as u32;

        let status = unsafe {
            RegGetValueW(
                hkey,
                subkey_wide.as_ptr(),
                value_wide.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut buf_bytes,
            )
        };

        if status == 0 {
            let len_chars = (buf_bytes as usize / 2).saturating_sub(1);
            let val = String::from_utf16_lossy(&buf[..len_chars]).trim().to_string();
            if !val.is_empty() {
                return Some(PathBuf::from(val));
            }
        }
        None
    };

    read_key(HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath")
        .or_else(|| read_key(HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"))
        .or_else(|| read_key(HKEY_LOCAL_MACHINE, r"SOFTWARE\Valve\Steam", "InstallPath"))
}

pub fn resolve_steam_game_folder(app_id: u32) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();
        if let Some(reg_path) = read_steam_registry_path() {
            candidates.push(reg_path);
        }
        if let Some(prog_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            candidates.push(PathBuf::from(prog_files_x86).join("Steam"));
        }
        if let Some(prog_files) = std::env::var_os("ProgramFiles") {
            candidates.push(PathBuf::from(prog_files).join("Steam"));
        }
        candidates.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
        candidates.push(PathBuf::from(r"C:\Program Files\Steam"));

        let steam_dir = candidates
            .into_iter()
            .find(|p| p.join("steam.exe").exists() || p.join("steamapps").exists())?;

        let mut library_paths = Vec::new();
        library_paths.push(steam_dir.clone());

        let vdf_path = steam_dir.join("steamapps").join("libraryfolders.vdf");
        if let Ok(vdf_content) = std::fs::read_to_string(&vdf_path) {
            for line in vdf_content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"path\"") {
                    let parts: Vec<&str> = trimmed.split('"').collect();
                    if parts.len() >= 4 {
                        let clean_path = parts[3].replace(r"\\", r"\");
                        let lib_path = PathBuf::from(clean_path);
                        if lib_path.exists() && !library_paths.contains(&lib_path) {
                            library_paths.push(lib_path);
                        }
                    }
                }
            }
        }

        let manifest_name = format!("appmanifest_{app_id}.acf");
        for lib in library_paths {
            let manifest_path = lib.join("steamapps").join(&manifest_name);
            if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                for line in manifest_content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("\"installdir\"") {
                        let parts: Vec<&str> = trimmed.split('"').collect();
                        if parts.len() >= 4 {
                            let install_dir = parts[3];
                            let game_dir = lib.join("steamapps").join("common").join(install_dir);
                            if game_dir.is_dir() {
                                return Some(game_dir);
                            }
                        }
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app_id;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let norm = normalize_path("C:\\Games\\MyVN");
        assert!(norm.ends_with('/'));
        assert!(!norm.contains('\\'));
        if cfg!(target_os = "windows") {
            assert_eq!(norm, "c:/games/myvn/");
        }
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn test_is_inside_folder_prefix_collision() {
        let folder = normalize_path("C:\\Games\\VN");
        let inside = "C:\\Games\\VN\\bin\\game.exe";
        let sibling = "C:\\Games\\VN2\\game.exe";

        assert!(is_inside_folder(inside, &folder));
        assert!(!is_inside_folder(sibling, &folder));
    }

    #[test]
    fn test_game_folder_for() {
        let explicit = game_folder_for(Some("C:\\Custom"), "C:\\Default\\game.exe");
        assert_eq!(explicit, Some(PathBuf::from("C:\\Custom")));

        let whitespace = game_folder_for(Some("   "), "C:\\Default\\game.exe");
        assert_eq!(whitespace, Some(PathBuf::from("C:\\Default")));

        let none = game_folder_for(None, "C:\\Default\\game.exe");
        assert_eq!(none, Some(PathBuf::from("C:\\Default")));
    }
}
