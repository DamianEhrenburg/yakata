use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const STARTUP_GRACE: Duration = Duration::from_secs(10);

pub fn wait_for_game_exit(mut child: Child, game_folder: Option<PathBuf>) -> Result<Instant, io::Error> {
    child.wait()?;
    let child_exited_at = Instant::now();
    let Some(game_folder) = game_folder else {
        return Ok(child_exited_at);
    };
    let folder = normalize_path(&game_folder.to_string_lossy());
    if folder.is_empty() {
        return Ok(child_exited_at);
    }
    let mut last_seen = child_exited_at;
    let mut seen_game_process = false;
    loop {
        if game_process_running(&folder) {
            seen_game_process = true;
            last_seen = Instant::now();
        } else if seen_game_process || child_exited_at.elapsed() >= STARTUP_GRACE {
            return Ok(last_seen);
        }
        thread::sleep(POLL_INTERVAL);
    }
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
