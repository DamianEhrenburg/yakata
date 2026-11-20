use crate::models::LaunchMode;
use crate::storage::yakata_data_dir;
use crate::system::find_host_command;
use std::fs::{self, OpenOptions};

use std::io;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[cfg(target_os = "linux")]
const STEAM_WRAPPER_SCRIPT: &str = r#"#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
helper="$script_dir/yakata-steam-helper"
arguments=("$@")
last_separator=-1

for index in "${!arguments[@]}"; do
    if [[ "${arguments[$index]}" == "--" ]]; then
        last_separator=$index
    fi
done

if (( last_separator < 0 )); then
    echo "Yakata wrapper: Steam command did not contain a --
    separator." >&2
    exit 1
fi

if [[ ! -x "$helper" ]]; then
    echo "Yakata wrapper: helper is missing or not executable:
    $helper" >&2
    exit 1
fi

runtime_arguments=("${arguments[@]:0:last_separator + 1}")
game_arguments=("${arguments[@]:last_separator + 1}")

exec "${runtime_arguments[@]}" "$helper" "${game_arguments[@]}"
"#;

#[cfg(target_os = "linux")]
const STEAM_HELPER_SCRIPT: &str = r#"#!/usr/bin/env bash
set -u

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

app_id="${STEAM_COMPAT_APP_ID:-}"

if [[ -z "$app_id" ]]; then
    echo "Yakata helper: Steam App ID is unavailable." >&2
    exit 1
fi

request_file="$script_dir/tool-request-$app_id"
active_file="$script_dir/active-$app_id"
log_file="$script_dir/helper-$app_id.log"


if [[ "$#" -lt 3 ]]; then
    echo "Yakata helper: incomplete Proton command." >> "$log_file"
    exit 1
fi

proton_runner="$1"
shift

rm -f "$request_file"
touch "$active_file"
trap 'rm -f "$active_file" "$request_file"' EXIT

"$proton_runner" "$@" >> "$log_file" 2>&1 &
game_process=$!

while kill -0 "$game_process" 2>/dev/null; do
    if [[ -s "$request_file" ]]; then
        tool_path="$(head -n 1 "$request_file")"
        rm -f "$request_file"

        echo "Launching tool: $tool_path" >> "$log_file"
        "$proton_runner" run "$tool_path" >> "$log_file" 2>&1 &
    fi

    sleep 0.25
done

wait "$game_process"
"#;

pub fn detect_leproc_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            "C:\\Program Files\\Locale Emulator\\LEProc.exe",
            "C:\\Program Files (x86)\\Locale Emulator\\LEProc.exe",
        ];
        for candidate in &candidates {
            if std::path::Path::new(candidate).exists() {
                return Some(candidate.to_string());
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let user_candidate = format!("{local_app_data}\\Locale Emulator\\LEProc.exe");
            if std::path::Path::new(&user_candidate).exists() {
                return Some(user_candidate);
            }
        }
    }
    None
}

pub fn parse_steam_app_id(input: &str) -> Option<u32> {
    let trimmed = input.trim();
    if let Ok(id) = trimmed.parse::<u32>() {
        return Some(id);
    }
    if let Some(pos) = trimmed.find("/app/") {
        let after = &trimmed[pos + 5..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(id) = digits.parse::<u32>() {
            return Some(id);
        }
    }
    if let Some(pos) = trimmed.find("rungameid/") {
        let after = &trimmed[pos + 10..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(id) = digits.parse::<u32>() {
            return Some(id);
        }
    }
    None
}

pub fn split_launch_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in input.chars() {
        match quote {
            Some(q) if ch == q => {
                quote = None;
            }
            Some(_) => {
                current.push(ch);
            }
            None if ch == '"' || ch == '\'' => {
                quote = Some(ch);
            }
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            None => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn open_launch_log(log_path: Option<PathBuf>) -> io::Result<(Option<fs::File>, Option<fs::File>)> {
    let Some(path) = log_path else {
        return Ok((None, None));
    };
    let stdout_file = OpenOptions::new().create(true).append(true).open(&path)?;
    let stderr_file = stdout_file.try_clone()?;
    Ok((Some(stdout_file), Some(stderr_file)))
}

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub executable_path: String,
    pub launch_mode: LaunchMode,
    pub steam_app_id: Option<u32>,
    pub wine_binary: Option<String>,
    pub wine_prefix: Option<String>,
    pub wine_locale: Option<String>,
    pub proton_path: Option<String>,
    pub umu_game_id: String,
    pub launch_arguments: String,
    pub launch_environment: Vec<(String, String)>,
    pub launch_log_path: Option<PathBuf>,
    pub run_with_locale_emulator: bool,
    pub leproc_path: Option<String>,
}

pub fn launch_executable(options: LaunchOptions) -> Result<Option<Child>, io::Error> {
    let path = PathBuf::from(&options.executable_path);
    let working_directory = path.parent().map(Path::to_path_buf);
    let (stdout_log, stderr_log) = open_launch_log(options.launch_log_path)?;
    let parsed_args = split_launch_arguments(&options.launch_arguments);

    match options.launch_mode {
        LaunchMode::Native => {
            let mut command = if cfg!(target_os = "windows") && options.run_with_locale_emulator {
                let leproc = options
                    .leproc_path
                    .or_else(detect_leproc_path)
                    .unwrap_or_else(|| "LEProc.exe".to_string());
                let mut cmd = host_command(leproc, options.launch_environment, working_directory);
                cmd.arg("-run");
                cmd.arg(path.to_string_lossy().to_string());
                cmd.args(&parsed_args);
                cmd
            } else {
                let mut cmd = host_command(
                    path.to_string_lossy().to_string(),
                    options.launch_environment,
                    working_directory,
                );
                cmd.args(&parsed_args);
                cmd
            };
            attach_launch_log(&mut command, stdout_log, stderr_log);
            Ok(Some(command.spawn()?))
        }
        LaunchMode::Wine => {
            let wine_command = options.wine_binary.unwrap_or_else(|| "wine".to_string());
            let mut environment = options.launch_environment;
            if let Some(prefix) = options.wine_prefix {
                environment.push(("WINEPREFIX".to_string(), prefix));
            }
            if let Some(locale) = options.wine_locale {
                environment.push(("LANG".to_string(), locale.clone()));
                environment.push(("LC_ALL".to_string(), locale));
            }
            let mut command = host_command(wine_command, environment, working_directory);
            command.arg(path);
            command.args(&parsed_args);
            attach_launch_log(&mut command, stdout_log, stderr_log);
            Ok(Some(command.spawn()?))
        }
        LaunchMode::Proton => {
            let mut environment = options.launch_environment;
            if let Some(prefix) = options.wine_prefix {
                environment.push(("WINEPREFIX".to_string(), prefix));
            }
            if let Some(proton_path) = options.proton_path {
                environment.push(("PROTONPATH".to_string(), proton_path));
            }
            if let Some(locale) = options.wine_locale {
                environment.push(("LANG".to_string(), locale.clone()));
                environment.push(("LC_ALL".to_string(), locale));
            }
            environment.push(("GAMEID".to_string(), options.umu_game_id));
            let mut command = if std::env::var_os("FLATPAK_ID").is_some() {
                let mut command = Command::new("/app/bin/umu-run");
                if let Some(directory) = working_directory {
                    command.current_dir(directory);
                }
                for (name, value) in environment {
                    command.env(name, value);
                }
                command
            } else {
                let umu_path = find_host_command("umu-run".to_string()).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "UMU Launcher not found")
                })?;
                host_command(umu_path, environment, working_directory)
            };
            command.arg(path);
            command.args(&parsed_args);
            attach_launch_log(&mut command, stdout_log, stderr_log);
            Ok(Some(command.spawn()?))
        }
        LaunchMode::Steam => {
            let app_id = options.steam_app_id.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Steam App ID is not configured",
                )
            })?;

            if cfg!(target_os = "windows") {
                crate::system::open_url(&format!("steam://rungameid/{app_id}"))?;
                Ok(None)
            } else {
                let mut command =
                    host_command("steam".to_string(), options.launch_environment, working_directory);
                command.arg("-applaunch").arg(app_id.to_string());
                attach_launch_log(&mut command, stdout_log, stderr_log);
                Ok(Some(command.spawn()?))
            }
        }
    }
}

pub enum ToolLaunch {
    Process(Child),
    Queued,
}

pub fn launch_steam_tool(
    tool_path: String,
    app_id: u32,
    launch_environment: Vec<(String, String)>,
    launch_log_path: Option<PathBuf>,
) -> Result<ToolLaunch, io::Error> {
    let path = PathBuf::from(tool_path);
    let working_directory = path.parent().map(Path::to_path_buf);

    if cfg!(target_os = "windows") {
        let (stdout_log, stderr_log) = open_launch_log(launch_log_path)?;

        let mut command = host_command(
            path.to_string_lossy().to_string(),
            launch_environment,
            working_directory,
        );

        attach_launch_log(&mut command, stdout_log, stderr_log);
        return Ok(ToolLaunch::Process(command.spawn()?));
    }

    let tools_directory = yakata_data_dir()?.join("steam-tools");
    let active_path = tools_directory.join(format!("active-{app_id}"));

    if !active_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "Steam wrapper is not active; launch the game from Steam first",
        ));
    }

    let request_path = tools_directory.join(format!("tool-request-{app_id}"));
    fs::write(request_path, format!("{}\n", path.display()))?;

    Ok(ToolLaunch::Queued)
}

#[cfg(target_os = "linux")]
pub fn install_steam_tool_wrapper() -> Result<String, io::Error> {
    let tools_directory = yakata_data_dir()?.join("steam-tools");
    fs::create_dir_all(&tools_directory)?;

    let wrapper_path = tools_directory.join("yakata-steam-wrapper");
    let helper_path = tools_directory.join("yakata-steam-helper");

    write_executable_script(wrapper_path.clone(), STEAM_WRAPPER_SCRIPT.to_string())?;
    write_executable_script(helper_path, STEAM_HELPER_SCRIPT.to_string())?;

    Ok(format!("{} %command%", wrapper_path.display()))
}

#[cfg(target_os = "linux")]
fn write_executable_script(path: PathBuf, contents: String) -> Result<(), io::Error> {
    fs::write(&path, contents)?;

    let permissions = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, permissions)?;

    Ok(())
}

fn host_command(
    program: String,
    environment: Vec<(String, String)>,
    working_directory: Option<PathBuf>,
) -> Command {
    if std::env::var_os("FLATPAK_ID").is_some() {
        let mut command = Command::new("flatpak-spawn");
        command.arg("--host");
        if let Ok(display) = std::env::var("DISPLAY") {
            command.arg(format!("--env=DISPLAY={display}"));
        }
        if let Some(directory) = working_directory {
            command.arg(format!("--directory={}", directory.to_string_lossy()));
        }
        for (name, value) in environment {
            command.arg(format!("--env={name}={value}"));
        }
        command.arg(program);
        command
    } else {
        let mut command = Command::new(program);
        if let Some(directory) = working_directory {
            command.current_dir(directory);
        }
        for (name, value) in environment {
            command.env(name, value);
        }
        command
    }
}
fn attach_launch_log(
    command: &mut Command,
    stdout_log: Option<std::fs::File>,
    stderr_log: Option<std::fs::File>,
) {
    if let Some(file) = stdout_log {
        command.stdout(Stdio::from(file));
    }
    if let Some(file) = stderr_log {
        command.stderr(Stdio::from(file));
    }
}
pub fn parse_launch_environment(environment_text: String) -> Result<Vec<(String, String)>, String> {
    let mut environment = Vec::new();
    for (index, line) in environment_text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            return Err(format!("Line {} must be KEY=value", index + 1));
        };
        let name = name.trim();
        let value = value.trim();
        if name.is_empty() {
            return Err(format!("Line {} has an empty variable name", index + 1));
        }
        if name.contains(char::is_whitespace) {
            return Err(format!(
                "Line {} has whitespace in the variable name",
                index + 1
            ));
        }
        environment.push((name.to_string(), value.to_string()));
    }
    Ok(environment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_steam_app_id() {
        assert_eq!(parse_steam_app_id("1989270"), Some(1989270));
        assert_eq!(
            parse_steam_app_id("https://store.steampowered.com/app/1989270/Slay_the_Princess/"),
            Some(1989270)
        );
        assert_eq!(
            parse_steam_app_id("steam://rungameid/1989270"),
            Some(1989270)
        );
        assert_eq!(parse_steam_app_id("e1989270"), None);
        assert_eq!(parse_steam_app_id("Higurashi 1 (Chapter 2)"), None);
        assert_eq!(parse_steam_app_id("  412830  "), Some(412830));
        assert_eq!(parse_steam_app_id(""), None);
        assert_eq!(parse_steam_app_id("invalid"), None);
    }

    #[test]
    fn test_split_launch_arguments() {
        let args = split_launch_arguments("-foo \"bar baz\" 'qux quux' --flag");
        assert_eq!(args, vec!["-foo", "bar baz", "qux quux", "--flag"]);
    }
}
