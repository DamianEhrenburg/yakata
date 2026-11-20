use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LaunchMode {
    #[default]
    Native,
    Wine,
    Proton,
    Steam,
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

impl NovelStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NovelStatus::Reading => "Reading",
            NovelStatus::Completed => "Completed",
            NovelStatus::PlanToRead => "Plan to Read",
            NovelStatus::OnHold => "On Hold",
            NovelStatus::Dropped => "Dropped",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            NovelStatus::Reading => "reading",
            NovelStatus::Completed => "completed",
            NovelStatus::PlanToRead => "plan_to_read",
            NovelStatus::OnHold => "on_hold",
            NovelStatus::Dropped => "dropped",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "reading" => Some(NovelStatus::Reading),
            "completed" => Some(NovelStatus::Completed),
            "plan_to_read" => Some(NovelStatus::PlanToRead),
            "on_hold" => Some(NovelStatus::OnHold),
            "dropped" => Some(NovelStatus::Dropped),
            _ => None,
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
    #[serde(default)]
    pub is_nsfw: bool,
}

impl Default for VisualNovel {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            cover_url: None,
            cover_path: None,
            description: None,
            is_favourite: false,
            tags: Vec::new(),
            executable_path: None,
            game_folder: None,
            launch_mode: LaunchMode::default(),
            steam_app_id: None,
            wine_binary: None,
            wine_prefix: None,
            wine_locale: None,
            proton_path: None,
            umu_game_id: default_umu_game_id(),
            launch_arguments: String::new(),
            launch_environment: String::new(),
            notes: String::new(),
            routes: Vec::new(),
            active_route: None,
            play_sessions: Vec::new(),
            status: None,
            rating: None,
            vndb_id: None,
            run_with_locale_emulator: false,
            is_nsfw: false,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LibrarySortMode {
    #[default]
    Added,
    TitleAsc,
    TitleDesc,
    LastPlayed,
    MostPlaytime,
    RatingDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LibraryFilterMode {
    #[default]
    All,
    Favourites,
    Status(NovelStatus),
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
    #[serde(default = "default_sort_descending")]
    pub library_sort_descending: bool,
    #[serde(default, skip_serializing)]
    pub library_filter_mode: LibraryFilterMode,
    #[serde(default)]
    pub custom_background_path: Option<String>,
    #[serde(default)]
    pub leproc_path: Option<String>,
    #[serde(default)]
    pub blur_nsfw: bool,
    #[serde(default)]
    pub filter_favourites_only: bool,
    #[serde(default)]
    pub filter_status: Option<NovelStatus>,
}

fn default_discord_status_text() -> String {
    "Reading".to_string()
}

fn default_sort_descending() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            discord_rich_presence_enabled: true,
            discord_status_text: default_discord_status_text(),
            discord_show_active_route: true,
            discord_custom_cover_url: String::new(),
            library_sort_mode: LibrarySortMode::default(),
            library_sort_descending: true,
            library_filter_mode: LibraryFilterMode::default(),
            custom_background_path: None,
            leproc_path: None,
            blur_nsfw: false,
            filter_favourites_only: false,
            filter_status: None,
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
        assert_eq!(NovelStatus::PlanToRead.as_str(), "Plan to Read");
        assert_eq!(NovelStatus::OnHold.as_str(), "On Hold");
        assert_eq!(NovelStatus::Dropped.as_str(), "Dropped");
        for status in all {
            assert_eq!(NovelStatus::from_key(status.key()), Some(*status));
        }
    }

    #[test]
    fn test_skip_serializing_old_filter_mode() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.contains("library_filter_mode"));
        assert!(json.contains("filter_favourites_only"));
        assert!(json.contains("filter_status"));
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
        assert!(!settings.filter_favourites_only);
        assert!(settings.filter_status.is_none());
        assert!(settings.library_sort_descending);
    }

    #[test]
    fn test_backwards_compatibility_settings_without_new_filters() {
        let old_json = r#"{
            "discord_rich_presence_enabled": true,
            "discord_status_text": "Reading",
            "discord_show_active_route": true,
            "discord_custom_cover_url": "",
            "library_sort_mode": "Added",
            "library_filter_mode": "All"
        }"#;
        let settings: AppSettings = serde_json::from_str(old_json).expect("Failed to deserialize old settings");
        assert!(!settings.filter_favourites_only);
        assert!(settings.filter_status.is_none());
        assert!(settings.library_sort_descending);
    }

    #[test]
    fn test_novel_status_serde_roundtrip() {
        let status = NovelStatus::Completed;
        let json = serde_json::to_string(&status).expect("Serialization failed");
        let parsed: NovelStatus = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(status, parsed);
    }
}
