use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LaunchMode {
    Native,
    Wine,
    Proton,
    Steam,
}

impl Default for LaunchMode {
    fn default() -> Self {
        LaunchMode::Native
    }
}

pub fn default_umu_game_id() -> String {
    "umu-default".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NovelStatus {
    Reading,
    Completed,
    PlanToRead,
    OnHold,
    Dropped,
}

#[allow(dead_code)]
impl NovelStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NovelStatus::Reading => "Reading",
            NovelStatus::Completed => "Completed",
            NovelStatus::PlanToRead => "Plan to read",
            NovelStatus::OnHold => "On hold",
            NovelStatus::Dropped => "Dropped",
        }
    }

    pub fn all() -> &'static [NovelStatus] {
        &[
            NovelStatus::Reading,
            NovelStatus::Completed,
            NovelStatus::PlanToRead,
            NovelStatus::OnHold,
            NovelStatus::Dropped,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualNovel {
    pub id: u64,
    pub title: String,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub is_favourite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub cover_path: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    #[serde(default)]
    pub game_folder: Option<String>,
    #[serde(default)]
    pub launch_mode: LaunchMode,
    #[serde(default)]
    pub steam_app_id: Option<u32>,
    #[serde(default)]
    pub wine_binary: Option<String>,
    #[serde(default)]
    pub wine_prefix: Option<String>,
    #[serde(default)]
    pub wine_locale: Option<String>,
    #[serde(default)]
    pub proton_path: Option<String>,
    #[serde(default = "default_umu_game_id")]
    pub umu_game_id: String,
    #[serde(default)]
    pub launch_arguments: String,
    #[serde(default)]
    pub launch_environment: String,
    pub notes: String,
    pub routes: Vec<StoryRoute>,
    #[serde(default)]
    pub active_route: Option<String>,
    pub play_sessions: Vec<PlaySession>,
    #[serde(default)]
    pub status: Option<NovelStatus>,
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub vndb_id: Option<String>,
    #[serde(default)]
    pub run_with_locale_emulator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryRoute {
    pub name: String,
    pub completed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaySession {
    #[serde(alias = "visual_novel_id")]
    pub vn_id: u64,
    pub started_at: String,
    pub duration_seconds: u64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppNotification {
    pub level: NotificationLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibrarySortMode {
    Added,
    TitleAsc,
    TitleDesc,
    LastPlayed,
    MostPlaytime,
    RatingDesc,
}

impl Default for LibrarySortMode {
    fn default() -> Self {
        LibrarySortMode::Added
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibraryFilterMode {
    All,
    Favourites,
    Status(NovelStatus),
}

impl Default for LibraryFilterMode {
    fn default() -> Self {
        LibraryFilterMode::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    #[serde(default)]
    pub discord_rich_presence_enabled: bool,
    #[serde(default = "default_discord_status_text")]
    pub discord_status_text: String,
    #[serde(default)]
    pub discord_show_active_route: bool,
    #[serde(default)]
    pub discord_custom_cover_url: String,
    #[serde(default)]
    pub library_sort_mode: LibrarySortMode,
    #[serde(default)]
    pub library_filter_mode: LibraryFilterMode,
    #[serde(default)]
    pub custom_background_path: Option<String>,
    #[serde(default)]
    pub leproc_path: Option<String>,
    #[serde(default)]
    pub blur_nsfw: bool,
}

fn default_discord_status_text() -> String {
    "Reading".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            discord_rich_presence_enabled: true,
            discord_status_text: default_discord_status_text(),
            discord_show_active_route: true,
            discord_custom_cover_url: String::new(),
            library_sort_mode: LibrarySortMode::default(),
            library_filter_mode: LibraryFilterMode::default(),
            custom_background_path: None,
            leproc_path: None,
            blur_nsfw: false,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novel_status_all_variants() {
        let all = NovelStatus::all();
        assert_eq!(all.len(), 5);
        assert_eq!(NovelStatus::Reading.as_str(), "Reading");
        assert_eq!(NovelStatus::Completed.as_str(), "Completed");
        assert_eq!(NovelStatus::PlanToRead.as_str(), "Plan to read");
        assert_eq!(NovelStatus::OnHold.as_str(), "On hold");
        assert_eq!(NovelStatus::Dropped.as_str(), "Dropped");
    }

    #[test]
    fn test_launch_mode_default() {
        assert_eq!(LaunchMode::default(), LaunchMode::Native);
    }

    #[test]
    fn test_app_settings_default_values() {
        let settings = AppSettings::default();
        assert!(settings.discord_rich_presence_enabled);
        assert_eq!(settings.discord_status_text, "Reading");
        assert!(settings.discord_show_active_route);
        assert!(!settings.blur_nsfw);
        assert!(settings.custom_background_path.is_none());
        assert!(settings.leproc_path.is_none());
    }

    #[test]
    fn test_novel_status_serde_roundtrip() {
        let status = NovelStatus::Completed;
        let json = serde_json::to_string(&status).expect("Serialization failed");
        let parsed: NovelStatus = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(status, parsed);
    }
}
