use crate::models::NovelStatus;
use crate::views::dropdown::{Dropdown, DropdownOption};
use crate::vndb::{VndbSearchResult, search_vns};
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
    let mut vndb_query = use_signal(String::new);
    let mut search_results: Signal<Vec<VndbSearchResult>> = use_signal(Vec::new);
    let mut search_message = use_signal(String::new);
    let mut title_has_error = use_signal(|| false);
    let mut loaded_result_id = use_signal(|| None::<String>);
    let message_text = search_message.read().clone();
    let results_to_show = search_results.read().clone();

    let status_options = vec![
        DropdownOption::new("plan_to_read", "Plan to Read"),
        DropdownOption::new("reading", "Reading"),
        DropdownOption::new("completed", "Completed"),
        DropdownOption::new("on_hold", "On Hold"),
        DropdownOption::new("dropped", "Dropped"),
        DropdownOption::new("none", "No status"),
    ];

    let current_status_val = match *selected_status.read() {
        Some(NovelStatus::Reading) => "reading",
        Some(NovelStatus::Completed) => "completed",
        Some(NovelStatus::PlanToRead) => "plan_to_read",
        Some(NovelStatus::OnHold) => "on_hold",
        Some(NovelStatus::Dropped) => "dropped",
        None => "none",
    }.to_string();

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
                                    let status = match val.as_str() {
                                        "reading" => Some(NovelStatus::Reading),
                                        "completed" => Some(NovelStatus::Completed),
                                        "plan_to_read" => Some(NovelStatus::PlanToRead),
                                        "on_hold" => Some(NovelStatus::OnHold),
                                        "dropped" => Some(NovelStatus::Dropped),
                                        _ => None,
                                    };
                                    selected_status.set(status);
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
                            if let Some(loaded_id) = loaded_result_id.read().as_ref() {
                                if existing_vndb_ids.iter().any(|id| id.eq_ignore_ascii_case(loaded_id)) {
                                    title_has_error.set(true);
                                    search_message.set("This visual novel is already in your library.".to_string());
                                    return;
                                }
                            }
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
                                });
                            title.set(String::new());
                            description.set(String::new());
                            cover_path.set(None);
                            cover_url.set(None);
                            loaded_result_id.set(None);
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
                                    let query = vndb_query.read().trim().to_string();
                                    if query.is_empty() {
                                        search_message.set("Enter a title or VNDB ID to search".to_string());
                                        return;
                                    }
                                    loaded_result_id.set(None);
                                    search_message.set("Searching VNDB...".to_string());
                                    spawn(async move {
                                        match search_vns(query).await {
                                            Ok(results) => {
                                                search_message.set(format!("Found {} result(s)", results.len()));
                                                search_results.set(results);
                                            }
                                            Err(error) => {
                                                search_message.set(format!("VNDB search failed: {error}"));
                                            }
                                        }
                                    });
                                }
                            },
                        }
                        button {
                            class: "vndb-search-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let query = vndb_query.read().trim().to_string();
                                if query.is_empty() {
                                    search_message.set("Enter a title or VNDB ID to search".to_string());
                                    return;
                                }
                                loaded_result_id.set(None);
                                search_message.set("Searching VNDB...".to_string());
                                spawn(async move {
                                    match search_vns(query).await {
                                        Ok(results) => {
                                            search_message.set(format!("Found {} result(s)", results.len()));
                                            search_results.set(results);
                                        }
                                        Err(error) => {
                                            search_message.set(format!("VNDB search failed: {error}"));
                                        }
                                    }
                                });
                            },
                            "Search"
                        }
                    }
                    div { class: "vndb-results-grid",
                        if results_to_show.is_empty() {
                            div { class: "vndb-empty-hint",
                                p { "Type a title and click Search to look up metadata and cover art on VNDB." }
                            }
                        }
                        for result in results_to_show {
                            {
                                let result_title = result.title.clone();
                                let result_description = result.description.clone();
                                let result_cover_url = result.image.clone().and_then(|image| image.url);
                                let is_sexual = result.image.as_ref().and_then(|img| img.sexual).unwrap_or(0.0) >= 1.0;
                                let is_nsfw = is_sexual || {
                                    let lower = result.title.to_lowercase();
                                    lower.contains("18+") || lower.contains("eroge") || lower.contains("nukige") || lower.contains("hentai") || lower.contains("sexual") || lower.contains("nsfw") || lower.contains("adult")
                                };
                                let cover_class = if blur_nsfw && is_nsfw { "vndb-result-cover nsfw-blur" } else { "vndb-result-cover" };
                                let result_id = result.id.clone();
                                let result_is_loaded = loaded_result_id.read().as_ref() == Some(&result.id);
                                let is_already_added = existing_vndb_ids.iter().any(|id| id.eq_ignore_ascii_case(&result.id))
                                    || existing_titles.iter().any(|t| t.trim().eq_ignore_ascii_case(result.title.trim()));
                                rsx! {
                                    article { class: "vndb-result-card",
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
                                        }
                                        div { class: "vndb-result-body",
                                            h3 { "{result.title}" }
                                            p { "VNDB ID: {result.id}" }
                                            button {
                                                class: if is_already_added {
                                                    "vndb-result-button in-library"
                                                } else if result_is_loaded {
                                                    "vndb-result-button loaded"
                                                } else {
                                                    "vndb-result-button"
                                                },
                                                onclick: move |_| {
                                                    if is_already_added {
                                                        search_message.set(format!("\"{}\" is already in your library.", result_title));
                                                        return;
                                                    }
                                                    title.set(result_title.clone());
                                                    description.set(result_description.clone().unwrap_or_default());
                                                    cover_url.set(result_cover_url.clone());
                                                    cover_path.set(None);
                                                    if is_sexual || is_nsfw {
                                                        let mut current = tags_text.read().clone();
                                                        if !current.contains("18+") {
                                                            if !current.trim().is_empty() && !current.trim_end().ends_with(',') {
                                                                current.push_str(", ");
                                                            }
                                                            current.push_str("18+");
                                                            tags_text.set(current);
                                                        }
                                                    }
                                                    loaded_result_id.set(Some(result_id.clone()));
                                                    search_message.set(format!("Loaded {}.", result_title));
                                                },
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

fn parse_tags(text: String) -> Vec<String> {
    text.split(",")
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}
