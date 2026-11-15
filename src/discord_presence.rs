use crate::models::{AppSettings, VisualNovel};
use chrono::{DateTime, Utc};
use discord_rich_presence::error::Error as DiscordError;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

const DISCORD_APP_ID: &str = "1512156673358172312";

pub struct DiscordPresence {
    client: DiscordIpcClient,
}

impl DiscordPresence {
    pub fn start_for_vn(
        vn: &VisualNovel,
        started_at: DateTime<Utc>,
        settings: AppSettings,
    ) -> Result<Self, DiscordError> {
        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
        client.connect()?;

        let timestamps = activity::Timestamps::new().start(started_at.timestamp_millis());
        let mut assets = activity::Assets::new().large_text(vn.title.clone());

        let cover_image = if !settings.discord_custom_cover_url.is_empty() {
            Some(settings.discord_custom_cover_url.clone())
        } else {
            vn.cover_url.clone()
        };

        if let Some(cover_url) = cover_image {
            assets = assets.large_image(cover_url);
        }

        let mut activity = activity::Activity::new()
            .name(vn.title.clone())
            .timestamps(timestamps)
            .assets(assets)
            .activity_type(activity::ActivityType::Playing);

        let status_text = settings.discord_status_text.trim();
        let active_route_str = if settings.discord_show_active_route {
            vn.active_route.as_deref().map(format_discord_route).filter(|s| !s.is_empty())
        } else {
            None
        };

        match (!status_text.is_empty(), active_route_str) {
            (true, Some(route)) => {
                activity = activity.details(status_text).state(route);
            }
            (true, None) => {
                activity = activity.details(status_text);
            }
            (false, Some(route)) => {
                activity = activity.details(route);
            }
            (false, None) => {}
        }

        client.set_activity(activity)?;
        match client.recv() {
            Ok((_opcode, response)) => {
                println!("Discord Rich Presence response: {response}");
            }
            Err(error) => {
                println!("Could not read Discord Rich Presence response: {error}");
            }
        }

        Ok(Self { client })
    }

    pub fn clear(mut self) -> Result<(), DiscordError> {
        self.client.clear_activity()?;
        self.client.close()?;
        Ok(())
    }
}

pub fn format_discord_route(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    let already_qualified = lower.contains("route")
        || lower.contains("ending")
        || lower.ends_with(" end")
        || lower.contains("chapter")
        || lower.contains("episode")
        || lower.starts_with("ep.")
        || lower.starts_with("ch.")
        || lower.contains("arc");

    if already_qualified {
        trimmed.to_string()
    } else {
        format!("{trimmed} Route")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_discord_route() {
        assert_eq!(format_discord_route("Kasumi"), "Kasumi Route");
        assert_eq!(format_discord_route("Kasumi route"), "Kasumi route");
        assert_eq!(format_discord_route("Saber Route"), "Saber Route");
        assert_eq!(format_discord_route("True End"), "True End");
        assert_eq!(format_discord_route("Bad Ending 1"), "Bad Ending 1");
        assert_eq!(format_discord_route("Chapter 3"), "Chapter 3");
        assert_eq!(format_discord_route("Episode 1: Onikakushi"), "Episode 1: Onikakushi");
        assert_eq!(format_discord_route("   "), "");
    }
}
