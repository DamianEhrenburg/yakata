use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbTag {
    pub name: String,
    #[serde(default)]
    pub rating: f32,
    #[serde(default)]
    pub spoiler: u8,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbSearchResult {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub description: Option<String>,
    pub released: Option<String>,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub tags: Vec<VndbTag>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbImage {
    pub url: Option<String>,
    #[serde(default)]
    pub sexual: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VndbSortOrder {
    SearchRank,
    Rating,
    Released,
    Title,
}

impl VndbSortOrder {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::SearchRank => "searchrank",
            Self::Rating => "rating",
            Self::Released => "released",
            Self::Title => "title",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct VndbSearchResponse {
    pub results: Vec<VndbSearchResult>,
}

#[derive(Debug, Clone, Serialize)]
struct VndbSearchRequest {
    pub filters: serde_json::Value,
    pub fields: String,
    pub sort: String,
    pub reverse: bool,
    pub results: u32,
}

pub async fn search_vns(
    query: String,
    sort: VndbSortOrder,
    reverse: bool,
) -> Result<Vec<VndbSearchResult>, reqwest::Error> {
    let client = reqwest::Client::new();
    let request_body = VndbSearchRequest {
        filters: serde_json::json!(["search", "=", query]),
        fields: "id, title, image.url, image.sexual, description, released, rating, tags.name, tags.rating, tags.spoiler, tags.category".to_string(),
        sort: sort.as_api_str().to_string(),
        reverse,
        results: 20,
    };
    let response = client
        .post("https://api.vndb.org/kana/vn")
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json::<VndbSearchResponse>()
        .await?;
    Ok(response.results)
}

pub fn filter_vndb_tags(tags: &[VndbTag], is_nsfw: bool) -> Vec<String> {
    const SPOILER_STOPWORDS: &[&str] = &[
        "death of",
        "murder of",
        "suicide of",
        "unavoidable",
        "killer is",
        "identity of",
        "true nature of",
        "betrayal",
        "mass murder",
        "massacre",
        "mind control",
    ];

    const GENRE_KEYWORDS: &[&str] = &[
        "science fiction",
        "sci-fi",
        "time travel",
        "mystery",
        "psychological",
        "thriller",
        "horror",
        "romance",
        "fantasy",
        "drama",
        "comedy",
        "dystopia",
        "cyberpunk",
        "action",
        "adventure",
        "detective",
        "school life",
        "supernatural",
        "post-apocalyptic",
        "historical",
        "slice of life",
        "philosophy",
        "philosophical",
        "gothic",
        "urban fantasy",
        "magic",
        "crime",
        "tragedy",
        "steampunk",
        "mecha",
        "dark fantasy",
        "space",
        "conspiracy",
        "time loop",
        "amnesia",
        "survival",
    ];

    let mut eligible: Vec<VndbTag> = tags
        .iter()
        .filter(|t| {
            if t.spoiler > 0 {
                return false;
            }
            if t.category.as_deref() == Some("tech") {
                return false;
            }
            if t.category.as_deref() == Some("ero") {
                return false;
            }
            if t.rating < 1.5 {
                return false;
            }
            let lower = t.name.to_lowercase();
            if SPOILER_STOPWORDS.iter().any(|&stop| lower.contains(stop)) {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    eligible.sort_by(|a, b| {
        let a_lower = a.name.to_lowercase();
        let b_lower = b.name.to_lowercase();
        let a_is_genre = GENRE_KEYWORDS.iter().any(|&g| a_lower.contains(g));
        let b_is_genre = GENRE_KEYWORDS.iter().any(|&g| b_lower.contains(g));

        match (a_is_genre, b_is_genre) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    let mut top_tags: Vec<String> = eligible
        .into_iter()
        .take(5)
        .map(|t| t.name)
        .collect();

    if is_nsfw && !top_tags.iter().any(|t| t == "18+") {
        top_tags.push("18+".to_string());
    }

    top_tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_vndb_tags_excludes_tech_and_spoilers() {
        let tags = vec![
            VndbTag {
                name: "Branching Plot".to_string(),
                rating: 3.0,
                spoiler: 0,
                category: Some("tech".to_string()),
            },
            VndbTag {
                name: "Death of Heroine".to_string(),
                rating: 2.8,
                spoiler: 0,
                category: Some("cont".to_string()),
            },
            VndbTag {
                name: "Time Travel".to_string(),
                rating: 2.9,
                spoiler: 0,
                category: Some("cont".to_string()),
            },
            VndbTag {
                name: "Science Fiction".to_string(),
                rating: 2.4,
                spoiler: 0,
                category: Some("cont".to_string()),
            },
            VndbTag {
                name: "Secret Society".to_string(),
                rating: 2.4,
                spoiler: 1,
                category: Some("cont".to_string()),
            },
            VndbTag {
                name: "Scientist Heroine".to_string(),
                rating: 2.7,
                spoiler: 0,
                category: Some("cont".to_string()),
            },
        ];

        let filtered = filter_vndb_tags(&tags, false);
        assert!(!filtered.contains(&"Branching Plot".to_string()));
        assert!(!filtered.contains(&"Death of Heroine".to_string()));
        assert!(!filtered.contains(&"Secret Society".to_string()));
        assert_eq!(filtered[0], "Time Travel");
        assert_eq!(filtered[1], "Science Fiction");
        assert!(filtered.contains(&"Scientist Heroine".to_string()));
    }

    #[test]
    fn test_filter_vndb_tags_appends_18_plus_when_nsfw() {
        let tags = vec![VndbTag {
            name: "Horror".to_string(),
            rating: 2.8,
            spoiler: 0,
            category: Some("cont".to_string()),
        }];
        let filtered = filter_vndb_tags(&tags, true);
        assert_eq!(filtered, vec!["Horror", "18+"]);
    }
}
