use crate::models::{AppSettings, VisualNovel};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn yakata_data_dir() -> Result<PathBuf, io::Error> {
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

pub fn load_library() -> Result<Vec<VisualNovel>, io::Error> {
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
    let library = match serde_json::from_str::<Vec<VisualNovel>>(&json_text) {
        Ok(vns) => vns,
        Err(error) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, error));
        }
    };
    Ok(library)
}

pub fn save_library(library: Vec<VisualNovel>) -> Result<(), io::Error> {
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
    let json_text = serde_json::to_string_pretty(&library)?;
    fs::write(library_file, json_text)?;
    Ok(())
}

pub fn export_library_backup(dest_path: &Path) -> Result<(), io::Error> {
    let library = load_library()?;
    let json_text = serde_json::to_string_pretty(&library)?;
    fs::write(dest_path, json_text)?;
    Ok(())
}

pub fn import_library_backup(src_path: &Path) -> Result<Vec<VisualNovel>, io::Error> {
    let json_text = fs::read_to_string(src_path)?;
    let library = serde_json::from_str::<Vec<VisualNovel>>(&json_text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    save_library(library.clone())?;
    Ok(library)
}

#[allow(dead_code)]
pub fn add_vn_to_library(vn: VisualNovel) -> Result<(), io::Error> {
    let mut library = load_library()?;
    library.push(vn);
    save_library(library)?;
    Ok(())
}

#[allow(dead_code)]
pub fn update_vn_in_library<F>(vn_id: u64, update_fn: F) -> Result<(), io::Error>
where
    F: FnOnce(&mut VisualNovel),
{
    let mut library = load_library()?;
    if let Some(vn) = library.iter_mut().find(|vn| vn.id == vn_id) {
        update_fn(vn);
        save_library(library)?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("VN with ID {vn_id} not found in library"),
        ))
    }
}

#[allow(dead_code)]
pub fn remove_vn_from_library(vn_id: u64) -> Result<(), io::Error> {
    let mut library = load_library()?;
    let initial_len = library.len();
    library.retain(|vn| vn.id != vn_id);
    if library.len() == initial_len {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("VN with ID {vn_id} not found in library"),
        ));
    }
    save_library(library)?;
    Ok(())
}

pub fn add_play_session_to_library(
    vn_id: u64,
    play_session: crate::models::PlaySession,
) -> Result<(), io::Error> {
    let mut library = load_library()?;
    for vn in &mut library {
        if vn.id == vn_id {
            vn.play_sessions.push(play_session.clone());
        }
    }
    save_library(library)?;
    Ok(())
}

pub fn settings_file_path() -> Result<PathBuf, io::Error> {
    Ok(yakata_data_dir()?.join("settings.json"))
}

pub fn load_settings() -> Result<AppSettings, io::Error> {
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
    fs::write(settings_file, json_text)?;
    Ok(())
}
