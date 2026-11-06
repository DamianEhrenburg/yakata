use crate::storage::yakata_data_dir;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn cover_cache_dir() -> Result<PathBuf, io::Error> {
    let cache_dir = yakata_data_dir()?.join("covers");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

pub async fn cache_cover_image(
    vn_id: u64,
    cover_url: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let cache_dir = cover_cache_dir()?;
    let cover_file = cache_dir.join(format!("{vn_id}.jpg"));
    let image_bytes = reqwest::get(cover_url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    fs::write(&cover_file, image_bytes)?;
    Ok(cover_file.to_string_lossy().to_string())
}
