//! Bounded, user-initiated file downloads.
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

pub const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub enum DownloadUpdate {
    Progress { id: u64, received: u64, total: Option<u64> },
    Completed { id: u64, path: PathBuf, bytes: u64 },
    Failed { id: u64, message: String },
}

pub fn run_download(id: u64, url: String, updates: Sender<DownloadUpdate>) {
    let result = download_to_disk(id, &url, &updates);
    if let Err(message) = result {
        let _ = updates.send(DownloadUpdate::Failed { id, message });
    }
}

fn download_to_disk(id: u64, url: &str, updates: &Sender<DownloadUpdate>) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "Enter a valid HTTP or HTTPS URL.".to_owned())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Downloads are only allowed from HTTP and HTTPS URLs.".into());
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("LucidBrowser/0.0.3")
        .timeout(Duration::from_secs(600))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|error| format!("Could not initialize download: {error}"))?;

    let mut response = client.get(parsed.clone()).send()
        .map_err(|error| format!("Download request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Server returned {}", response.status()));
    }

    let total = response.content_length();
    if total.is_some_and(|size| size > MAX_DOWNLOAD_BYTES) {
        return Err("This prototype limits individual downloads to 512 MiB.".into());
    }

    let directory = download_directory();
    fs::create_dir_all(&directory).map_err(|error| format!("Could not create download folder: {error}"))?;
    let filename = safe_filename(parsed.path_segments().and_then(|segments| segments.filter(|s| !s.is_empty()).next_back()).unwrap_or("download.bin"));
    let destination = unique_path(&directory, &filename);
    let partial = destination.with_extension(format!("{}.part", destination.extension().and_then(|e| e.to_str()).unwrap_or("download")));
    let mut file = File::create(&partial).map_err(|error| format!("Could not create download file: {error}"))?;

    let mut received = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = response.read(&mut buffer).map_err(|error| format!("Download interrupted: {error}"))?;
        if count == 0 { break; }
        received = received.saturating_add(count as u64);
        if received > MAX_DOWNLOAD_BYTES {
            let _ = fs::remove_file(&partial);
            return Err("Download exceeded the 512 MiB safety limit.".into());
        }
        file.write_all(&buffer[..count]).map_err(|error| format!("Could not write download: {error}"))?;
        let _ = updates.send(DownloadUpdate::Progress { id, received, total });
    }
    file.flush().map_err(|error| format!("Could not finish writing download: {error}"))?;
    drop(file);
    fs::rename(&partial, &destination).map_err(|error| format!("Could not finalize download: {error}"))?;
    let _ = updates.send(DownloadUpdate::Completed { id, path: destination, bytes: received });
    Ok(())
}

fn download_directory() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    home.join("Downloads").join("Lucid")
}

fn safe_filename(input: &str) -> String {
    let filtered: String = input.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ' ') { ch } else { '_' })
        .collect();
    let trimmed = filtered.trim_matches(|ch: char| ch == '.' || ch == ' ');
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." { "download.bin".into() }
    else { trimmed.chars().take(120).collect() }
}

fn unique_path(directory: &Path, filename: &str) -> PathBuf {
    let first = directory.join(filename);
    if !first.exists() { return first; }
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("download");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("bin");
    for index in 1..10_000 {
        let candidate = directory.join(format!("{stem} ({index}).{extension}"));
        if !candidate.exists() { return candidate; }
    }
    directory.join(format!("download-{id_placeholder()}.bin"))
}

fn id_placeholder() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_micros() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_is_sanitized() {
        assert_eq!(safe_filename("../../secret?.zip"), "___secret_.zip");
        assert_eq!(safe_filename(".."), "download.bin");
    }

    #[test]
    fn download_directory_is_inside_user_downloads() {
        assert!(download_directory().ends_with(Path::new("Downloads").join("Lucid")));
    }
}
