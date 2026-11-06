use dioxus::prelude::*;

#[component]
pub fn SettingsView(
    discord_rich_presence_enabled: bool,
    discord_status_text: String,
    discord_show_active_route: bool,
    discord_custom_cover_url: String,
    data_dir_text: String,
    custom_background_path: Option<String>,
    leproc_path: Option<String>,
    blur_nsfw: bool,
    on_discord_rich_presence_change: EventHandler<bool>,
    on_discord_status_text_change: EventHandler<String>,
    on_discord_show_active_route_change: EventHandler<bool>,
    on_discord_custom_cover_url_change: EventHandler<String>,
    on_custom_background_path_change: EventHandler<Option<String>>,
    on_leproc_path_change: EventHandler<Option<String>>,
    on_blur_nsfw_change: EventHandler<bool>,
    on_open_data_folder: EventHandler<()>,
    on_open_logs_folder: EventHandler<()>,
    on_export_library: EventHandler<()>,
    on_import_library: EventHandler<()>,
) -> Element {
    let mut discord_status_text_draft = use_signal(|| discord_status_text.clone());
    let mut discord_custom_cover_url_draft = use_signal(|| discord_custom_cover_url.clone());
    let discord_status_text_value = discord_status_text_draft.read().clone();
    let discord_custom_cover_url_value = discord_custom_cover_url_draft.read().clone();
    rsx! {
        section { class: "settings-panel",
            h2 { "Settings" }
            div { class: "settings-section",
                h3 { "Appearance" }
                div { class: "setting-row",
                    span { "Background wallpaper" }
                    if let Some(ref path) = custom_background_path {
                        code { class: "setting-path", "{path}" }
                    } else {
                        span { class: "setting-help", "Default stars wallpaper" }
                    }
                }
                div { class: "setting-button-row",
                    button {
                        class: "fp-button",
                        onclick: move |_| {
                            let picked_file = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                .pick_file();
                            if let Some(path) = picked_file {
                                on_custom_background_path_change.call(Some(path.to_string_lossy().to_string()));
                            }
                        },
                        "Choose background image"
                    }
                    if custom_background_path.is_some() {
                        button {
                            class: "fp-button",
                            onclick: move |_| {
                                on_custom_background_path_change.call(None);
                            },
                            "Reset to default background"
                        }
                    }
                }
                div { class: "setting-row",
                    label { class: "locale-emulator-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: blur_nsfw,
                            onchange: move |event| {
                                on_blur_nsfw_change.call(event.checked());
                            },
                        }
                        span { "Blur 18+ covers (NSFW)" }
                    }
                }
            }
            if cfg!(target_os = "windows") {
                div { class: "settings-section",
                    h3 { "Locale Emulator" }
                    div { class: "setting-row",
                        span { "LEProc.exe path" }
                        if let Some(ref path) = leproc_path {
                            code { class: "setting-path", "{path}" }
                        } else {
                            span { class: "setting-help", "Not configured" }
                        }
                    }
                    div { class: "setting-button-row",
                        button {
                            class: "fp-button",
                            onclick: move |_| {
                                let picked_file = rfd::FileDialog::new()
                                    .add_filter("Executable", &["exe"])
                                    .pick_file();
                                if let Some(path) = picked_file {
                                    on_leproc_path_change.call(Some(path.to_string_lossy().to_string()));
                                }
                            },
                            "Locate LEProc.exe"
                        }
                        if leproc_path.is_some() {
                            button {
                                class: "fp-button",
                                onclick: move |_| {
                                    on_leproc_path_change.call(None);
                                },
                                "Clear"
                            }
                        }
                    }
                    p { class: "setting-help", "Used to launch Japanese visual novels requiring Shift-JIS system locale." }
                }
            }
            div { class: "settings-section",
                h3 { "Discord Rich Presence" }
                label { class: "setting-row",
                    span { "Enable Rich Presence" }
                    input {
                        class: "setting-checkbox",
                        r#type: "checkbox",
                        checked: discord_rich_presence_enabled,
                        onchange: move |event| {
                            on_discord_rich_presence_change.call(event.checked());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Default status text" }
                    input {
                        value: "{discord_status_text_value}",
                        oninput: move |event| {
                            discord_status_text_draft.set(event.value());
                        },
                        onblur: move |_| {
                            on_discord_status_text_change.call(discord_status_text_draft.read().clone());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Show active route" }
                    input {
                        class: "setting-checkbox",
                        r#type: "checkbox",
                        checked: discord_show_active_route,
                        onchange: move |event| {
                            on_discord_show_active_route_change.call(event.checked());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Custom cover URL" }
                    input {
                        value: "{discord_custom_cover_url_value}",
                        placeholder: "Leave blank to use VNDB cover",
                        oninput: move |event| {
                            discord_custom_cover_url_draft.set(event.value());
                        },
                        onblur: move |_| {
                            on_discord_custom_cover_url_change
                                .call(discord_custom_cover_url_draft.read().clone());
                        },
                    }
                }
                p { class: "setting-help", "Show the VN being played on your Discord profile." }
            }
            div { class: "settings-section",
                h3 { "Data & Backup" }
                div { class: "setting-row",
                    span { "Data folder" }
                    code { class: "setting-path", "{data_dir_text}" }
                }
                div { class: "setting-button-row",
                    button {
                        class: "fp-button",
                        onclick: move |_| {
                            on_export_library.call(());
                        },
                        "Export Library (JSON)"
                    }
                    button {
                        class: "fp-button",
                        onclick: move |_| {
                            on_import_library.call(());
                        },
                        "Import Library (JSON)"
                    }
                }
                div { class: "setting-button-row",
                    button {
                        class: "fp-button",
                        onclick: move |_| {
                            on_open_data_folder.call(());
                        },
                        "Open data folder"
                    }
                    button {
                        class: "fp-button",
                        onclick: move |_| {
                            on_open_logs_folder.call(());
                        },
                        "Open logs folder"
                    }
                }
            }
        }
    }
}
