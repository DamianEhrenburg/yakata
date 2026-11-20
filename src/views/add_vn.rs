use crate::models::{LaunchMode, NovelStatus};
use crate::views::dropdown::{Dropdown, DropdownOption};
use crate::vndb::{VndbSearchResult, VndbSortOrder, search_vns};
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct NewVN {
    pub title: String,
    pub cover_url: Option<String>,
    pub cover_path: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub vndb_id: Option<String>,
    pub status: Option<NovelStatus>,
    pub rating: Option<u8>,
    pub is_nsfw: bool,
    pub executable_path: Option<String>,
    pub run_with_locale_emulator: bool,
    pub launch_mode: LaunchMode,
    pub steam_app_id: Option<u32>,
}

#[component]
pub fn AddVnForm(
    blur_nsfw: bool,
    existing_titles: Vec<String>,
    existing_vndb_ids: Vec<String>,
    on_add: EventHandler<NewVN>,
    on_close: EventHandler<()>,
) -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut cover_path = use_signal(|| None::<String>);
    let mut cover_url = use_signal(|| None::<String>);
    let description_text = description.read().clone();
    let cover_path_text = cover_path.read().clone();
    let mut tags_text = use_signal(String::new);
    let current_tags_text = tags_text.read().clone();
    let mut selected_status = use_signal(|| Some(NovelStatus::PlanToRead));
    let mut selected_rating = use_signal(|| None::<u8>);
    let mut loaded_is_nsfw = use_signal(|| false);
    let mut executable_path = use_signal(|| None::<String>);
    let mut run_with_locale_emulator = use_signal(|| false);
    let mut launch_mode = use_signal(|| LaunchMode::Native);
    let mut steam_app_id = use_signal(String::new);
    let mut vndb_query = use_signal(String::new);
    let mut vndb_sort = use_signal(|| VndbSortOrder::SearchRank);
    let mut sort_reverse = use_signal(|| false);
    let search_results: Signal<Vec<VndbSearchResult>> = use_signal(Vec::new);
    let mut search_message = use_signal(String::new);
    let mut title_has_error = use_signal(|| false);
    let mut loaded_result_id = use_signal(|| None::<String>);
    let search_request_id = use_signal(|| 0u64);
    let message_text = search_message.read().clone();
    let results_to_show = search_results.read().clone();

    let mut status_options = vec![DropdownOption::new("none", "No status")];
    status_options.extend(
        NovelStatus::all()
            .iter()
            .map(|s| DropdownOption::new(s.key(), s.as_str())),
    );

    let current_status_val = (*selected_status.read())
        .map(|s| s.key())
        .unwrap_or("none")
        .to_string();

    let score_options = vec![
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
    ];

    let current_score_val = match *selected_rating.read() {
        Some(r) => r.to_string(),
        None => "none".to_string(),
    };

    let sort_options = vec![
        DropdownOption::new("searchrank", "Relevance"),
        DropdownOption::new("rating", "Rating"),
        DropdownOption::new("released", "Newest"),
        DropdownOption::new("title", "Title"),
    ];

    let current_sort_val = match *vndb_sort.read() {
        VndbSortOrder::SearchRank => "searchrank",
        VndbSortOrder::Rating => "rating",
        VndbSortOrder::Released => "released",
        VndbSortOrder::Title => "title",
    }
    .to_string();

    let mut launch_mode_options = vec![DropdownOption::new("native", "Native")];
    if cfg!(target_os = "linux") {
        launch_mode_options.push(DropdownOption::new("wine", "Wine"));
        launch_mode_options.push(DropdownOption::new("proton", "Proton"));
    }
    launch_mode_options.push(DropdownOption::new("steam", "Steam"));

    let current_launch_mode_val = match *launch_mode.read() {
        LaunchMode::Native => "native",
        LaunchMode::Wine => "wine",
        LaunchMode::Proton => "proton",
        LaunchMode::Steam => "steam",
    }
    .to_string();

    rsx! {
        section {
            class: "add-vn-panel",
            tabindex: "0",
            onkeydown: move |evt| {
                if evt.key().to_string() == "Escape" {
                    on_close.call(());
                }
            },
            div { class: "add-vn-header",
                h2 { "Add VN" }
                button {
                    class: "add-vn-close",
                    aria_label: "Close add VN form",
                    onclick: move |_| {
                        on_close.call(());
                    },
                    "×"
                }
            }
            div { class: "add-vn-body-grid",
                div { class: "add-vn-form-col",
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Title" }
                        input {
                            class: if *title_has_error.read() { "field-invalid" } else { "" },
                            value: "{title}",
                            placeholder: "Novel title",
                            oninput: move |event| {
                                let value = event.value();
                                if !value.trim().is_empty() {
                                    title_has_error.set(false);
                                }
                                if vndb_query.read().is_empty() {
                                    vndb_query.set(value.clone());
                                }
                                title.set(value);
                            },
                        }
                    }
                    if *title_has_error.read() {
                        p { class: "field-error", "A title is required" }
                    }
                    div { class: "add-vn-form-row",
                        div { class: "add-vn-field-group",
                            span { class: "add-vn-field-label", "Status" }
                            Dropdown {
                                options: status_options,
                                selected_value: current_status_val,
                                on_select: move |val: String| {
                                    selected_status.set(NovelStatus::from_key(&val));
                                },
                            }
                        }
                        div { class: "add-vn-field-group",
                            span { class: "add-vn-field-label", "Score" }
                            Dropdown {
                                class: "score-dropdown",
                                options: score_options,
                                selected_value: current_score_val,
                                on_select: move |val: String| {
                                    let rating = if val == "none" { None } else { val.parse::<u8>().ok() };
                                    selected_rating.set(rating);
                                },
                            }
                        }
                    }
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Tags" }
                        input {
                            value: "{current_tags_text}",
                            placeholder: "Action, Romance, Sci-Fi",
                            oninput: move |event| {
                                tags_text.set(event.value());
                            },
                        }
                    }
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Cover path" }
                        div { class: "add-vn-input-browse-row",
                            input {
                                value: cover_path_text.clone().unwrap_or_default(),
                                placeholder: "Optional: local image path",
                                oninput: move |event| {
                                    let value = event.value();
                                    if value.trim().is_empty() {
                                        cover_path.set(None);
                                    } else {
                                        cover_path.set(Some(value));
                                        cover_url.set(None);
                                    }
                                },
                            }
                            button {
                                class: "add-vn-browse-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let picked_file = rfd::FileDialog::new()
                                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                        .pick_file();
                                    if let Some(path) = picked_file {
                                        let path_str = path.to_string_lossy().to_string();
                                        cover_path.set(Some(path_str));
                                        cover_url.set(None);
                                    }
                                },
                                "Browse"
                            }
                        }
                    }
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Description" }
                        textarea {
                            class: "add-vn-desc-input",
                            value: "{description_text}",
                            placeholder: "Optional synopsis or notes",
                            oninput: move |event| {
                                description.set(event.value());
                            },
                        }
                    }
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Executable path" }
                        div { class: "add-vn-input-browse-row",
                            input {
                                value: executable_path.read().clone().unwrap_or_default(),
                                placeholder: "Optional: path to game .exe",
                                oninput: move |event| {
                                    let value = event.value();
                                    if value.trim().is_empty() {
                                        executable_path.set(None);
                                    } else {
                                        executable_path.set(Some(value));
                                    }
                                },
                            }
                            button {
                                class: "add-vn-browse-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let picked_file = rfd::FileDialog::new()
                                        .add_filter("Executable", &["exe", "bin", "sh", "AppImage"])
                                        .pick_file();
                                    if let Some(path) = picked_file {
                                        let path_str = path.to_string_lossy().to_string();
                                        executable_path.set(Some(path_str.clone()));
                                        if title.read().trim().is_empty()
                                            && let Some(inferred) = infer_title_from_executable_path(&path_str)
                                        {
                                            title.set(inferred.clone());
                                            vndb_query.set(inferred.clone());
                                            title_has_error.set(false);
                                            let current_sort = *vndb_sort.read();
                                            let current_rev = *sort_reverse.read();
                                            execute_vndb_search(
                                                inferred,
                                                current_sort,
                                                current_rev,
                                                search_request_id,
                                                search_message,
                                                search_results,
                                                loaded_result_id,
                                            );
                                        }
                                    }
                                },
                                "Browse"
                            }
                        }
                    }
                    if cfg!(target_os = "windows") && *launch_mode.read() != LaunchMode::Steam {
                        label { class: "checkbox-row",
                            input {
                                class: "setting-checkbox",
                                r#type: "checkbox",
                                checked: *run_with_locale_emulator.read(),
                                onchange: move |event| {
                                    run_with_locale_emulator.set(event.checked());
                                },
                            }
                            span { "Run with Locale Emulator" }
                        }
                    }
                    div { class: "add-vn-field-group",
                        span { class: "add-vn-field-label", "Launch mode" }
                        Dropdown {
                            class: "drop-up".to_string(),
                            options: launch_mode_options,
                            selected_value: current_launch_mode_val,
                            on_select: move |val: String| {
                                let mode = match val.as_str() {
                                    "wine" => LaunchMode::Wine,
                                    "proton" => LaunchMode::Proton,
                                    "steam" => LaunchMode::Steam,
                                    _ => LaunchMode::Native,
                                };
                                launch_mode.set(mode);
                            },
                        }
                    }
                    if *launch_mode.read() == LaunchMode::Steam {
                        div { class: "add-vn-field-group",
                            span { class: "add-vn-field-label", "Steam App ID" }
                            input {
                                r#type: "text",
                                value: "{steam_app_id.read()}",
                                placeholder: "e.g. 1989270 or Steam URL",
                                oninput: move |event| {
                                    steam_app_id.set(event.value());
                                },
                                onblur: move |_| {
                                    let parsed = crate::launcher::parse_steam_app_id(&steam_app_id.read());
                                    if let Some(id) = parsed {
                                        steam_app_id.set(id.to_string());
                                    }
                                },
                            }
                        }
                    }
                    button {
                        class: "add-vn-submit-button",
                        onclick: move |_| {
                            let typed_title = title.read().trim().to_string();
                            if typed_title.is_empty() {
                                title_has_error.set(true);
                                search_message.set("A title is required".to_string());
                                return;
                            }
                            if existing_titles.iter().any(|t| t.trim().eq_ignore_ascii_case(&typed_title)) {
                                title_has_error.set(true);
                                search_message.set(format!("\"{}\" is already in your library.", typed_title));
                                return;
                            }
                            if let Some(loaded_id) = loaded_result_id.read().as_ref()
                                && existing_vndb_ids.iter().any(|id| id.eq_ignore_ascii_case(loaded_id))
                            {
                                title_has_error.set(true);
                                search_message.set("This visual novel is already in your library.".to_string());
                                return;
                            }
                            let final_steam_app_id = if *launch_mode.read() == LaunchMode::Steam {
                                crate::launcher::parse_steam_app_id(&steam_app_id.read())
                            } else {
                                None
                            };
                            on_add
                                .call(NewVN {
                                    title: typed_title,
                                    cover_url: cover_url.read().clone(),
                                    cover_path: cover_path.read().clone(),
                                    description: if description.read().is_empty() {
                                        None
                                    } else {
                                        Some(description.read().clone())
                                    },
                                    tags: parse_tags(tags_text.read().clone()),
                                    vndb_id: loaded_result_id.read().clone(),
                                    status: *selected_status.read(),
                                    rating: *selected_rating.read(),
                                    is_nsfw: *loaded_is_nsfw.read(),
                                    executable_path: executable_path.read().clone(),
                                    run_with_locale_emulator: if *launch_mode.read() == LaunchMode::Steam {
                                        false
                                    } else {
                                        *run_with_locale_emulator.read()
                                    },
                                    launch_mode: launch_mode.read().clone(),
                                    steam_app_id: final_steam_app_id,
                                });
                            title.set(String::new());
                            description.set(String::new());
                            cover_path.set(None);
                            cover_url.set(None);
                            loaded_result_id.set(None);
                            loaded_is_nsfw.set(false);
                            executable_path.set(None);
                            run_with_locale_emulator.set(false);
                            launch_mode.set(LaunchMode::Native);
                            steam_app_id.set(String::new());
                            tags_text.set(String::new());
                            selected_rating.set(None);
                        },
                        "Add to library"
                    }
                }
                div { class: "add-vn-vndb-col",
                    div { class: "vndb-col-header",
                        span { class: "vndb-results-title", "VNDB Search & Import" }
                        if !message_text.is_empty() {
                            span { class: "vndb-status-pill", "{message_text}" }
                        }
                    }
                    div { class: "vndb-search-row",
                        input {
                            class: "vndb-search-input",
                            placeholder: "Search VNDB by title or ID...",
                            value: "{vndb_query}",
                            oninput: move |e| vndb_query.set(e.value()),
                            onkeydown: move |e| {
                                if e.key().to_string() == "Enter" {
                                    let query = vndb_query.read().clone();
                                    let sort = *vndb_sort.read();
                                    let rev = *sort_reverse.read();
                                    execute_vndb_search(
                                        query,
                                        sort,
                                        rev,
                                        search_request_id,
                                        search_message,
                                        search_results,
                                        loaded_result_id,
                                    );
                                }
                            },
                        }
                        Dropdown {
                            class: "vndb-sort-dropdown",
                            options: sort_options,
                            selected_value: current_sort_val,
                            on_select: move |val: String| {
                                let new_sort = match val.as_str() {
                                    "rating" => {
                                        sort_reverse.set(true);
                                        VndbSortOrder::Rating
                                    }
                                    "released" => {
                                        sort_reverse.set(true);
                                        VndbSortOrder::Released
                                    }
                                    "title" => {
                                        sort_reverse.set(false);
                                        VndbSortOrder::Title
                                    }
                                    _ => {
                                        sort_reverse.set(false);
                                        VndbSortOrder::SearchRank
                                    }
                                };
                                vndb_sort.set(new_sort);
                                let rev = *sort_reverse.read();
                                let query = vndb_query.read().clone();
                                if !query.trim().is_empty() {
                                    execute_vndb_search(
                                        query,
                                        new_sort,
                                        rev,
                                        search_request_id,
                                        search_message,
                                        search_results,
                                        loaded_result_id,
                                    );
                                }
                            },
                        }
                        button {
                            class: "vndb-sort-direction-btn",
                            r#type: "button",
                            title: if *sort_reverse.read() { "Sort descending" } else { "Sort ascending" },
                            onclick: move |_| {
                                let new_rev = !*sort_reverse.read();
                                sort_reverse.set(new_rev);
                                let query = vndb_query.read().clone();
                                if !query.trim().is_empty() {
                                    let sort = *vndb_sort.read();
                                    execute_vndb_search(
                                        query,
                                        sort,
                                        new_rev,
                                        search_request_id,
                                        search_message,
                                        search_results,
                                        loaded_result_id,
                                    );
                                }
                            },
                            if *sort_reverse.read() { "▼" } else { "▲" }
                        }
                        button {
                            class: "vndb-search-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let query = vndb_query.read().clone();
                                let sort = *vndb_sort.read();
                                let rev = *sort_reverse.read();
                                execute_vndb_search(
                                    query,
                                    sort,
                                    rev,
                                    search_request_id,
                                    search_message,
                                    search_results,
                                    loaded_result_id,
                                );
                            },
                            "Search"
                        }
                    }
                    if results_to_show.is_empty() {
                        div { class: "vndb-empty-hint",
                            p { "Search by title or VNDB ID" }
                        }
                    } else {
                        div { class: "vndb-results-grid",
                            for result in results_to_show {
                                {
                                    let result_title = result.title.clone();
                                    let result_description = result.description.clone();
                                    let result_cover_url = result.image.clone().and_then(|image| image.url);
                                    let is_sexual = result.image.as_ref().and_then(|img| img.sexual).unwrap_or(0.0) >= 1.0;
                                    let has_ero_tag = result.tags.iter().any(|t| t.category.as_deref() == Some("ero"));
                                    let is_nsfw = is_sexual || has_ero_tag || {
                                        let lower = result.title.to_lowercase();
                                        lower.contains("18+") || lower.contains("eroge") || lower.contains("nukige") || lower.contains("hentai") || lower.contains("sexual") || lower.contains("nsfw")
                                    };
                                    let cover_class = if blur_nsfw && is_nsfw { "vndb-result-cover nsfw-blur" } else { "vndb-result-cover" };
                                    let result_id = result.id.clone();
                                    let result_is_loaded = loaded_result_id.read().as_ref() == Some(&result.id);
                                    let is_already_added = existing_vndb_ids.iter().any(|id| id.eq_ignore_ascii_case(&result.id))
                                        || existing_titles.iter().any(|t| t.trim().eq_ignore_ascii_case(result.title.trim()));
                                    let card_class = if is_already_added {
                                        "vndb-result-card in-library"
                                    } else if result_is_loaded {
                                        "vndb-result-card loaded"
                                    } else {
                                        "vndb-result-card"
                                    };
                                    let load_result = {
                                        let result_title = result_title.clone();
                                        let result_description = result_description.clone();
                                        let result_cover_url = result_cover_url.clone();
                                        let result_id = result_id.clone();
                                        let tags = result.tags.clone();
                                        move |_| {
                                            if is_already_added {
                                                search_message.set(format!("\"{}\" is already in your library.", result_title));
                                                return;
                                            }
                                            title.set(result_title.clone());
                                            description.set(result_description.clone().unwrap_or_default());
                                            cover_url.set(result_cover_url.clone());
                                            cover_path.set(None);

                                            let top_tags = crate::vndb::filter_vndb_tags(&tags, is_sexual || is_nsfw);
                                            if !top_tags.is_empty() {
                                                tags_text.set(top_tags.join(", "));
                                            }

                                            loaded_is_nsfw.set(is_nsfw);
                                            loaded_result_id.set(Some(result_id.clone()));
                                            search_message.set(format!("Loaded {}.", result_title));
                                        }
                                    };
                                    rsx! {
                                        article {
                                            class: "{card_class}",
                                            onclick: load_result,
                                            div { class: "vndb-result-cover-frame",
                                                if let Some(cover_src) = result_cover_url.clone() {
                                                    img {
                                                        class: "{cover_class}",
                                                        src: "{cover_src}",
                                                        alt: "Cover art for {result.title}",
                                                    }
                                                } else {
                                                    div { class: "vndb-result-cover-placeholder", "No cover" }
                                                }
                                                if let Some(rating) = result.rating {
                                                    span { class: "vndb-rating-badge", "★ {rating / 10.0:.2}" }
                                                }
                                            }
                                            div { class: "vndb-result-body",
                                                h3 { "{result.title}" }
                                                if let Some(ref rel) = result.released {
                                                    if rel.len() >= 4 {
                                                        span { class: "vndb-release-year", "{&rel[..4]}" }
                                                    }
                                                }
                                                p { "VNDB ID: {result.id}" }
                                                button {
                                                    class: if is_already_added {
                                                        "vndb-result-button in-library"
                                                    } else if result_is_loaded {
                                                        "vndb-result-button loaded"
                                                    } else {
                                                        "vndb-result-button"
                                                    },
                                                    tabindex: "-1",
                                                    if is_already_added {
                                                        "In library"
                                                    } else if result_is_loaded {
                                                        "Loaded"
                                                    } else {
                                                        "Use"
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
            }
        }
    }
}

fn execute_vndb_search(
    query: String,
    sort: VndbSortOrder,
    rev: bool,
    mut search_request_id: Signal<u64>,
    mut search_message: Signal<String>,
    mut search_results: Signal<Vec<VndbSearchResult>>,
    mut loaded_result_id: Signal<Option<String>>,
) {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        search_message.set("Enter a title or VNDB ID to search".to_string());
        return;
    }
    loaded_result_id.set(None);
    search_message.set("Searching VNDB...".to_string());
    let req_id = {
        let next = *search_request_id.read() + 1;
        search_request_id.set(next);
        next
    };
    spawn(async move {
        let res = search_vns(trimmed, sort, rev).await;
        if *search_request_id.read() != req_id {
            return;
        }
        match res {
            Ok(results) => {
                search_message.set(format_results_count(results.len()));
                search_results.set(results);
            }
            Err(error) => {
                search_message.set(format!("VNDB search failed: {error}"));
            }
        }
    });
}

fn infer_title_from_executable_path(path_str: &str) -> Option<String> {
    let path = std::path::Path::new(path_str);
    let mut current = path.parent();
    let skip_names = [
        "bin", "binaries", "game", "games", "system", "win32", "win64", "release", "data", "x86", "x64",
        "windowsnoeditor", "content", "app", "engine",
    ];
    while let Some(parent) = current {
        let name = parent.file_name()?.to_string_lossy();
        if !skip_names.contains(&name.to_lowercase().as_str()) && !name.trim().is_empty() {
            let cleaned = clean_folder_title(&name);
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
        current = parent.parent();
    }
    None
}

fn is_version_token(s: &str) -> bool {
    let lower = s.to_lowercase();
    let trimmed = lower.trim();
    if let Some(rest) = trimmed.strip_prefix('v')
        && !rest.is_empty()
        && rest.contains('.')
        && rest.starts_with(|c: char| c.is_ascii_digit())
        && rest.chars().all(|c| c.is_ascii_digit() || c == '.')
        && rest.chars().any(|c| c.is_ascii_digit())
    {
        return true;
    }
    if let Some(rest) = trimmed
        .strip_prefix("ver.")
        .or_else(|| trimmed.strip_prefix("ver"))
    {
        let rest = rest.trim();
        if !rest.is_empty()
            && rest.starts_with(|c: char| c.is_ascii_digit())
            && rest.chars().all(|c| c.is_ascii_digit() || c == '.')
            && rest.chars().any(|c| c.is_ascii_digit())
        {
            return true;
        }
    }
    trimmed.contains('.')
        && trimmed.starts_with(|c: char| c.is_ascii_digit())
        && trimmed.chars().all(|c| c.is_ascii_digit() || c == '.')
        && trimmed.chars().any(|c| c.is_ascii_digit())
}

fn clean_folder_title(name: &str) -> String {
    let mut result = String::new();
    let mut bracket_depth = 0;
    for ch in name.chars() {
        match ch {
            '[' | '(' | '{' | '【' | '（' => bracket_depth += 1,
            ']' | ')' | '}' | '】' | '）' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            _ if bracket_depth == 0 => result.push(ch),
            _ => {}
        }
    }
    let mut s = result.trim().replace('_', " ");

    if !s.contains(' ') && s.contains('.') {
        let raw_segs: Vec<&str> = s.split('.').collect();
        let has_abbreviation = raw_segs.windows(2).any(|w| {
            w[0].len() == 1
                && w[0].chars().all(|c| c.is_alphabetic())
                && w[1].len() == 1
                && w[1].chars().all(|c| c.is_alphabetic())
        });
        if !has_abbreviation {
            let chars: Vec<char> = s.chars().collect();
            let n = chars.len();
            let mut new_chars = Vec::with_capacity(n);
            for i in 0..n {
                if chars[i] == '.' {
                    let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
                    let next_is_digit = i + 1 < n && chars[i + 1].is_ascii_digit();
                    if prev_is_digit && next_is_digit {
                        new_chars.push('.');
                    } else {
                        new_chars.push(' ');
                    }
                } else {
                    new_chars.push(chars[i]);
                }
            }
            s = new_chars.into_iter().collect();
        }
    }

    let mut tokens: Vec<&str> = s.split_whitespace().collect();
    let mut removed_version = false;
    while let Some(&last) = tokens.last() {
        if is_version_token(last) {
            tokens.pop();
            removed_version = true;
        } else {
            break;
        }
    }
    if removed_version && tokens.last() == Some(&"-") {
        tokens.pop();
    }

    tokens.join(" ")
}

fn parse_tags(text: String) -> Vec<String> {
    text.split(",")
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn format_results_count(count: usize) -> String {
    if count == 1 {
        "Found 1 result".to_string()
    } else {
        format!("Found {count} results")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_title_from_executable_path() {
        assert_eq!(
            infer_title_from_executable_path(r"C:\Games\Steins;Gate\game.exe"),
            Some("Steins;Gate".to_string())
        );
        assert_eq!(
            infer_title_from_executable_path(r"C:\Games\[RJ12345] Mahoyo (v1.0)\bin\game.exe"),
            Some("Mahoyo".to_string())
        );
        assert_eq!(
            infer_title_from_executable_path(r"C:\Games\Fate_Stay_Night\release\fate.exe"),
            Some("Fate Stay Night".to_string())
        );
    }

    #[test]
    fn test_parse_tags() {
        let tags = parse_tags("Action, Sci-Fi, Mystery, , 18+".to_string());
        assert_eq!(tags, vec!["Action", "Sci-Fi", "Mystery", "18+"]);
    }

    #[test]
    fn test_clean_folder_title() {
        assert_eq!(clean_folder_title("Steins;Gate"), "Steins;Gate");
        assert_eq!(clean_folder_title("[RJ12345] Mahoyo (v1.0)"), "Mahoyo");
        assert_eq!(clean_folder_title("【同人】Mahoyo（18禁）"), "Mahoyo");
        assert_eq!(
            clean_folder_title("A.Sky.Full.of.Stars.v1.0"),
            "A Sky Full of Stars"
        );
        assert_eq!(clean_folder_title("Danganronpa V3"), "Danganronpa V3");
        assert_eq!(clean_folder_title("Fate_Stay_Night"), "Fate Stay Night");
        assert_eq!(clean_folder_title("Fate.Stay.Night"), "Fate Stay Night");
        assert_eq!(clean_folder_title("Clannad.v1.05"), "Clannad");
        assert_eq!(clean_folder_title("Q.U.Q"), "Q.U.Q");
        assert_eq!(clean_folder_title("D.C."), "D.C.");
        assert_eq!(clean_folder_title("D.C. ~Da Capo~"), "D.C. ~Da Capo~");
        assert_eq!(
            clean_folder_title("Muv-Luv.Alternative.v1.2"),
            "Muv-Luv Alternative"
        );
        assert_eq!(clean_folder_title("Tsukihime v1.0"), "Tsukihime");
        assert_eq!(
            clean_folder_title("Fate Stay Night - v1.0"),
            "Fate Stay Night"
        );
    }
}
