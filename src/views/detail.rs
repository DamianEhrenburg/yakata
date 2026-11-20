use crate::models::{LaunchMode, NovelStatus, StoryRoute, VisualNovel};
use crate::views::dropdown::{Dropdown, DropdownOption};
use crate::views::library::{cover_source, is_vn_nsfw};
use crate::vn_markup::{DescriptionPart, parse_description};
use crate::wine::{ProtonRunner, SteamPrefix, WineRunner};
use dioxus::prelude::*;
use rfd::FileDialog;

#[derive(Debug, Clone, PartialEq)]
enum DetailTab {
    Info,
    Launch,
    Routes,
    History,
}

#[component]
pub fn DetailView(
    vn: VisualNovel,
    blur_nsfw: bool,
    wine_runners: Vec<WineRunner>,
    proton_runners: Vec<ProtonRunner>,
    steam_prefixes: Vec<SteamPrefix>,
    steam_launch_option: String,
    on_notes_change: EventHandler<(u64, String)>,
    on_route_add: EventHandler<(u64, String)>,
    on_route_toggle: EventHandler<(u64, String)>,
    on_executable_path_change: EventHandler<(u64, String)>,
    on_launch: EventHandler<u64>,
    on_run_tool: EventHandler<(u64, String)>,
    on_launch_mode_change: EventHandler<(u64, LaunchMode)>,
    on_steam_app_id_change: EventHandler<(u64, String)>,
    on_wine_binary_change: EventHandler<(u64, String)>,
    on_proton_path_change: EventHandler<(u64, String)>,
    on_umu_game_id_change: EventHandler<(u64, String)>,
    on_delete: EventHandler<u64>,
    on_wine_prefix_change: EventHandler<(u64, String)>,
    on_wine_locale_change: EventHandler<(u64, String)>,
    on_launch_arguments_change: EventHandler<(u64, String)>,
    on_launch_environment_change: EventHandler<(u64, String)>,
    on_description_change: EventHandler<(u64, String)>,
    on_tags_change: EventHandler<(u64, Vec<String>)>,
    on_cover_path_change: EventHandler<(u64, String)>,
    on_cover_refresh: EventHandler<u64>,
    on_active_route_change: EventHandler<(u64, Option<String>)>,
    on_route_delete: EventHandler<(u64, String)>,
    on_status_change: EventHandler<(u64, Option<NovelStatus>)>,
    on_rating_change: EventHandler<(u64, Option<u8>)>,
    on_run_with_locale_emulator_change: EventHandler<(u64, bool)>,
    on_add_play_session: EventHandler<(u64, u64, Option<String>)>,
    on_delete_play_session: EventHandler<(u64, usize)>,
) -> Element {
    let mut new_route_name = use_signal(String::new);
    let typed_route_name = new_route_name.read().clone();
    let mut route_error_message = use_signal(|| None::<String>);
    let route_error_text = route_error_message.read().clone();
    let executable_path_text = vn.executable_path.clone().unwrap_or_default();
    let mut executable_path_draft = use_signal(|| executable_path_text.clone());
    let wine_prefix_text = vn.wine_prefix.clone().unwrap_or_default();
    let wine_locale_text = vn.wine_locale.clone().unwrap_or_default();
    let show_compatibility_settings = cfg!(target_os = "linux")
        && (vn.launch_mode == LaunchMode::Wine
            || vn.launch_mode == LaunchMode::Proton
            || vn.launch_mode == LaunchMode::Steam);

    let mut wine_prefix_draft = use_signal(|| wine_prefix_text.clone());
    let mut wine_locale_draft = use_signal(|| wine_locale_text.clone());
    let mut launch_arguments_draft = use_signal(|| vn.launch_arguments.clone());
    let mut launch_environment_draft = use_signal(|| vn.launch_environment.clone());
    let wine_binary_text = vn.wine_binary.clone().unwrap_or_default();
    let mut wine_binary_draft = use_signal(|| wine_binary_text.clone());
    let wine_binary_value = wine_binary_draft.read().clone();
    let proton_path_text = vn.proton_path.clone().unwrap_or_default();
    let mut proton_path_draft = use_signal(|| proton_path_text.clone());
    let proton_path_value = proton_path_draft.read().clone();
    let mut umu_game_id_draft = use_signal(|| vn.umu_game_id.clone());
    let umu_game_id_value = umu_game_id_draft.read().clone();
    let steam_app_id_text = vn
        .steam_app_id
        .map(|app_id| app_id.to_string())
        .unwrap_or_default();

    let mut steam_app_id_draft = use_signal(|| steam_app_id_text);
    let steam_app_id_value = steam_app_id_draft.read().clone();

    let executable_path_value = executable_path_draft.read().clone();
    let wine_prefix_value = wine_prefix_draft.read().clone();
    let wine_locale_value = wine_locale_draft.read().clone();
    let launch_arguments_value = launch_arguments_draft.read().clone();
    let launch_environment_value = launch_environment_draft.read().clone();
    let mut notes_draft = use_signal(|| vn.notes.clone());
    let notes_value = notes_draft.read().clone();
    let saved_notes = vn.notes.clone();
    let mut notes_is_editing = use_signal(|| false);
    let mut notes_cancelled = use_signal(|| false);
    let mut description_draft = use_signal(|| vn.description.clone().unwrap_or_default());
    let mut description_is_editing = use_signal(|| false);
    let mut description_cancelled = use_signal(|| false);
    let saved_description = vn.description.clone().unwrap_or_default();
    let description_value = description_draft.read().clone();
    let description_parts = parse_description(saved_description.clone());
    let mut new_tag_text = use_signal(String::new);
    let new_tag_value = new_tag_text.read().clone();
    let mut delete_is_confirming = use_signal(|| false);
    let mut tag_editor_is_open = use_signal(|| false);
    let tag_editor_open = *tag_editor_is_open.read();
    let mut active_tab = use_signal(|| DetailTab::Info);
    let selected_tab = active_tab.read().clone();
    let mut launch_settings_are_open = use_signal(|| false);
    let launch_settings_open = *launch_settings_are_open.read();
    let mut add_session_open = use_signal(|| false);
    let mut add_session_hours = use_signal(|| "0".to_string());
    let mut add_session_minutes = use_signal(|| "30".to_string());
    let mut add_session_note = use_signal(String::new);
    let total_playtime_seconds: u64 = vn
        .play_sessions
        .iter()
        .map(|session| session.duration_seconds)
        .sum();
    let total_playtime_text = format_playtime(total_playtime_seconds);
    let saved_desc_for_mouse = saved_description.clone();
    let saved_desc_for_click = saved_description.clone();
    let saved_notes_for_mouse = saved_notes.clone();
    let saved_notes_for_click = saved_notes.clone();
    let existing_routes_for_key = vn.routes.clone();
    let existing_routes_for_click = vn.routes.clone();
    rsx! {
        section { class: "detail-panel",

            div { class: "detail-header",
                h2 { "{vn.title}" }
                div { class: "detail-cover-frame",
                    if let Some(cover_src) = cover_source(vn.clone()) {
                        {
                            let cover_class = if blur_nsfw && is_vn_nsfw(&vn) {
                                "detail-cover nsfw-blur"
                            } else {
                                "detail-cover"
                            };
                            rsx! {
                                img {
                                    class: "{cover_class}",
                                    src: "{cover_src}",
                                    alt: "Cover art for {vn.title}",
                                }
                            }
                        }
                    } else {
                        div { class: "detail-cover-placeholder", "No cover" }
                    }
                    button {
                        class: "cover-action-button cover-edit-button",
                        title: "Change cover image",
                        onclick: move |_| {
                            let picked_file = FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                .pick_file();
                            if let Some(path) = picked_file {
                                on_cover_path_change.call((vn.id, path.to_string_lossy().to_string()));
                            }
                        },
                        "✎"
                    }
                    if vn.cover_url.is_some() {
                        button {
                            class: "cover-action-button cover-refresh-button",
                            title: "Refresh cover from VNDB",
                            onclick: move |_| {
                                on_cover_refresh.call(vn.id);
                            },
                            "↻"
                        }
                    }
                }

                div { class: "detail-tabs",
                    button {
                        class: if selected_tab == DetailTab::Info { "detail-tab active" } else { "detail-tab" },
                        onclick: move |_| {
                            active_tab.set(DetailTab::Info);
                        },
                        "Info"
                    }
                    button {
                        class: if selected_tab == DetailTab::Launch { "detail-tab active" } else { "detail-tab" },
                        onclick: move |_| {
                            active_tab.set(DetailTab::Launch);
                        },
                        "Launch"
                    }
                    button {
                        class: if selected_tab == DetailTab::Routes { "detail-tab active" } else { "detail-tab" },
                        onclick: move |_| {
                            active_tab.set(DetailTab::Routes);
                        },
                        "Routes"
                    }
                    button {
                        class: if selected_tab == DetailTab::History { "detail-tab active" } else { "detail-tab" },
                        onclick: move |_| {
                            active_tab.set(DetailTab::History);
                        },
                        "History"
                    }
                }

            }
            div { class: "detail-tab-content",
                if selected_tab == DetailTab::Info {
                    div { class: "detail-tracker-row",
                        div { class: "detail-tracker-field",
                            label { "Status" }
                            Dropdown {
                                options: {
                                    let mut opts = vec![DropdownOption::new("none", "No status")];
                                    opts.extend(NovelStatus::all().iter().map(|s| DropdownOption::new(s.key(), s.as_str())));
                                    opts
                                },
                                selected_value: vn.status.map(|s| s.key()).unwrap_or("none").to_string(),
                                on_select: move |val: String| {
                                    on_status_change.call((vn.id, NovelStatus::from_key(&val)));
                                },
                            }
                        }
                        div { class: "detail-tracker-field",
                            label { "Score" }
                            Dropdown {
                                options: vec![
                                    DropdownOption::new("none", "No score"),
                                    DropdownOption::with_sublabel("10", "★ 10", "Masterpiece"),
                                    DropdownOption::with_sublabel("9", "★ 9", "Great"),
                                    DropdownOption::with_sublabel("8", "★ 8", "Very Good"),
                                    DropdownOption::with_sublabel("7", "★ 7", "Good"),
                                    DropdownOption::with_sublabel("6", "★ 6", "Fine"),
                                    DropdownOption::with_sublabel("5", "★ 5", "Average"),
                                    DropdownOption::with_sublabel("4", "★ 4", "Bad"),
                                    DropdownOption::with_sublabel("3", "★ 3", "Very Bad"),
                                    DropdownOption::with_sublabel("2", "★ 2", "Horrible"),
                                    DropdownOption::with_sublabel("1", "★ 1", "Appalling"),
                                ],
                                selected_value: vn.rating.map(|r| r.to_string()).unwrap_or_else(|| "none".to_string()),
                                on_select: move |val: String| {
                                    let rating = if val == "none" {
                                        None
                                    } else {
                                        val.parse::<u8>().ok()
                                    };
                                    on_rating_change.call((vn.id, rating));
                                },
                            }
                        }
                    }
                    {
                        let vndb_url = if let Some(ref vid) = vn.vndb_id {
                            format!("https://vndb.org/{vid}")
                        } else {
                            let clean_title = crate::system::percent_encode(&vn.title);
                            format!("https://vndb.org/v/all?sq={clean_title}")
                        };
                        rsx! {
                            button {
                                class: "vndb-link-button",
                                onclick: move |_| {
                                    let _ = crate::system::open_url(&vndb_url);
                                },
                                "Open on VNDB"
                            }
                        }
                    }
                    h3 { "Tags" }
                    div { class: "tag-editor",
                        div { class: "tag-chip-list",
                            if vn.tags.is_empty() {
                                span { class: "tag-editor-empty", "No tags yet." }
                            }
                            for tag in vn.tags.iter() {
                                {
                                    let tag_to_remove = tag.clone();
                                    let vn_id = vn.id;
                                    let current_tags = vn.tags.clone();
                                    let on_tags_change_for_remove = on_tags_change;
                                    rsx! {
                                        button {
                                            class: "tag-chip editable",
                                            title: "Remove tag",
                                            onclick: move |_| {
                                                let mut next_tags = current_tags.clone();
                                                next_tags.retain(|tag| tag != &tag_to_remove);
                                                on_tags_change_for_remove.call((vn_id, next_tags));
                                            },
                                            span { class: "tag-chip-label", "{tag}" }
                                            span { class: "tag-chip-remove", "✕" }
                                        }
                                    }
                                }
                            }
                            if tag_editor_open {
                                {
                                    let tags_for_keydown = vn.tags.clone();
                                    let vn_id = vn.id;
                                    rsx! {
                                        input {
                                            class: "tag-editor-input inline",
                                            value: "{new_tag_value}",
                                            placeholder: "Tag name",
                                            maxlength: "30",
                                            autofocus: true,

                                            oninput: move |event| {
                                                let val = event.value();
                                                if val.chars().count() <= 30 {
                                                    new_tag_text.set(val);
                                                }
                                            },

                                            onkeydown: move |event| {
                                                let pressed_key = event.key().to_string();

                                                if pressed_key == "Enter" {
                                                    let typed_tag = new_tag_text.read().trim().to_string();

                                                    if typed_tag.is_empty() || typed_tag.chars().count() > 30 {
                                                        return;
                                                    }

                                                    let mut next_tags = tags_for_keydown.clone();
                                                    let already_exists = next_tags
                                                        .iter()
                                                        .any(|tag| tag.eq_ignore_ascii_case(&typed_tag));

                                                    if !already_exists {
                                                        next_tags.push(typed_tag);
                                                        on_tags_change.call((vn_id, next_tags));
                                                    }

                                                    new_tag_text.set(String::new());
                                                    tag_editor_is_open.set(false);
                                                } else if pressed_key == "Escape" {
                                                    new_tag_text.set(String::new());
                                                    tag_editor_is_open.set(false);
                                                }
                                            },

                                            onblur: move |_| {
                                                new_tag_text.set(String::new());
                                                tag_editor_is_open.set(false);
                                            },
                                        }
                                    }
                                }
                            } else {
                                button {
                                    class: "tag-add-button",
                                    title: "Add tag",
                                    onclick: move |_| {
                                        tag_editor_is_open.set(true);
                                    },
                                    "+"
                                }
                            }
                        }
                    }
                    div { class: "detail-section-header",
                        h3 { "Description" }
                        button {
                            class: "section-action-btn",
                            title: if *description_is_editing.read() { "Close editor" } else { "Edit description" },
                            onmousedown: move |event: MouseEvent| {
                                if *description_is_editing.read() {
                                    event.prevent_default();
                                    description_cancelled.set(true);
                                    description_draft.set(saved_desc_for_mouse.clone());
                                    description_is_editing.set(false);
                                }
                            },
                            onclick: move |_| {
                                if *description_cancelled.read() {
                                    description_cancelled.set(false);
                                    return;
                                }
                                let next_value = !*description_is_editing.read();
                                if next_value {
                                    description_draft.set(saved_desc_for_click.clone());
                                }
                                description_is_editing.set(next_value);
                            },
                            if *description_is_editing.read() { "✕" } else { "✎" }
                        }
                    }
                    div { class: if *description_is_editing.read() { "desc-box editing" } else { "desc-box" },
                        if *description_is_editing.read() {
                            textarea {
                                class: "detail-desc-input",
                                value: "{description_value}",
                                oninput: move |event| {
                                    description_draft.set(event.value());
                                },
                                onblur: move |_| {
                                    if *description_is_editing.read() && !*description_cancelled.read() {
                                        on_description_change.call((vn.id, description_draft.read().clone()));
                                        description_is_editing.set(false);
                                    }
                                },
                            }
                        } else if saved_description.is_empty() {
                            div { class: "detail-desc empty", "No description yet." }
                        } else {
                            div { class: "detail-desc",
                                for part in description_parts {
                                    DescriptionPartView { part }
                                }
                            }
                        }
                    }

                    div { class: "detail-section-header",
                        h3 { "Notes" }
                        button {
                            class: "section-action-btn",
                            title: if *notes_is_editing.read() { "Close editor" } else { "Edit notes" },
                            onmousedown: move |event: MouseEvent| {
                                if *notes_is_editing.read() {
                                    event.prevent_default();
                                    notes_cancelled.set(true);
                                    notes_draft.set(saved_notes_for_mouse.clone());
                                    notes_is_editing.set(false);
                                }
                            },
                            onclick: move |_| {
                                if *notes_cancelled.read() {
                                    notes_cancelled.set(false);
                                    return;
                                }
                                let next_value = !*notes_is_editing.read();
                                if next_value {
                                    notes_draft.set(saved_notes_for_click.clone());
                                }
                                notes_is_editing.set(next_value);
                            },
                            if *notes_is_editing.read() { "✕" } else { "✎" }
                        }
                    }

                    div { class: if *notes_is_editing.read() { "notes-box editing" } else { "notes-box" },
                        if *notes_is_editing.read() {
                            textarea {
                                class: "notes-input",
                                value: "{notes_value}",
                                oninput: move |event| {
                                    notes_draft.set(event.value());
                                },
                                onblur: move |_| {
                                    if *notes_is_editing.read() && !*notes_cancelled.read() {
                                        on_notes_change.call((vn.id, notes_draft.read().clone()));
                                        notes_is_editing.set(false);
                                    }
                                },
                            }
                        } else if saved_notes.is_empty() {
                            p { class: "notes-text empty", "No notes yet." }
                        } else {
                            p { class: "notes-text", "{saved_notes}" }
                        }
                    }
                    if *delete_is_confirming.read() {
                        div { class: "delete-confirm-row",
                            button {
                                class: "delete-button confirming",
                                onclick: move |_| {
                                    delete_is_confirming.set(false);
                                    on_delete.call(vn.id);
                                },
                                "Confirm delete"
                            }
                            button {
                                class: "delete-cancel-button",
                                onclick: move |_| {
                                    delete_is_confirming.set(false);
                                },
                                "Cancel"
                            }
                        }
                    } else {
                        button {
                            class: "delete-button",
                            onclick: move |_| {
                                delete_is_confirming.set(true);
                            },
                            "Delete VN"
                        }
                    }
                }
                if selected_tab == DetailTab::Launch {
                    h3 { "Launch" }
                    label {
                        "Executable path"
                        div { class: "add-vn-input-browse-row",
                            input {
                                value: "{executable_path_value}",
                                placeholder: "Optional: path to game .exe",
                                oninput: move |event| {
                                    executable_path_draft.set(event.value());
                                },
                                onblur: move |_| {
                                    on_executable_path_change.call((vn.id, executable_path_draft.read().clone()));
                                },
                            }
                            button {
                                class: "add-vn-browse-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let picked_file = FileDialog::new()
                                        .add_filter("Executables", &["exe", "bin", "sh", "AppImage"])
                                        .add_filter("All Files", &["*"])
                                        .pick_file();
                                    if let Some(path) = picked_file {
                                        let path_text = path.to_string_lossy().to_string();
                                        executable_path_draft.set(path_text.clone());
                                        on_executable_path_change.call((vn.id, path_text));
                                    }
                                },
                                "Browse"
                            }
                        }
                    }
                    if cfg!(target_os = "windows") && vn.launch_mode != LaunchMode::Steam {
                        label { class: "checkbox-row",
                            input {
                                class: "setting-checkbox",
                                r#type: "checkbox",
                                checked: vn.run_with_locale_emulator,
                                onchange: move |event| {
                                    on_run_with_locale_emulator_change.call((vn.id, event.checked()));
                                },
                            }
                            span { "Run with Locale Emulator" }
                        }
                    }
                    div { class: "launch-selector",
                        label { "Launch mode" }
                        {
                            let mut launch_opts = vec![DropdownOption::new("native", "Native")];
                            if cfg!(target_os = "linux") {
                                launch_opts.push(DropdownOption::new("wine", "Wine"));
                                launch_opts.push(DropdownOption::new("proton", "Proton"));
                            }
                            launch_opts.push(DropdownOption::new("steam", "Steam"));
                            let cur_val = match vn.launch_mode {
                                LaunchMode::Native => "native",
                                LaunchMode::Wine => "wine",
                                LaunchMode::Proton => "proton",
                                LaunchMode::Steam => "steam",
                            }.to_string();
                            rsx! {
                                Dropdown {
                                    class: "drop-up".to_string(),
                                    options: launch_opts,
                                    selected_value: cur_val,
                                    on_select: move |val: String| {
                                        let launch_mode = match val.as_str() {
                                            "wine" => LaunchMode::Wine,
                                            "proton" => LaunchMode::Proton,
                                            "steam" => LaunchMode::Steam,
                                            _ => LaunchMode::Native,
                                        };
                                        on_launch_mode_change.call((vn.id, launch_mode));
                                    },
                                }
                            }
                        }
                    }
                    if vn.launch_mode == LaunchMode::Steam {
                        label {
                            "Steam App ID"
                            input {
                                r#type: "text",
                                placeholder: "e.g. 1989270 or Steam URL",
                                value: "{steam_app_id_value}",
                                oninput: move |event| {
                                    steam_app_id_draft.set(event.value());
                                },
                                onblur: move |_| {
                                    let parsed = crate::launcher::parse_steam_app_id(&steam_app_id_draft.read());
                                    let normalized = parsed.map(|id| id.to_string()).unwrap_or_default();
                                    steam_app_id_draft.set(normalized.clone());
                                    on_steam_app_id_change.call((
                                        vn.id,
                                        normalized,
                                    ));
                                },
                            }
                        }
                    }
                    if show_compatibility_settings {
                        button {
                            class: "launch-settings-toggle",
                            onclick: move |_| {
                                launch_settings_are_open.set(!launch_settings_open);
                            },
                            if launch_settings_open {
                                "Hide settings"
                            } else {
                                "Show settings"
                            }
                        }
                        if launch_settings_open {
                            div { class: "wine-settings",
                                h3 { "Wine Settings" }
                                if vn.launch_mode == LaunchMode::Wine {
                                    label {
                                        "Wine binary / runner"
                                        select {
                                            onchange: move |event| {
                                                let binary_path = event.value();
                                                wine_binary_draft.set(binary_path.clone());
                                                on_wine_binary_change.call((vn.id, binary_path));
                                            },
                                            option {
                                                value: "",
                                                selected: wine_binary_value.is_empty(),
                                                "Default Wine from PATH"
                                            }
                                            for runner in wine_runners.clone() {
                                                option {
                                                    value: "{runner.binary_path}",
                                                    selected: runner.binary_path == wine_binary_value,
                                                    "{runner.name}"
                                                }
                                            }
                                            if !wine_binary_value.is_empty()
                                                && !wine_runners.iter().any(|runner| runner.binary_path == wine_binary_value)
                                            {
                                                option {
                                                    value: "{wine_binary_value}",
                                                    selected: true,
                                                    "Custom runner"
                                                }
                                            }
                                        }
                                        button {
                                            class: "fp-button",
                                            onclick: move |_| {
                                                let picked_file = FileDialog::new().pick_file();
                                                if let Some(path) = picked_file {
                                                    let path_text = path.to_string_lossy().to_string();
                                                    wine_binary_draft.set(path_text.clone());
                                                    on_wine_binary_change.call((vn.id, path_text));
                                                }
                                            },
                                            "Choose Wine binary"
                                        }
                                    }
                                }
                                if vn.launch_mode == LaunchMode::Proton {
                                    label {
                                        "Proton version"
                                        select {
                                            value: "{proton_path_value}",
                                            onchange: move |event| {
                                                let proton_path = event.value();
                                                proton_path_draft.set(proton_path.clone());
                                                on_proton_path_change.call((vn.id, proton_path));
                                            },
                                            option {
                                                value: "",
                                                selected: proton_path_value.is_empty(),
                                                "UMU managed default"
                                            }
                                            for runner in proton_runners.clone() {
                                                option {
                                                    value: "{runner.path}",
                                                    selected: runner.path == proton_path_value,
                                                    "{runner.name}"
                                                }
                                            }
                                            if !proton_path_value.is_empty()
                                                && !proton_runners.iter().any(|runner| runner.path == proton_path_value)
                                            {
                                                option {
                                                    value: "{proton_path_value}",
                                                    selected: true,
                                                    "Custom Proton install"
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "fp-button",
                                        onclick: move |_| {
                                            let picked_folder = FileDialog::new().pick_folder();
                                            if let Some(path) = picked_folder {
                                                let path_text = path.to_string_lossy().to_string();
                                                proton_path_draft.set(path_text.clone());
                                                on_proton_path_change.call((vn.id, path_text));
                                            }
                                        },
                                        "Choose Proton folder"
                                    }
                                    label {
                                        "UMU game ID"
                                        input {
                                            placeholder: "umu-default",
                                            value: "{umu_game_id_value}",
                                            oninput: move |event| {
                                                umu_game_id_draft.set(event.value());
                                            },
                                            onblur: move |_| {
                                                on_umu_game_id_change.call((vn.id, umu_game_id_draft.read().clone()));
                                            },
                                        }
                                    }
                                }
                                if vn.launch_mode == LaunchMode::Steam {
                                    label {
                                        "Steam launch option"

                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            value: "{steam_launch_option}",
                                            onclick: move |event| {
                                                event.stop_propagation();
                                            },
                                        }
                                    }
                                }
                                label {
                                    "Detected Steam prefix"
                                    select {
                                        value: "{wine_prefix_value}",
                                        onchange: move |event| {
                                            let prefix_path = event.value();
                                            wine_prefix_draft.set(prefix_path.clone());
                                            on_wine_prefix_change.call((vn.id, prefix_path));
                                        },
                                        option { value: "", "No Steam prefix selected" }
                                        for prefix in steam_prefixes.clone() {
                                            option { value: "{prefix.path}",
                                                match prefix.game_name {
                                                    Some(name) => format!("{name} - Steam {}", prefix.app_id),
                                                    None => format!("Steam {}", prefix.app_id),
                                                }
                                            }
                                        }
                                        if !wine_prefix_value.is_empty()
                                            && !steam_prefixes.iter().any(|prefix| prefix.path == wine_prefix_value)
                                        {
                                            option { value: "{wine_prefix_value}", "Custom prefix" }
                                        }
                                    }
                                }
                                label {
                                    "Wine prefix"
                                    input {
                                        value: "{wine_prefix_value}",
                                        oninput: move |event| {
                                            wine_prefix_draft.set(event.value());
                                        },
                                        onblur: move |_| {
                                            on_wine_prefix_change.call((vn.id, wine_prefix_draft.read().clone()));
                                        },
                                    }
                                }
                                button {
                                    class: "fp-button",
                                    onclick: move |_| {
                                        let picked_folder = FileDialog::new().pick_folder();
                                        if let Some(path) = picked_folder {
                                            let path_text = path.to_string_lossy().to_string();
                                            wine_prefix_draft.set(path_text.clone());
                                            on_wine_prefix_change.call((vn.id, path.to_string_lossy().to_string()));
                                        }
                                    },
                                    "Choose prefix folder"
                                }
                                div { class: "locale-field",
                                    label {
                                        "Wine locale preset"
                                        select {
                                            value: "{wine_locale_value}",
                                            onchange: move |event| {
                                                let locale = event.value();
                                                wine_locale_draft.set(locale.clone());
                                                on_wine_locale_change.call((vn.id, locale));
                                            },
                                            option { value: "", "System locale" }
                                            option { value: "ja_JP.UTF-8", "Japanese" }
                                            option { value: "zh_CN.UTF-8", "Chinese Simplified" }
                                            option { value: "ko_KR.UTF-8", "Korean" }
                                            option { value: "en_US.UTF-8", "English US" }
                                            if !wine_locale_value.is_empty() && wine_locale_value != "ja_JP.UTF-8"
                                                && wine_locale_value != "zh_CN.UTF-8" && wine_locale_value != "ko_KR.UTF-8"
                                                && wine_locale_value != "en_US.UTF-8"
                                            {
                                                option { value: "{wine_locale_value}",
                                                    "Custom - {wine_locale_value}"
                                                }
                                            }
                                        }
                                    }
                                }
                                label {
                                    "Custom Wine locale"
                                    input {
                                        placeholder: "Leave blank for system locale",
                                        value: "{wine_locale_value}",
                                        oninput: move |event| {
                                            wine_locale_draft.set(event.value());
                                        },
                                        onblur: move |_| {
                                            on_wine_locale_change.call((vn.id, wine_locale_draft.read().clone()));
                                        },
                                    }
                                }
                                label {
                                    "Launch arguments"
                                    input {
                                        placeholder: "--some-flag",
                                        value: "{launch_arguments_value}",
                                        oninput: move |event| {
                                            launch_arguments_draft.set(event.value());
                                        },
                                        onblur: move |_| {
                                            on_launch_arguments_change.call((vn.id, launch_arguments_draft.read().clone()));
                                        },
                                    }
                                }
                                label {
                                    "Environment Variables"
                                    textarea {
                                        class: "launch-env-input",
                                        placeholder: "KEY=VALUE, one per line",
                                        value: "{launch_environment_value}",
                                        oninput: move |event| {
                                            launch_environment_draft.set(event.value());
                                        },
                                        onblur: move |_| {
                                            on_launch_environment_change
                                                .call((vn.id, launch_environment_draft.read().clone()));
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                if selected_tab == DetailTab::History {
                    h3 { "Playtime" }
                    div { class: "playtime-summary",
                        div {
                            span { class: "stat-label", "Total playtime" }
                            strong { "{total_playtime_text}" }
                        }
                        div {
                            span { class: "stat-label", "Sessions recorded" }
                            strong { "{vn.play_sessions.len()}" }
                        }
                    }
                    div { class: "detail-section-header",
                        h3 { "Play sessions" }
                        button {
                            class: "section-action-btn",
                            title: if *add_session_open.read() { "Cancel adding session" } else { "Add play session" },
                            onclick: move |_| {
                                let next = !*add_session_open.read();
                                add_session_open.set(next);
                            },
                            if *add_session_open.read() { "−" } else { "+" }
                        }
                    }
                    if *add_session_open.read() {
                        div { class: "add-session-form",
                            div { class: "add-session-inputs",
                                label {
                                    "Hours"
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        max: "9999",
                                        value: "{add_session_hours}",
                                        oninput: move |e| add_session_hours.set(e.value()),
                                    }
                                }
                                label {
                                    "Minutes"
                                    input {
                                        r#type: "number",
                                        min: "0",
                                        max: "59",
                                        value: "{add_session_minutes}",
                                        oninput: move |e| add_session_minutes.set(e.value()),
                                    }
                                }
                                label {
                                    "Note"
                                    input {
                                        r#type: "text",
                                        placeholder: "e.g. Steam, Offline",
                                        value: "{add_session_note}",
                                        oninput: move |e| add_session_note.set(e.value()),
                                    }
                                }
                            }
                            div { class: "add-session-actions",
                                button {
                                    class: "add-session-save-btn",
                                    onclick: move |_| {
                                        let h: u64 = add_session_hours.read().parse().unwrap_or(0).min(9999);
                                        let m: u64 = add_session_minutes.read().parse().unwrap_or(0).min(59);
                                        let total_sec = h.saturating_mul(3600).saturating_add(m.saturating_mul(60));
                                        if total_sec > 0 {
                                            let note = add_session_note.read().trim().to_string();
                                            let note_opt = if note.is_empty() { None } else { Some(note) };
                                            on_add_play_session.call((vn.id, total_sec, note_opt));
                                            add_session_open.set(false);
                                            add_session_hours.set("0".to_string());
                                            add_session_minutes.set("30".to_string());
                                            add_session_note.set(String::new());
                                        }
                                    },
                                    "Save"
                                }
                                button {
                                    class: "add-session-cancel-btn",
                                    onclick: move |_| {
                                        add_session_open.set(false);
                                    },
                                    "Cancel"
                                }
                            }
                        }
                    }
                    if vn.play_sessions.is_empty() {
                        p { class: "stat-label", "No play sessions yet." }
                    } else {
                        div { class: "history-session-list",
                            for (idx, session) in vn.play_sessions.iter().enumerate().rev() {
                                div { class: "history-session-item",
                                    div { class: "history-session-info",
                                        span { class: "session-date", "{format_started_at(session.started_at.clone())}" }
                                        if let Some(ref note) = session.notes {
                                            span { class: "session-note", "{note}" }
                                        }
                                    }
                                    span { class: "session-duration", "{format_session_duration(session.duration_seconds)}" }
                                    button {
                                        class: "session-delete-button",
                                        title: "Delete session",
                                        onclick: move |_| {
                                            on_delete_play_session.call((vn.id, idx));
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }
                if selected_tab == DetailTab::Routes {
                    {
                        let completed_count = vn.routes.iter().filter(|r| r.completed).count();
                        let total_count = vn.routes.len();
                        let is_all_completed = completed_count == total_count && total_count > 0;
                        let progress_badge_class = if is_all_completed {
                            "route-progress-badge all-completed"
                        } else {
                            "route-progress-badge"
                        };
                        rsx! {
                            div { class: "detail-section-header",
                                h3 { "Routes" }
                                if total_count > 0 {
                                    span { class: "{progress_badge_class}", "{completed_count} / {total_count} completed" }
                                }
                            }
                            div { class: "route-add-row",
                                input {
                                    class: if route_error_text.is_some() { "route-add-input field-invalid" } else { "route-add-input" },
                                    placeholder: "New route name...",
                                    value: "{typed_route_name}",
                                    oninput: move |event| {
                                        if route_error_message.read().is_some() {
                                            route_error_message.set(None);
                                        }
                                        new_route_name.set(event.value());
                                    },
                                    onkeydown: move |event: KeyboardEvent| {
                                        if event.key() == Key::Enter {
                                            let route_name = new_route_name.read().trim().to_string();
                                            if route_name.is_empty() {
                                                route_error_message.set(Some("Route name is required.".to_string()));
                                                return;
                                            }
                                            if existing_routes_for_key.iter().any(|r| r.name.trim().eq_ignore_ascii_case(&route_name)) {
                                                route_error_message.set(Some("A route with this name already exists.".to_string()));
                                                return;
                                            }
                                            route_error_message.set(None);
                                            on_route_add.call((vn.id, route_name));
                                            new_route_name.set(String::new());
                                        }
                                    },
                                }
                                button {
                                    class: "route-add-btn",
                                    title: "Add route",
                                    onclick: move |_| {
                                        let route_name = new_route_name.read().trim().to_string();
                                        if route_name.is_empty() {
                                            route_error_message.set(Some("Route name is required.".to_string()));
                                            return;
                                        }
                                        if existing_routes_for_click.iter().any(|r| r.name.trim().eq_ignore_ascii_case(&route_name)) {
                                            route_error_message.set(Some("A route with this name already exists.".to_string()));
                                            return;
                                        }
                                        route_error_message.set(None);
                                        on_route_add.call((vn.id, route_name));
                                        new_route_name.set(String::new());
                                    },
                                    "+"
                                }
                            }
                            if let Some(ref err_msg) = route_error_text {
                                p { class: "field-error", "{err_msg}" }
                            }
                            if vn.routes.is_empty() {
                                p { class: "stat-label", "No routes yet." }
                            } else {
                                div { class: "route-list",
                                    for route in vn.routes.clone() {
                                        {
                                            let is_active = vn.active_route.as_deref() == Some(route.name.as_str());
                                            rsx! {
                                                RouteItem {
                                                    key: "{route.name}",
                                                    vn_id: vn.id,
                                                    route,
                                                    is_active,
                                                    on_route_toggle,
                                                    on_route_set_active: on_active_route_change,
                                                    on_route_delete,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "detail-launch-footer",
                if cfg!(target_os = "windows") || vn.launch_mode != LaunchMode::Native {
                    button {
                        class: "secondary-launch-button",
                        onclick: move |_| {
                            let picked_file = FileDialog::new()
                                .add_filter("Executables", &["exe", "bin", "sh", "AppImage"])
                                .add_filter("All Files", &["*"])
                                .pick_file();
                            if let Some(path) = picked_file {
                                on_run_tool.call((vn.id, path.to_string_lossy().to_string()));
                            }
                        },
                        if cfg!(target_os = "windows") {
                            "Run auxiliary .exe"
                        } else {
                            "Run .exe inside prefix"
                        }
                    }
                }
                button {
                    class: "launch-button",
                    disabled: if vn.launch_mode == LaunchMode::Steam {
                        vn.steam_app_id.is_none()
                    } else {
                        vn.executable_path.is_none()
                    },
                    onclick: move |_| {
                        on_launch.call(vn.id);
                    },
                    "Launch"
                }
            }
        }
    }
}

fn format_started_at(started_at: String) -> String {
    let parsed_time = match chrono::DateTime::parse_from_rfc3339(&started_at) {
        Ok(time) => time,
        Err(_error) => return started_at,
    };
    parsed_time.format("%Y-%m-%d %H:%M").to_string()
}

fn format_playtime(total_seconds: u64) -> String {
    let hours = total_seconds as f64 / 3600.0;
    if hours < 1.0 {
        let minutes = total_seconds / 60;
        format!("{minutes} min")
    } else {
        format!("{hours:.1} hrs")
    }
}

fn format_session_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let rem_seconds = seconds % 60;
        if rem_seconds > 0 {
            format!("{minutes}m {rem_seconds}s")
        } else {
            format!("{minutes}m")
        }
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        if minutes > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{hours}h")
        }
    }
}

#[component]
fn DescriptionPartView(part: DescriptionPart) -> Element {
    match part {
        DescriptionPart::Text(text) => {
            rsx! {
                span { "{text}" }
            }
        }
        DescriptionPart::Bold(text) => {
            rsx! {
                strong { "{text}" }
            }
        }
        DescriptionPart::Italic(text) => {
            rsx! {
                em { "{text}" }
            }
        }
        DescriptionPart::Link { label, url } => {
            if crate::system::is_safe_url(&url) {
                let target_url = url.clone();
                rsx! {
                    a {
                        href: "{url}",
                        target: "_blank",
                        onclick: move |e| {
                            e.prevent_default();
                            let _ = crate::system::open_url(&target_url);
                        },
                        "{label}"
                    }
                }
            } else {
                rsx! {
                    span { "{label}" }
                }
            }
        }
        DescriptionPart::Spoiler(parts) => {
            rsx! {
                SpoilerText { parts }
            }
        }
    }
}

#[component]
fn SpoilerText(parts: Vec<DescriptionPart>) -> Element {
    let mut is_revealed = use_signal(|| false);
    if *is_revealed.read() {
        rsx! {
            span { class: "spoiler-text revealed",
                for part in parts {
                    DescriptionPartView { part }
                }
            }
        }
    } else {
        rsx! {
            button {
                class: "spoiler-text hidden",
                onclick: move |_| {
                    is_revealed.set(true);
                },
                "Spoiler"
            }
        }
    }
}

#[component]
fn RouteItem(
    vn_id: u64,
    route: StoryRoute,
    is_active: bool,
    on_route_toggle: EventHandler<(u64, String)>,
    on_route_set_active: EventHandler<(u64, Option<String>)>,
    on_route_delete: EventHandler<(u64, String)>,
) -> Element {
    let route_name = route.name.clone();
    let route_name_for_toggle = route.name.clone();
    let route_name_for_active = route.name.clone();
    let route_name_for_delete = route.name.clone();
    let item_class = if is_active {
        "route-item active"
    } else {
        "route-item"
    };
    let name_class = if route.completed {
        "route-name completed"
    } else {
        "route-name"
    };

    rsx! {
        div { class: "{item_class}",
            input {
                class: "route-checkbox",
                r#type: "checkbox",
                checked: route.completed,
                title: if route.completed { "Mark as incomplete" } else { "Mark as completed" },
                onchange: move |_| {
                    on_route_toggle.call((vn_id, route_name_for_toggle.clone()));
                },
            }
            button {
                class: "route-name-btn",
                r#type: "button",
                title: if is_active { "Clear active route" } else { "Set as active route" },
                onclick: move |_| {
                    if is_active {
                        on_route_set_active.call((vn_id, None));
                    } else {
                        on_route_set_active.call((vn_id, Some(route_name_for_active.clone())));
                    }
                },
                span { class: "{name_class}", "{route_name}" }
                if is_active {
                    span { class: "route-active-badge", "Playing" }
                }
            }
            button {
                class: "route-delete-button",
                r#type: "button",
                title: "Delete route",
                onclick: move |_| {
                    on_route_delete.call((vn_id, route_name_for_delete.clone()));
                },
                "✕"
            }
        }
    }
}
