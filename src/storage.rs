use crate::models::{AppSettings, VisualNovel};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::{fs, io};

fn library_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn settings_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn yakata_data_dir() -> Result<PathBuf, io::Error> {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
        && exe_dir.join("portable.txt").exists()
    {
        let data_dir = exe_dir.join("data");
        fs::create_dir_all(&data_dir)?;
        return Ok(data_dir);
    }

    let data_dir = match dirs::data_dir() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find an app data directory",
            ));
        }
    };
    let yakata_dir = data_dir.join("yakata");
    let kakera_dir = data_dir.join("kakera");
    if !yakata_dir.join("library_json").exists() && kakera_dir.join("library_json").exists() {
        let _ = fs::create_dir_all(&yakata_dir);
        let _ = fs::copy(kakera_dir.join("library_json"), yakata_dir.join("library_json"));
        if kakera_dir.join("settings.json").exists() && !yakata_dir.join("settings.json").exists() {
            let _ = fs::copy(kakera_dir.join("settings.json"), yakata_dir.join("settings.json"));
        }
    }
    Ok(yakata_dir)
}

pub fn library_file_path() -> Result<PathBuf, io::Error> {
    Ok(yakata_data_dir()?.join("library_json"))
}

fn read_library_unlocked() -> Result<Vec<VisualNovel>, io::Error> {
    let library_file = library_file_path()?;
    let json_text = match fs::read_to_string(&library_file) {
        Ok(text) => text,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(Vec::new());
            }
            return Err(error);
        }
    };
    serde_json::from_str::<Vec<VisualNovel>>(&json_text)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_library_unlocked(library: &[VisualNovel]) -> Result<(), io::Error> {
    let library_file = library_file_path()?;
    let library_dir = match library_file.parent() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Library file path has no parent directory",
            ));
        }
    };
    fs::create_dir_all(library_dir)?;
    let json_text = serde_json::to_string_pretty(library)?;
    let temp_file = library_file.with_extension("tmp");
    fs::write(&temp_file, json_text)?;
    fs::rename(&temp_file, &library_file)?;
    Ok(())
}

pub fn load_library() -> Result<Vec<VisualNovel>, io::Error> {
    let _guard = library_lock().lock().unwrap_or_else(|e| e.into_inner());
    read_library_unlocked()
}

pub fn save_library(library: Vec<VisualNovel>) -> Result<(), io::Error> {
    let _guard = library_lock().lock().unwrap_or_else(|e| e.into_inner());
    write_library_unlocked(&library)
}

pub fn export_library_backup(dest_path: &Path, library: &[VisualNovel]) -> Result<(), io::Error> {
    let json_text = serde_json::to_string_pretty(library)?;
    fs::write(dest_path, json_text)?;
    Ok(())
}

pub fn import_library_backup(src_path: &Path) -> Result<Vec<VisualNovel>, io::Error> {
    let json_text = fs::read_to_string(src_path)?;
    let library = serde_json::from_str::<Vec<VisualNovel>>(&json_text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let _guard = library_lock().lock().unwrap_or_else(|e| e.into_inner());
    write_library_unlocked(&library)?;
    Ok(library)
}

pub fn settings_file_path() -> Result<PathBuf, io::Error> {
    Ok(yakata_data_dir()?.join("settings.json"))
}

pub fn load_settings() -> Result<AppSettings, io::Error> {
    let _guard = settings_lock().lock().unwrap_or_else(|e| e.into_inner());
    let settings_file = settings_file_path()?;
    let json_text = match fs::read_to_string(&settings_file) {
        Ok(text) => text,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(AppSettings::default());
            }
            return Err(error);
        }
    };
    let settings = match serde_json::from_str::<AppSettings>(&json_text) {
        Ok(settings) => settings,
        Err(error) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, error));
        }
    };
    Ok(settings)
}

pub fn save_settings(settings: AppSettings) -> Result<(), io::Error> {
    let _guard = settings_lock().lock().unwrap_or_else(|e| e.into_inner());
    let settings_file = settings_file_path()?;
    let settings_dir = match settings_file.parent() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Settings file path has no parent directory",
            ));
        }
    };
    fs::create_dir_all(settings_dir)?;
    let json_text = serde_json::to_string_pretty(&settings)?;
    let temp_file = settings_file.with_extension("tmp");
    fs::write(&temp_file, json_text)?;
    fs::rename(&temp_file, &settings_file)?;
    Ok(())
}
