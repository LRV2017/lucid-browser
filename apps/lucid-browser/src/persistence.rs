//! Durable browser profile data. Writes are kept small and local to the user profile.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub visited_at_unix: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuickLink {
    pub label: String,
    pub subtitle: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedTab {
    pub title: String,
    pub url: String,
    pub workspace: String,
    pub pinned: bool,
    pub history: Vec<String>,
    pub history_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BrowserProfile {
    pub schema_version: u32,
    pub theme: String,
    pub accent: String,
    pub active_workspace: String,
    pub workspaces: Vec<String>,
    pub bookmarks: Vec<Bookmark>,
    pub history: Vec<HistoryEntry>,
    pub quick_links: Vec<QuickLink>,
    pub session_tabs: Vec<SavedTab>,
    pub bookmarks_bar: bool,
    pub memory_saver: bool,
    pub navigation_protection: bool,
    /// 0 means unlimited; otherwise a per-navigation HTML transfer cap in KiB/s.
    pub navigation_limit_kib: u32,
    pub animations_enabled: bool,
}

impl Default for BrowserProfile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            theme: "calm".into(),
            accent: "lavender".into(),
            active_workspace: "Personal".into(),
            workspaces: vec!["Personal".into(), "Gaming".into(), "School".into(), "Research".into()],
            bookmarks: Vec::new(),
            history: Vec::new(),
            quick_links: vec![
                QuickLink { label: "YouTube".into(), subtitle: "Video".into(), url: "https://www.youtube.com".into() },
                QuickLink { label: "GitHub".into(), subtitle: "Development".into(), url: "https://github.com".into() },
                QuickLink { label: "Reddit".into(), subtitle: "Community".into(), url: "https://www.reddit.com".into() },
                QuickLink { label: "Wikipedia".into(), subtitle: "Knowledge".into(), url: "https://www.wikipedia.org".into() },
            ],
            session_tabs: Vec::new(),
            bookmarks_bar: false,
            memory_saver: true,
            navigation_protection: true,
            navigation_limit_kib: 0,
            animations_enabled: true,
        }
    }
}

impl BrowserProfile {
    pub fn load() -> Self {
        let path = profile_path();
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = profile_path();
        let parent = path.parent().ok_or_else(|| "Invalid Lucid profile path".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| format!("Could not create profile folder: {error}"))?;

        let mut snapshot = self.clone();
        snapshot.schema_version = 1;
        snapshot.history.truncate(500);
        snapshot.session_tabs.truncate(30);
        let json = serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| format!("Could not serialize profile: {error}"))?;

        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, json).map_err(|error| format!("Could not write profile: {error}"))?;

        // Windows rename does not replace an existing target. Replace after the
        // complete temporary file has been written to avoid partially written JSON.
        if path.exists() {
            fs::remove_file(&path).map_err(|error| format!("Could not replace profile: {error}"))?;
        }
        fs::rename(&temp_path, &path).map_err(|error| format!("Could not finish profile save: {error}"))
    }

    pub fn add_history(&mut self, title: String, url: String) {
        if url == "lucid://home" || url == "lucid://demo" {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        if self.history.last().is_some_and(|entry| entry.url == url) {
            if let Some(last) = self.history.last_mut() {
                last.title = title;
                last.visited_at_unix = now;
            }
            return;
        }
        self.history.push(HistoryEntry { title, url, visited_at_unix: now });
        if self.history.len() > 500 {
            let excess = self.history.len() - 500;
            self.history.drain(0..excess);
        }
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|bookmark| bookmark.url == url)
    }

    pub fn toggle_bookmark(&mut self, title: String, url: String) {
        if let Some(index) = self.bookmarks.iter().position(|bookmark| bookmark.url == url) {
            self.bookmarks.remove(index);
        } else {
            self.bookmarks.push(Bookmark { title, url });
        }
    }
}

pub fn profile_path() -> PathBuf {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        return PathBuf::from(app_data).join("Lucid").join("profile.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("lucid").join("profile.json");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        .join("lucid-profile.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_has_expected_workspaces_and_quick_links() {
        let profile = BrowserProfile::default();
        assert!(profile.workspaces.iter().any(|workspace| workspace == "Gaming"));
        assert!(profile.quick_links.iter().any(|link| link.label == "GitHub"));
        assert!(profile.memory_saver);
    }

    #[test]
    fn bookmark_toggle_adds_and_removes() {
        let mut profile = BrowserProfile::default();
        profile.toggle_bookmark("Example".into(), "https://example.com".into());
        assert!(profile.is_bookmarked("https://example.com"));
        profile.toggle_bookmark("Example".into(), "https://example.com".into());
        assert!(!profile.is_bookmarked("https://example.com"));
    }

    #[test]
    fn history_is_bounded() {
        let mut profile = BrowserProfile::default();
        for index in 0..510 {
            profile.add_history(format!("Page {index}"), format!("https://example.com/{index}"));
        }
        assert_eq!(profile.history.len(), 500);
    }
}
