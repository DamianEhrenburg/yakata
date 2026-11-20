use crate::models::VisualNovel;
use base64::Engine;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, OnceLock};

#[component]
pub fn LibraryView(
    vns: Vec<VisualNovel>,
    total_vns_count: usize,
    selected_vn_id: Option<u64>,
    blur_nsfw: bool,
    on_select: EventHandler<u64>,
    on_toggle_favourite: EventHandler<u64>,
    on_open_add_form: EventHandler<()>,
    on_reset_filters: EventHandler<()>,
) -> Element {
    rsx! {
        section { class: "library-section",
            div { class: "library-heading",
                h2 { "Library" }
                span { class: "library-count", "{vns.len()}" }
            }
            if total_vns_count == 0 {
                div { class: "library-empty-state",
                    h3 { "No visual novels yet" }
                    p { "Add an entry manually or search VNDB." }
                    button {
                        class: "library-empty-button",
                        onclick: move |_| on_open_add_form.call(()),
                        "+ Add visual novel"
                    }
                }
            } else if vns.is_empty() {
                div { class: "library-empty-state",
                    h3 { "No visual novels found" }
                    p { "No visual novels match the active search or filters." }
                    button {
                        class: "library-empty-button",
                        onclick: move |_| on_reset_filters.call(()),
                        "Reset filters"
                    }
                }
            } else {
                div { class: "vn-grid",
                    for vn in vns {
                        {
                            let vn_id = vn.id;
                            let card_class = if selected_vn_id == Some(vn_id) {
                                "vn-card selected"
                            } else {
                                "vn-card"
                            };
                            let favourite_button_class = if vn.is_favourite {
                                "favourite-button active"
                            } else {
                                "favourite-button"
                            };
                            let favourite_label = if vn.is_favourite {
                                "Remove from favourites"
                            } else {
                                "Add to favourites"
                            };
                            rsx! {
                                article {
                                    class: "{card_class}",
                                    onclick: move |_| {
                                        on_select.call(vn.id);
                                    },
                                    button {
                                        class: "{favourite_button_class}",
                                        title: "{favourite_label}",
                                        aria_label: "{favourite_label}",
                                        onclick: move |event| {
                                            event.stop_propagation();
                                            on_toggle_favourite.call(vn_id);
                                        },
                                        if vn.is_favourite {
                                            "★"
                                        } else {
                                            "☆"
                                        }
                                    }
                                    div { class: "vn-cover-frame",
                                        VnCover { vn: vn.clone(), blur: blur_nsfw && vn.effective_nsfw() }
                                        if let Some(rating) = vn.rating {
                                            span { class: "vn-rating-badge", "★ {rating}" }
                                        }
                                    }
                                    div { class: "vn-card-info",
                                        h3 { title: "{vn.title}", "{vn.title}" }
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

#[component]
fn VnCover(vn: VisualNovel, blur: bool) -> Element {
    let mut image_failed = use_signal(|| false);
    let has_failed = *image_failed.read();

    let title = vn.title.clone();
    let cover_src = cover_source(vn);
    let cover_class = if blur { "vn-cover nsfw-blur" } else { "vn-cover" };

    rsx! {
        if has_failed {
            div { class: "vn-cover-placeholder", "No cover" }
        } else if let Some(src) = cover_src {
            img {
                class: "{cover_class}",
                src: "{src}",
                alt: "Cover art for {title}",
                onerror: move |_| {
                    image_failed.set(true);
                },
            }
        } else {
            div { class: "vn-cover-placeholder", "No cover" }
        }
    }
}

pub fn is_vn_nsfw(vn: &VisualNovel) -> bool {
    vn.effective_nsfw()
}

pub fn cover_source(vn: VisualNovel) -> Option<String> {
    if let Some(path) = vn.cover_path
        && let Some(data_url) = local_cover_source(path)
    {
        return Some(data_url);
    }
    vn.cover_url
}

fn local_cover_source(path: String) -> Option<String> {
    local_path_to_data_url(&path)
}

fn data_url_cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn invalidate_cover_cache(path: &str) {
    if let Ok(mut cache) = data_url_cache().lock() {
        cache.remove(path);
    }
}

pub fn local_path_to_data_url(path: &str) -> Option<String> {
    if let Ok(cache) = data_url_cache().lock()
        && let Some(cached) = cache.get(path)
    {
        return Some(cached.clone());
    }
    let image_bytes = fs::read(path).ok()?;
    let mime_type = image_mime_type(path);
    let encoded_image = base64::engine::general_purpose::STANDARD.encode(image_bytes);
    let data_url = format!("data:{mime_type};base64,{encoded_image}");
    if let Ok(mut cache) = data_url_cache().lock() {
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(path.to_string(), data_url.clone());
    }
    Some(data_url)
}

fn image_mime_type(path: &str) -> &'static str {
    let lower_path = path.to_lowercase();
    if lower_path.ends_with(".png") {
        "image/png"
    } else if lower_path.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_vn_nsfw() {
        let clean = VisualNovel {
            title: "Young Adult Story".to_string(),
            description: Some("A young adult drama about growing up.".to_string()),
            tags: vec!["Drama".to_string(), "Romance".to_string()],
            ..Default::default()
        };
        assert!(!is_vn_nsfw(&clean));

        let flag_nsfw = VisualNovel { is_nsfw: true, ..Default::default() };
        assert!(is_vn_nsfw(&flag_nsfw));

        let tag_nsfw = VisualNovel { tags: vec!["Eroge".to_string()], ..Default::default() };
        assert!(is_vn_nsfw(&tag_nsfw));

        let title_nsfw = VisualNovel { title: "Title 18+".to_string(), ..Default::default() };
        assert!(is_vn_nsfw(&title_nsfw));
    }
}
