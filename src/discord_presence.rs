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

        if settings.discord_show_active_route && vn.active_route.is_some() {
            let route_name = vn.active_route.as_ref().unwrap();
            activity = activity.details(route_name.clone());
        } else if !settings.discord_status_text.trim().is_empty() {
            activity = activity.details(settings.discord_status_text.trim());
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
