mod covers;
mod discord_presence;
mod game_process;
mod launcher;
mod logs;
mod models;
mod storage;
mod system;
mod views;
mod vn_markup;
mod vndb;
mod wine;

use covers::cache_cover_image;
use discord_presence::DiscordPresence;
#[cfg(target_os = "linux")]
use launcher::install_steam_tool_wrapper;
use launcher::{ToolLaunch, launch_executable, launch_steam_tool, parse_launch_environment};

use logs::{launch_logs_dir, new_launch_log_path, update_latest_launch_log};
use models::{
    AppNotification, AppSettings, LaunchMode, LibraryFilterMode, LibrarySortMode,
    NotificationLevel, NovelStatus, PlaySession, StoryRoute, VisualNovel, default_umu_game_id,
};
use storage::{
    add_play_session_to_library, yakata_data_dir, load_library, load_settings, save_library,
    save_settings,
};
use system::{is_flatpak_document_portal_path, open_folder, umu_launcher_is_available};
use views::{AddVnForm, DetailView, Dropdown, DropdownOption, LibraryView, NewVN, SettingsView, local_path_to_data_url};

use chrono::Utc;
use dioxus::desktop::{
    Config, WindowBuilder, icon_from_memory,
    tao::{dpi::LogicalSize, window::ResizeDirection},
};
use dioxus::prelude::*;
use std::thread;
use std::time::Instant;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const BG_IMAGE: Asset = asset!("/assets/yakata_bg.jpg");
const LOGO_IMAGE: Asset = asset!("/assets/yakatalogo.png");
const STARS_IMAGE: Asset = asset!("/assets/stars.jpg");

fn main() {
    let config = Config::new()
        .with_custom_head(r#"<meta name="color-scheme" content="dark"><link rel="stylesheet" href="/assets/main.css"><style>html,body{background-color:#0b0c10 !important;margin:0;}.logo-image{max-width:140px;height:auto;}.app-frame{visibility:hidden;}</style>"#.to_string())
        .with_window(
        WindowBuilder::new()
            .with_title("Yakata")
            .with_maximized(true)
            .with_inner_size(LogicalSize::new(1340.0, 840.0))
            .with_min_inner_size(LogicalSize::new(960.0, 640.0))
            .with_decorations(false),
    );

    let config = match icon_from_memory(include_bytes!("../assets/favicon.ico")) {
        Ok(icon) => config.with_icon(icon),
        Err(error) => {
            eprintln!("Could not load app icon: {error}");
            config
        }
    };

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

#[derive(Debug, Clone, PartialEq)]
enum AppView {
    Library,
    Settings,
}

#[component]
fn ResizeHandle(class: String, direction: ResizeDirection) -> Element {
    let window = dioxus::desktop::window();

    rsx! {
        div {
            class: "resize-handle {class}",

            onmousedown: move |_| {
                if let Err(error) = window.drag_resize_window(direction) {
                    eprintln!("Could not begin resizing window: {error}");
                }
            },
        }
    }
}

#[component]
fn App() -> Element {
    let saved_library = match load_library() {
        Ok(library) => library,
        Err(error) => {
            println!("Could not load library: {error}");
            Vec::new()
        }
    };

    let saved_settings = match load_settings() {
        Ok(settings) => settings,
        Err(error) => {
            println!("Could not load settings: {error}");
            AppSettings::default()
        }
    };

    let mut vns = use_signal(move || saved_library);
    let mut selected_vn_id = use_signal(|| None::<u64>);

    let selected_vn_id_value = *selected_vn_id.read();

    let selected_vn = match selected_vn_id_value {
        Some(id) => vns.read().iter().find(|vn| vn.id == id).cloned(),
        None => None,
    };

    let saved_library_sort_mode = saved_settings.library_sort_mode.clone();
    let saved_library_filter_mode = saved_settings.library_filter_mode.clone();

    let mut selected_tag_filter = use_signal(|| None::<String>);
    let selected_tag = selected_tag_filter.read().clone();

    let mut settings = use_signal(move || saved_settings);

    let mut search_query = use_signal(String::new);
    let mut show_add_form = use_signal(|| false);

    let mut library_sort_mode = use_signal(|| saved_library_sort_mode);
    let selected_sort_mode = library_sort_mode.read().clone();

    let mut library_filter_mode = use_signal(|| saved_library_filter_mode);
    let selected_filter_mode = library_filter_mode.read().clone();

    let umu_is_available = use_hook(umu_launcher_is_available);

    let mut notification = use_signal(|| {
        if cfg!(target_os = "linux") && !umu_is_available {
            Some(AppNotification {
                level: NotificationLevel::Warning,
                message: "UMU Launcher was not found. Proton launches will not work until UMU is installed."
                    .to_string(),
            })
        } else {
            None
        }
    });

    let mut notification_token = use_signal(|| 0u64);
    use_effect(move || {
        if notification.read().is_some() {
            let current_token = *notification_token.peek() + 1;
            notification_token.set(current_token);
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                if *notification_token.peek() == current_token {
                    notification.set(None);
                }
            });
        }
    });

    let session_tx = use_hook(|| {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(u64, PlaySession)>();
        let mut vns_signal = vns;
        spawn(async move {
            while let Some((vn_id, session)) = rx.recv().await {
                for vn in vns_signal.write().iter_mut() {
                    if vn.id == vn_id {
                        vn.play_sessions.push(session.clone());
                    }
                }
            }
        });
        tx
    });

    let mut current_view = use_signal(|| AppView::Library);
    let selected_view = current_view.read().clone();

    let search_text = search_query.read().to_lowercase();

    let mut available_tags: Vec<String> =
        vns.read().iter().flat_map(|vn| vn.tags.clone()).collect();

    available_tags.sort_by_key(|tag| tag.to_lowercase());
    available_tags.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

    let mut filtered_vns: Vec<VisualNovel> = vns
        .read()
        .iter()
        .filter(|vn| {
            let matches_search = vn.title.to_lowercase().contains(&search_text);

            let matches_filter = match selected_filter_mode {
                LibraryFilterMode::All => true,
                LibraryFilterMode::Favourites => vn.is_favourite,
                LibraryFilterMode::Status(status) => vn.status == Some(status),
            };

            let matches_tag = match selected_tag.clone() {
                Some(tag) => vn
                    .tags
                    .iter()
                    .any(|vn_tag| vn_tag.eq_ignore_ascii_case(&tag)),
                None => true,
            };

            matches_search && matches_filter && matches_tag
        })
        .cloned()
        .collect();

    match selected_sort_mode {
        LibrarySortMode::Added => {
            filtered_vns.sort_by(|a, b| b.id.cmp(&a.id));
        }
        LibrarySortMode::RatingDesc => {
            filtered_vns.sort_by(|a, b| b.rating.unwrap_or(0).cmp(&a.rating.unwrap_or(0)));
        }
        LibrarySortMode::TitleAsc => {
            filtered_vns.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        LibrarySortMode::TitleDesc => {
            filtered_vns.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()));
        }
        LibrarySortMode::LastPlayed => {
            filtered_vns.sort_by(|a, b| latest_played_at(b).cmp(&latest_played_at(a)));
        }
        LibrarySortMode::MostPlaytime => {
            filtered_vns.sort_by(|a, b| total_playtime_seconds(b).cmp(&total_playtime_seconds(a)));
        }
    }

    let desktop = dioxus::desktop::window();
    let drag_win = desktop.clone();
    let min_win = desktop.clone();
    let max_win = desktop.clone();
    let close_win = desktop.clone();

    let data_dir_text = match yakata_data_dir() {
        Ok(path) => path.display().to_string(),
        Err(error) => format!("Could not find data folder: {error}"),
    };

    let wine_runners = use_hook(wine::detect_wine_runners);
    let proton_runners = use_hook(wine::detect_proton_runners);
    let steam_prefixes = use_hook(wine::detect_steam_prefixes);

    let steam_launch_option = use_hook(|| {
        #[cfg(target_os = "linux")]
        {
            install_steam_tool_wrapper()
                .unwrap_or_else(|error| format!("Could not install Steam wrapper: {error}"))
        }

        #[cfg(not(target_os = "linux"))]
        {
            String::new()
        }
    });

    let custom_bg_url = settings
        .read()
        .custom_background_path
        .as_ref()
        .and_then(|p| local_path_to_data_url(p));
    let bg_style = match custom_bg_url {
        Some(url) => format!("--app-bg-image: url('{url}'); --stars-image: url('{STARS_IMAGE}');"),
        None => format!("--app-bg-image: url('{BG_IMAGE}'); --stars-image: url('{STARS_IMAGE}');"),
    };

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        main {
            class: "app-frame",
            style: "{bg_style}",

            ResizeHandle { class: "resize-north", direction: ResizeDirection::North }
            ResizeHandle {
                class: "resize-north-east",
                direction: ResizeDirection::NorthEast,
            }
            ResizeHandle { class: "resize-east", direction: ResizeDirection::East }
            ResizeHandle {
                class: "resize-south-east",
                direction: ResizeDirection::SouthEast,
            }
            ResizeHandle { class: "resize-south", direction: ResizeDirection::South }
            ResizeHandle {
                class: "resize-south-west",
                direction: ResizeDirection::SouthWest,
            }
            ResizeHandle { class: "resize-west", direction: ResizeDirection::West }
            ResizeHandle {
                class: "resize-north-west",
                direction: ResizeDirection::NorthWest,
            }

            div { class: "star-overlay" }

            div { class: "win-titlebar",

                div {
                    class: "win-drag-region",

                    onmousedown: move |_| {
                        drag_win.drag();
                    },

                    span { class: "win-title", "Yakata" }
                }

                div { class: "win-controls",

                    button {
                        class: "win-control-button",
                        onclick: move |_| {
                            min_win.set_minimized(true);
                        },

                        "-"
                    }

                    button {
                        class: "win-control-button",
                        onclick: move |_| {
                            max_win.toggle_maximized();
                        },

                        "□"
                    }

                    button {
                        class: "win-control-button close",
                        onclick: move |_| {
                            close_win.close();
                        },

                        "×"
                    }
                }
            }

            if let Some(active_notification) = notification.read().clone() {
                div {
                    class: match active_notification.level {
                        NotificationLevel::Info => "notification-banner info",
                        NotificationLevel::Warning => "notification-banner warning",
                        NotificationLevel::Error => "notification-banner error",
                    },

                    span { "{active_notification.message}" }

                    button {
                        class: "notification-dismiss",
                        title: "Dismiss notification",

                        onclick: move |_| {
                            notification.set(None);
                        },

                        "×"
                    }
                }
            }

            div { class: "app-body",

                aside { class: "sidebar",

                    div { class: "logo",

                        img {
                            class: "logo-image",
                            src: LOGO_IMAGE,
                            alt: "Yakata",
                        }
                    }

                    nav { class: "sidebar-nav",

                        button {
                            class: if selected_view == AppView::Library { "nav-item active" } else { "nav-item" },

                            onclick: move |_| {
                                current_view.set(AppView::Library);
                            },

                            "Library"
                        }
                        button {
                            class: if selected_view == AppView::Settings { "nav-item active" } else { "nav-item" },

                            onclick: move |_| {
                                current_view.set(AppView::Settings);
                            },

                            "Settings"
                        }
                    }
                }

                section { class: if selected_view == AppView::Library { "main-area" } else { "main-area settings-view" },

                    if selected_view == AppView::Library {
                        header { class: "topbar",

                            input {
                                class: "search-input",
                                placeholder: "Search visual novels...",
                                value: "{search_query}",

                                oninput: move |event| {
                                    search_query.set(event.value());
                                },
                            }

                            div { class: "topbar-select-field sort-field",
                                span { class: "topbar-select-label", "Sort by" }

                                Dropdown {
                                    options: vec![
                                        DropdownOption::new("added", "Added"),
                                        DropdownOption::new("rating-desc", "Rating"),
                                        DropdownOption::new("title-asc", "A-Z"),
                                        DropdownOption::new("title-desc", "Z-A"),
                                        DropdownOption::new("last-played", "Last Played"),
                                        DropdownOption::new("most-playtime", "Most Playtime"),
                                    ],
                                    selected_value: match selected_sort_mode {
                                        LibrarySortMode::Added => "added",
                                        LibrarySortMode::RatingDesc => "rating-desc",
                                        LibrarySortMode::TitleAsc => "title-asc",
                                        LibrarySortMode::TitleDesc => "title-desc",
                                        LibrarySortMode::LastPlayed => "last-played",
                                        LibrarySortMode::MostPlaytime => "most-playtime",
                                    }.to_string(),
                                    on_select: move |val: String| {
                                        let sort_mode = match val.as_str() {
                                            "rating-desc" => LibrarySortMode::RatingDesc,
                                            "title-asc" => LibrarySortMode::TitleAsc,
                                            "title-desc" => LibrarySortMode::TitleDesc,
                                            "last-played" => LibrarySortMode::LastPlayed,
                                            "most-playtime" => LibrarySortMode::MostPlaytime,
                                            _ => LibrarySortMode::Added,
                                        };

                                        library_sort_mode.set(sort_mode.clone());
                                        settings.write().library_sort_mode = sort_mode;
                                        save_settings_or_log(&settings);
                                    },
                                }
                            }

                            div { class: "topbar-select-field view-field",
                                span { class: "topbar-select-label", "View" }

                                div { class: "topbar-segmented-control",
                                    button {
                                        class: if !matches!(selected_filter_mode, LibraryFilterMode::Favourites) { "topbar-segment active" } else { "topbar-segment" },

                                        onclick: move |_| {
                                            let filter_mode = LibraryFilterMode::All;

                                            library_filter_mode.set(filter_mode.clone());
                                            settings.write().library_filter_mode = filter_mode;
                                            save_settings_or_log(&settings);
                                        },

                                        "All"
                                    }

                                    button {
                                        class: if matches!(selected_filter_mode, LibraryFilterMode::Favourites) { "topbar-segment active" } else { "topbar-segment" },

                                        onclick: move |_| {
                                            let filter_mode = LibraryFilterMode::Favourites;

                                            library_filter_mode.set(filter_mode.clone());
                                            settings.write().library_filter_mode = filter_mode;
                                            save_settings_or_log(&settings);
                                        },

                                        "Favourites"
                                    }
                                }
                            }

                            div { class: "topbar-select-field status-field",
                                span { class: "topbar-select-label", "Status" }

                                Dropdown {
                                    options: vec![
                                        DropdownOption::new("all", "All Statuses"),
                                        DropdownOption::new("reading", "Reading"),
                                        DropdownOption::new("completed", "Completed"),
                                        DropdownOption::new("plan_to_read", "Plan to Read"),
                                        DropdownOption::new("on_hold", "On Hold"),
                                        DropdownOption::new("dropped", "Dropped"),
                                    ],
                                    selected_value: match selected_filter_mode {
                                        LibraryFilterMode::Status(NovelStatus::Reading) => "reading",
                                        LibraryFilterMode::Status(NovelStatus::Completed) => "completed",
                                        LibraryFilterMode::Status(NovelStatus::PlanToRead) => "plan_to_read",
                                        LibraryFilterMode::Status(NovelStatus::OnHold) => "on_hold",
                                        LibraryFilterMode::Status(NovelStatus::Dropped) => "dropped",
                                        _ => "all",
                                    }.to_string(),
                                    on_select: move |val: String| {
                                        let filter_mode = match val.as_str() {
                                            "reading" => LibraryFilterMode::Status(NovelStatus::Reading),
                                            "completed" => LibraryFilterMode::Status(NovelStatus::Completed),
                                            "plan_to_read" => LibraryFilterMode::Status(NovelStatus::PlanToRead),
                                            "on_hold" => LibraryFilterMode::Status(NovelStatus::OnHold),
                                            "dropped" => LibraryFilterMode::Status(NovelStatus::Dropped),
                                            _ => LibraryFilterMode::All,
                                        };

                                        library_filter_mode.set(filter_mode.clone());
                                        settings.write().library_filter_mode = filter_mode;
                                        save_settings_or_log(&settings);
                                    },
                                }
                            }

                            div { class: "topbar-select-field",
                                span { class: "topbar-select-label", "Tag" }

                                {
                                    let mut tag_opts = vec![DropdownOption::new("", "All Tags")];
                                    for tag in available_tags {
                                        tag_opts.push(DropdownOption::new(tag.clone(), tag.clone()));
                                    }
                                    let cur_tag = selected_tag.clone().unwrap_or_default();
                                    rsx! {
                                        Dropdown {
                                            options: tag_opts,
                                            selected_value: cur_tag,
                                            on_select: move |val: String| {
                                                if val.is_empty() {
                                                    selected_tag_filter.set(None);
                                                } else {
                                                    selected_tag_filter.set(Some(val));
                                                }
                                            },
                                        }
                                    }
                                }
                            }

                            button {
                                class: "icon-button",
                                onclick: move |_| {
                                    let next_value = !*show_add_form.read();
                                    show_add_form.set(next_value);
                                },

                                "+"
                            }

                        }
                    }

                    if selected_view == AppView::Library {
                        div { class: "app-layout",
                            section { class: "library-column",

                                if *show_add_form.read() {
                                    div { class: "add-vn-overlay",
                                        div { class: "add-vn-modal",
                                            AddVnForm {
                                                blur_nsfw: settings.read().blur_nsfw,
                                                existing_titles: vns.read().iter().map(|vn| vn.title.clone()).collect(),
                                                existing_vndb_ids: vns.read().iter().filter_map(|vn| vn.vndb_id.clone()).collect(),
                                                on_close: move |_| {
                                                    show_add_form.set(false);
                                                },

                                                on_add: move |new_vn: NewVN| {
                                                    let is_duplicate_title = vns.read().iter().any(|vn| vn.title.trim().eq_ignore_ascii_case(new_vn.title.trim()));
                                                    let is_duplicate_id = new_vn.vndb_id.as_ref().map_or(false, |id| {
                                                        vns.read().iter().any(|vn| vn.vndb_id.as_deref() == Some(id.as_str()))
                                                    });
                                                    if is_duplicate_title || is_duplicate_id {
                                                        notification.set(Some(AppNotification {
                                                            level: NotificationLevel::Warning,
                                                            message: format!("\"{}\" is already in your library.", new_vn.title),
                                                        }));
                                                        return;
                                                    }

                                                    let next_id = vns
                                                        .read()
                                                        .iter()
                                                        .map(|vn| vn.id)
                                                        .max()
                                                        .unwrap_or(0)
                                                        + 1;

                                                    let cover_url = new_vn.cover_url.clone();

                                                    let new_vn = VisualNovel {
                                                        id: next_id,
                                                        title: new_vn.title,
                                                        cover_url: new_vn.cover_url,
                                                        description: new_vn.description,
                                                        is_favourite: false,
                                                        tags: new_vn.tags,
                                                        cover_path: new_vn.cover_path,
                                                        executable_path: None,
                                                        launch_mode: LaunchMode::default(),
                                                        steam_app_id: None,
                                                        wine_binary: None,
                                                        wine_prefix: None,
                                                        wine_locale: None,
                                                        launch_arguments: String::new(),
                                                        launch_environment: String::new(),
                                                        proton_path: None,
                                                        umu_game_id: default_umu_game_id(),
                                                        notes: String::new(),
                                                        routes: Vec::new(),
                                                        active_route: None,
                                                        play_sessions: Vec::new(),
                                                        status: new_vn.status,
                                                        rating: new_vn.rating,
                                                        vndb_id: new_vn.vndb_id,
                                                        game_folder: None,
                                                        run_with_locale_emulator: false,
                                                    };

                                                    vns.write().push(new_vn);

                                                    if let Some(cover_url) = cover_url {
                                                        let mut vns_for_cover = vns;

                                                        spawn(async move {
                                                            match cache_cover_image(next_id, cover_url).await {
                                                                Ok(cover_path) => {
                                                                    for vn in vns_for_cover
                                                                        .write()
                                                                        .iter_mut()
                                                                    {
                                                                        if vn.id == next_id {
                                                                            vn.cover_path = Some(cover_path.clone());
                                                                        }
                                                                    }
                                                                    let save_result = save_library(vns_for_cover.read().clone());
                                                                    if let Err(error) = save_result {
                                                                        println!("Could not save cached cover path: {error}");
                                                                    }
                                                                }

                                                                Err(error) => {
                                                                    println!("Could not cache cover image: {error}");
                                                                }
                                                            }
                                                        });
                                                    }

                                                    let save_result = save_library(vns.read().clone());

                                                    if let Err(error) = save_result {
                                                        println!("Could not save library: {error}");
                                                    }

                                                    show_add_form.set(false);
                                                },
                                            }
                                        }
                                    }
                                }

                                LibraryView {
                                    vns: filtered_vns,
                                    selected_vn_id: selected_vn_id_value,
                                    blur_nsfw: settings.read().blur_nsfw,

                                    on_select: move |id| {
                                        selected_vn_id.set(Some(id));

                                    },

                                    on_toggle_favourite: move |id| {
                                        update_vn_and_save(
                                            &mut vns,
                                            id,
                                            move |vn| {
                                                vn.is_favourite = !vn.is_favourite;
                                            },
                                            "Could not save favourite".to_string(),
                                        );
                                    },
                                }
                            }

                            aside { class: "detail-column",
                                if let Some(vn) = selected_vn {
                                    DetailView {
                                        key: "{vn.id}",
                                        vn,
                                        blur_nsfw: settings.read().blur_nsfw,
                                        wine_runners: wine_runners.clone(),
                                        proton_runners: proton_runners.clone(),
                                        steam_prefixes: steam_prefixes.clone(),
                                        steam_launch_option: steam_launch_option.clone(),
                                        on_notes_change: move |(id, notes): (u64, String)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.notes = notes;
                                                },
                                                "Could not save notes".to_string(),
                                            );
                                        },

                                        on_route_add: move |(id, route_name)| {
                                            let new_route = StoryRoute {
                                                name: route_name,
                                                completed: false,
                                                notes: None,
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.routes.push(new_route);
                                                },
                                                "Could not save route".to_string(),
                                            );
                                        },

                                        on_route_toggle: move |(id, route_name)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    for route in vn.routes.iter_mut() {
                                                        if route.name == route_name {
                                                            route.completed = !route.completed;
                                                            break;
                                                        }
                                                    }
                                                },
                                                "Could not save route completion".to_string(),
                                            );
                                        },

                                        on_executable_path_change: move |(id, path): (u64, String)| {
                                            let trimmed_path = path.trim().to_string();

                                            if is_flatpak_document_portal_path(&trimmed_path) {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Error,
                                                            message: "The executable has been selected through Flatpak's temporary document portal. Move the game into a normal folder such as ~/Games, then type or select the real path."
                                                                .to_string(),
                                                        }),
                                                    );
                                                return;
                                            }
                                            let executable_path = if trimmed_path.is_empty() {
                                                None
                                            } else {
                                                Some(trimmed_path)
                                            };
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.executable_path = executable_path;
                                                },
                                                "Could not save executable path".to_string(),
                                            );
                                        },

                                        on_launch: move |id| {
                                            let vn = vns
                                                .read()
                                                .iter()
                                                .find(|vn| vn.id == id)
                                                .cloned();

                                            match vn {
                                                Some(vn) => {
                                                    let presence_vn = vn.clone();

                                                    match vn.executable_path {
                                                        Some(path) => {
                                                            if vn.launch_mode == LaunchMode::Proton
                                                                && !umu_launcher_is_available()
                                                            {
                                                                notification
                                                                    .set(
                                                                        Some(AppNotification {
                                                                            level: NotificationLevel::Error,
                                                                            message: "Could not launch VN: UMU Launcher was not found. Install umu-launcher to run games with Proton."
                                                                                .to_string(),
                                                                        }),
                                                                    );
                                                                return;
                                                            }
                                                            let launch_log_path = match new_launch_log_path(
                                                                vn.id,
                                                                vn.title.clone(),
                                                            ) {
                                                                Ok(path) => Some(path),
                                                                Err(error) => {
                                                                    notification
                                                                        .set(
                                                                            Some(AppNotification {
                                                                                level: NotificationLevel::Warning,
                                                                                message: format!("Could not create launch log: {error}"),
                                                                            }),
                                                                        );
                                                                    None
                                                                }
                                                            };
                                                            let launch_environment = match parse_launch_environment(
                                                                vn.launch_environment.clone(),
                                                            ) {
                                                                Ok(environment) => environment,
                                                                Err(error) => {
                                                                    notification
                                                                        .set(
                                                                            Some(AppNotification {
                                                                                level: NotificationLevel::Error,
                                                                                message: format!("Could not launch VN: {error}"),
                                                                            }),
                                                                        );
                                                                    return;
                                                                }
                                                            };

                                                            let game_folder = game_process::game_folder_for(vn.game_folder.as_deref(), &path);

                                                            match launch_executable(
                                                                path,
                                                                vn.launch_mode,
                                                                vn.steam_app_id,
                                                                vn.wine_binary,
                                                                vn.wine_prefix,
                                                                vn.wine_locale,
                                                                vn.proton_path,
                                                                vn.umu_game_id,
                                                                vn.launch_arguments,
                                                                launch_environment,
                                                                launch_log_path.clone(),
                                                                vn.run_with_locale_emulator,
                                                                settings.read().leproc_path.clone(),
                                                            ) {
                                                                Ok(child) => {
                                                                    let started_time = Utc::now();
                                                                    let started_at = started_time.to_rfc3339();
                                                                    let started_timer = Instant::now();
                                                                    let vn_id = presence_vn.id;
                                                                    notification
                                                                        .set(
                                                                            Some(AppNotification {
                                                                                level: NotificationLevel::Info,
                                                                                message: format!("Launched {}.", presence_vn.title),

                                                                            }),
                                                                        );
                                                                    let mut notif_signal = notification;
                                                                    spawn(async move {
                                                                        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                                                                        notif_signal.set(None);
                                                                    });
                                                                    let discord_presence = if settings
                                                                        .read()
                                                                        .discord_rich_presence_enabled
                                                                    {
                                                                        match DiscordPresence::start_for_vn(
                                                                            &presence_vn,
                                                                            started_time,
                                                                            settings.read().clone(),
                                                                        ) {
                                                                            Ok(presence) => Some(presence),
                                                                            Err(error) => {
                                                                                println!("Could not start Discord Rich Presence: {error}");
                                                                                None
                                                                            }
                                                                        }
                                                                    } else {
                                                                        None
                                                                    };
                                                                    let thread_session_tx = session_tx.clone();
                                                                    thread::spawn(move || {
                                                                        let wait_result = game_process::wait_for_game_exit(child, game_folder);
                                                                        if let Some(log_path) = launch_log_path {
                                                                            if let Err(error) = update_latest_launch_log(&log_path) {
                                                                                println!("Could not update latest launch log: {error}");
                                                                            }
                                                                        }
                                                                        match wait_result {
                                                                            Ok(last_seen_running) => {
                                                                                let duration_seconds = last_seen_running
                                                                                    .duration_since(started_timer)
                                                                                    .as_secs();
                                                                                let play_session = PlaySession {
                                                                                    vn_id,
                                                                                    started_at: started_at.clone(),
                                                                                    duration_seconds,
                                                                                    notes: None,
                                                                                };
                                                                                let save_result = add_play_session_to_library(
                                                                                    vn_id,
                                                                                    play_session.clone(),
                                                                                );
                                                                                match save_result {
                                                                                    Ok(()) => {
                                                                                        println!(
                                                                                            "VN {vn_id} closed after {duration_seconds} seconds and was saved.",
                                                                                        );
                                                                                        let _ = thread_session_tx.send((vn_id, play_session));
                                                                                    }
                                                                                    Err(error) => {
                                                                                        println!("Could not save measured play session: {error}");
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(error) => {
                                                                                println!("Could not monitor VN process: {error}");
                                                                            }
                                                                        }
                                                                        if let Some(presence) = discord_presence {
                                                                            if let Err(error) = presence.clear() {
                                                                                println!("Could not clear Discord Rich Presence: {error}");
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                                Err(error) => {
                                                                    notification
                                                                        .set(
                                                                            Some(AppNotification {
                                                                                level: NotificationLevel::Error,
                                                                                message: format!("Could not launch VN: {error}"),
                                                                            }),
                                                                        );
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            notification
                                                                .set(
                                                                    Some(AppNotification {
                                                                        level: NotificationLevel::Warning,
                                                                        message: "No executable path saved for this VN.".to_string(),
                                                                    }),
                                                                );
                                                        }
                                                    }
                                                }
                                                None => {
                                                    notification
                                                        .set(
                                                            Some(AppNotification {
                                                                level: NotificationLevel::Error,
                                                                message: format!("Could not find VN with id {id}."),
                                                            }),
                                                        );
                                                }
                                            }
                                        },

                                        on_run_tool: move |(id, tool_path): (u64, String)| {
                                            let vn = vns
                                                .read()
                                                .iter()
                                                .find(|vn| vn.id == id)
                                                .cloned();

                                            let Some(vn) = vn else {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Error,
                                                            message: "Could not run exe: VN was not found.".to_string(),
                                                        }),
                                                    );

                                                return;
                                            };
                                            if vn.launch_mode == LaunchMode::Proton && !umu_launcher_is_available() {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Error,
                                                            message: "Could not run tool: UMU Launcher was not found. Install umu-launcher to run tools with Proton."
                                                                .to_string(),
                                                        }),
                                                    );
                                                return;
                                            }
                                            let launch_environment = match parse_launch_environment(
                                                vn.launch_environment.clone(),
                                            ) {
                                                Ok(environment) => environment,
                                                Err(error) => {
                                                    notification
                                                        .set(
                                                            Some(AppNotification {
                                                                level: NotificationLevel::Error,
                                                                message: format!("Could not run tool: {error}"),
                                                            }),
                                                        );
                                                    return;
                                                }
                                            };
                                            let launch_log_path = match new_launch_log_path(
                                                vn.id,
                                                format!("{} tool", vn.title),
                                            ) {
                                                Ok(path) => Some(path),
                                                Err(error) => {
                                                    notification
                                                        .set(
                                                            Some(AppNotification {
                                                                level: NotificationLevel::Warning,
                                                                message: format!("Could not create tool log: {error}"),
                                                            }),
                                                        );
                                                    None
                                                }
                                            };
                                            let launch_result = if vn.launch_mode == LaunchMode::Steam {
                                                let Some(app_id) = vn.steam_app_id else {
                                                    notification.set(Some(AppNotification {
                                                        level: NotificationLevel::Error,
                                                        message: "Could not run tool: Steam App ID is not configured.".to_string(),
                                                    }));
                                                    return;
                                                };
                                                launch_steam_tool(
                                                    tool_path,
                                                    app_id,
                                                    launch_environment,
                                                    launch_log_path,
                                                )
                                            } else {
                                                launch_executable(
                                                    tool_path,
                                                    vn.launch_mode,
                                                    vn.steam_app_id,
                                                    vn.wine_binary,
                                                    vn.wine_prefix,
                                                    vn.wine_locale,
                                                    vn.proton_path,
                                                    vn.umu_game_id,
                                                    vn.launch_arguments,
                                                    launch_environment,
                                                    launch_log_path,
                                                    false,
                                                    None,
                                                )
                                                .map(ToolLaunch::Process)
                                            };
                                            match launch_result {
                                                Ok(ToolLaunch::Queued) => {
                                                    notification.set(Some(AppNotification {
                                                        level: NotificationLevel::Info,
                                                        message: format!("Queued tool inside {}'s Steam environment.", vn.title),
                                                    }));
                                                }
                                                Ok(ToolLaunch::Process(mut child)) => {
                                                    notification
                                                        .set(
                                                            Some(AppNotification {
                                                                level: NotificationLevel::Info,
                                                                message: "Tool launched.".to_string(),
                                                            }),
                                                        );
                                                    let mut notif_signal = notification;
                                                    spawn(async move {
                                                        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                                                        notif_signal.set(None);
                                                    });
                                                    thread::spawn(move || {
                                                        if let Err(error) = child.wait() {
                                                            println!("Could not wait for tool process: {error}");
                                                        }
                                                    });
                                                }
                                                Err(error) => {
                                                    notification
                                                        .set(
                                                            Some(AppNotification {
                                                                level: NotificationLevel::Error,
                                                                message: format!("Could not launch tool: {error}"),
                                                            }),
                                                        );
                                                }
                                            }
                                        },

                                        on_launch_mode_change: move |(id, launch_mode)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.launch_mode = launch_mode;
                                                },
                                                "Could not save launch mode".to_string(),
                                            );
                                        },

                                        on_steam_app_id_change: move |(id, steam_app_id): (u64, String)| {
                                            let steam_app_id = steam_app_id.parse::<u32>().ok();

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.steam_app_id = steam_app_id;
                                                },
                                                "Could not save Steam App ID".to_string(),
                                            );
                                        },

                                        on_wine_binary_change: move |(id, wine_binary): (u64, String)| {
                                            let wine_binary = if wine_binary.trim().is_empty() {
                                                None
                                            } else {
                                                Some(wine_binary)
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.wine_binary = wine_binary;
                                                },
                                                "Could not save Wine binary".to_string(),
                                            );
                                        },

                                        on_proton_path_change: move |(id, proton_path): (u64, String)| {
                                            let proton_path = if proton_path.trim().is_empty() {
                                                None
                                            } else {
                                                Some(proton_path)
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.proton_path = proton_path;
                                                },
                                                "Could not save Proton path".to_string(),
                                            );
                                        },

                                        on_umu_game_id_change: move |(id, umu_game_id): (u64, String)| {
                                            let umu_game_id = if umu_game_id.trim().is_empty() {
                                                default_umu_game_id()
                                            } else {
                                                umu_game_id
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.umu_game_id = umu_game_id;
                                                },
                                                "Could not save UMU game ID".to_string(),
                                            );
                                        },

                                        on_delete: move |id| {
                                            vns.write().retain(|vn| vn.id != id);

                                            if selected_vn_id_value == Some(id) {
                                                selected_vn_id.set(None);
                                            }

                                            save_vns_or_log(&vns, "Could not delete VN".to_string());
                                        },

                                        on_wine_prefix_change: move |(id, wine_prefix): (u64, String)| {
                                            let wine_prefix = if wine_prefix.trim().is_empty() {
                                                None
                                            } else {
                                                Some(wine_prefix)
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.wine_prefix = wine_prefix;
                                                },
                                                "Could not save Wine prefix".to_string(),
                                            );
                                        },

                                        on_wine_locale_change: move |(id, wine_locale): (u64, String)| {
                                            let wine_locale = if wine_locale.trim().is_empty() {
                                                None
                                            } else {
                                                Some(wine_locale)
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.wine_locale = wine_locale;
                                                },
                                                "Could not save Wine locale".to_string(),
                                            );
                                        },

                                        on_launch_arguments_change: move |(id, launch_arguments): (u64, String)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.launch_arguments = launch_arguments;
                                                },
                                                "Could not save launch arguments".to_string(),
                                            );
                                        },

                                        on_launch_environment_change: move |(id, launch_environment): (u64, String)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.launch_environment = launch_environment;
                                                },
                                                "Could not save launch environment".to_string(),
                                            );
                                        },

                                        on_description_change: move |(id, description): (u64, String)| {
                                            let description = if description.trim().is_empty() {
                                                None
                                            } else {
                                                Some(description)
                                            };

                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.description = description;
                                                },
                                                "Could not save description".to_string(),
                                            );
                                        },

                                        on_tags_change: move |(id, tags)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.tags = tags;
                                                },
                                                "Could not save tags".to_string(),
                                            );
                                        },

                                        on_cover_path_change: move |(id, cover_path): (u64, String)| {
                                            let trimmed_cover_path = cover_path.trim().to_string();

                                            if is_flatpak_document_portal_path(&trimmed_cover_path) {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Error,
                                                            message: "The cover image has been selected through Flatpak's temporary document portal. Grant Yakata access to the folder, then reselect the real path."
                                                                .to_string(),
                                                        }),
                                                    );
                                                return;
                                            }
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.cover_path = Some(trimmed_cover_path);
                                                },
                                                "Could not save cover image".to_string(),
                                            );
                                        },

                                        on_cover_refresh: move |id| {
                                            let vn = vns
                                                .read()
                                                .iter()
                                                .find(|vn| vn.id == id)
                                                .cloned();

                                            let Some(vn) = vn else {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Error,
                                                            message: "Could not refresh cover: VN was not found.".to_string(),
                                                        }),
                                                    );
                                                return;
                                            };
                                            let Some(cover_url) = vn.cover_url.clone() else {
                                                notification
                                                    .set(
                                                        Some(AppNotification {
                                                            level: NotificationLevel::Warning,
                                                            message: "This VN does not have a VNDB cover URL saved."
                                                                .to_string(),
                                                        }),
                                                    );
                                                return;
                                            };
                                            let mut vns_for_cover = vns;
                                            let mut notification_for_cover = notification;
                                            spawn(async move {
                                                match cache_cover_image(id, cover_url).await {
                                                    Ok(cover_path) => {
                                                        for vn in vns_for_cover.write().iter_mut() {
                                                            if vn.id == id {
                                                                vn.cover_path = Some(cover_path.clone());
                                                                break;
                                                            }
                                                        }
                                                        save_vns_or_log(
                                                            &vns_for_cover,
                                                            "Could not save refreshed cover path.".to_string(),
                                                        );
                                                        notification_for_cover
                                                            .set(
                                                                Some(AppNotification {
                                                                    level: NotificationLevel::Info,
                                                                    message: "Cover refreshed from VNDB".to_string(),
                                                                }),
                                                            );
                                                    }
                                                    Err(error) => {
                                                        notification_for_cover
                                                            .set(
                                                                Some(AppNotification {
                                                                    level: NotificationLevel::Error,
                                                                    message: format!("Could not refresh cover: {error}"),
                                                                }),
                                                            );
                                                    }
                                                }
                                            });
                                        },

                                        on_active_route_change: move |(id, active_route): (u64, Option<String>)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.active_route = active_route;
                                                },
                                                "Could not save active route".to_string(),
                                            );
                                        },

                                        on_route_delete: move |(id, route_name): (u64, String)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.routes.retain(|route| route.name != route_name);

                                                    if vn.active_route == Some(route_name) {
                                                        vn.active_route = None;
                                                    }
                                                },

                                                "Could not delete route".to_string(),
                                            );
                                        },

                                        on_status_change: move |(id, status)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.status = status;
                                                },
                                                "Could not save status".to_string(),
                                            );
                                        },

                                        on_rating_change: move |(id, rating)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.rating = rating;
                                                },
                                                "Could not save rating".to_string(),
                                            );
                                        },

                                        on_run_with_locale_emulator_change: move |(id, enabled)| {
                                            update_vn_and_save(
                                                &mut vns,
                                                id,
                                                move |vn| {
                                                    vn.run_with_locale_emulator = enabled;
                                                },
                                                "Could not save locale emulator setting".to_string(),
                                            );
                                        },
                                    }
                                } else {
                                    section { class: "empty-detail-panel",

                                        h2 { "No VN selected" }
                                        p {
                                            "Choose a visual novel from the library to view notes, routes, launch settings, and play sessions."
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        SettingsView {
                            discord_rich_presence_enabled: settings.read().discord_rich_presence_enabled,
                            discord_status_text: settings.read().discord_status_text.clone(),
                            discord_show_active_route: settings.read().discord_show_active_route,
                            discord_custom_cover_url: settings.read().discord_custom_cover_url.clone(),
                            data_dir_text,
                            custom_background_path: settings.read().custom_background_path.clone(),
                            leproc_path: settings.read().leproc_path.clone().or_else(launcher::detect_leproc_path),
                            blur_nsfw: settings.read().blur_nsfw,

                            on_discord_rich_presence_change: move |enabled| {
                                settings.write().discord_rich_presence_enabled = enabled;
                                save_settings_or_log(&settings);
                            },

                            on_discord_status_text_change: move |status_text| {
                                settings.write().discord_status_text = status_text;
                                save_settings_or_log(&settings);
                            },

                            on_discord_show_active_route_change: move |enabled| {
                                settings.write().discord_show_active_route = enabled;
                                save_settings_or_log(&settings);
                            },

                            on_discord_custom_cover_url_change: move |cover_url| {
                                settings.write().discord_custom_cover_url = cover_url;
                                save_settings_or_log(&settings);
                            },

                            on_custom_background_path_change: move |path| {
                                settings.write().custom_background_path = path;
                                save_settings_or_log(&settings);
                            },

                            on_leproc_path_change: move |path| {
                                settings.write().leproc_path = path;
                                save_settings_or_log(&settings);
                            },

                            on_blur_nsfw_change: move |enabled| {
                                settings.write().blur_nsfw = enabled;
                                save_settings_or_log(&settings);
                            },

                            on_open_data_folder: move |_| {
                                match yakata_data_dir() {
                                    Ok(path) => {
                                        if let Err(error) = open_folder(&path) {
                                            println!("Could not open data folder: {error}");
                                        }
                                    }
                                    Err(error) => {
                                        println!("Could not find data folder: {error}");
                                    }
                                }
                            },

                            on_open_logs_folder: move |_| {
                                match launch_logs_dir() {
                                    Ok(path) => {
                                        if let Err(error) = open_folder(&path) {
                                            println!("Could not open logs folder: {error}");
                                            notification
                                                .set(
                                                    Some(AppNotification {
                                                        level: NotificationLevel::Error,
                                                        message: format!("Could not open logs folder: {error}"),
                                                    }),
                                                );
                                        }
                                    }
                                    Err(error) => {
                                        println!("Could not find logs folder: {error}");
                                        notification
                                            .set(
                                                Some(AppNotification {
                                                    level: NotificationLevel::Error,
                                                    message: format!("Could not find logs folder: {error}"),
                                                }),
                                            );
                                    }
                                }
                            },

                            on_export_library: move |_| {
                                let picked_file = rfd::FileDialog::new()
                                    .add_filter("JSON", &["json"])
                                    .set_file_name("yakata_library_backup.json")
                                    .save_file();
                                if let Some(path) = picked_file {
                                    match storage::export_library_backup(&path) {
                                        Ok(()) => {
                                            notification.set(Some(AppNotification {
                                                level: NotificationLevel::Info,
                                                message: "Library backup exported successfully.".to_string(),
                                            }));
                                        }
                                        Err(error) => {
                                            notification.set(Some(AppNotification {
                                                level: NotificationLevel::Error,
                                                message: format!("Could not export library backup: {error}"),
                                            }));
                                        }
                                    }
                                }
                            },

                            on_import_library: move |_| {
                                let picked_file = rfd::FileDialog::new()
                                    .add_filter("JSON", &["json"])
                                    .pick_file();
                                if let Some(path) = picked_file {
                                    match storage::import_library_backup(&path) {
                                        Ok(imported) => {
                                            *vns.write() = imported;
                                            notification.set(Some(AppNotification {
                                                level: NotificationLevel::Info,
                                                message: "Library backup restored successfully.".to_string(),
                                            }));
                                        }
                                        Err(error) => {
                                            notification.set(Some(AppNotification {
                                                level: NotificationLevel::Error,
                                                message: format!("Could not import library backup: {error}"),
                                            }));
                                        }
                                    }
                                }
                            },
                        }
                    }

                }

            }

        }
    }
}

fn update_vn_and_save(
    vns: &mut Signal<Vec<VisualNovel>>,
    id: u64,
    update: impl FnOnce(&mut VisualNovel),
    error_message: String,
) {
    for vn in vns.write().iter_mut() {
        if vn.id == id {
            update(vn);
            break;
        }
    }

    save_vns_or_log(vns, error_message);
}

fn save_vns_or_log(vns: &Signal<Vec<VisualNovel>>, error_message: String) {
    let save_result = save_library(vns.read().clone());

    if let Err(error) = save_result {
        println!("{error_message}: {error}");
    }
}

fn save_settings_or_log(settings: &Signal<AppSettings>) {
    let save_result = save_settings(settings.read().clone());

    if let Err(error) = save_result {
        println!("Could not save settings: {error}");
    }
}

fn latest_played_at(vn: &VisualNovel) -> String {
    vn.play_sessions
        .iter()
        .map(|session| session.started_at.clone())
        .max()
        .unwrap_or_default()
}

fn total_playtime_seconds(vn: &VisualNovel) -> u64 {
    vn.play_sessions
        .iter()
        .map(|session| session.duration_seconds)
        .sum()
}
