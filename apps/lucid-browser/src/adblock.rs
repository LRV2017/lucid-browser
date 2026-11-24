//! Built-in network ad and tracker filtering for Chromium requests.
//!
//! The filter engine uses the same adblock-rust library used by Brave. This module currently
//! applies network filters; cosmetic filtering and scriptlet injection are separate work.

use adblock::{Engine, FilterSet, lists::ParseOptions, request::Request as AdBlockRequest};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

const BUILTIN_FILTERS: &str = r#"
||doubleclick.net^
||googlesyndication.com^
||googleadservices.com^
||adservice.google.com^
||adnxs.com^
||taboola.com^
||outbrain.com^
||scorecardresearch.com^
||criteo.com^
||adform.net^
||tracking-server.com^
||analytics.yahoo.com^
"#;

const FILTER_LISTS: [(&str, &str); 3] = [
    ("easylist.txt", "https://easylist.to/easylist/easylist.txt"),
    ("easyprivacy.txt", "https://easylist.to/easylist/easyprivacy.txt"),
    ("ublock-filters.txt", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt"),
];

#[derive(Clone)]
pub struct AdBlockService {
    engine: Arc<RwLock<Arc<Engine>>>,
    enabled: Arc<AtomicBool>,
    blocked: Arc<AtomicU64>,
}

impl AdBlockService {
    pub fn new() -> Self {
        let service = Self {
            engine: Arc::new(RwLock::new(Arc::new(build_engine(&[])))),
            enabled: Arc::new(AtomicBool::new(true)),
            blocked: Arc::new(AtomicU64::new(0)),
        };

        // Load cached lists and refresh the lists away from the UI thread.
        let updater = service.clone();
        thread::spawn(move || {
            let cache_dir = filter_cache_directory();
            let _ = fs::create_dir_all(&cache_dir);
            let mut lists = Vec::new();

            let client = reqwest::blocking::Client::builder()
                .user_agent("LucidBrowser/0.0.2 (built-in content blocker)")
                .timeout(Duration::from_secs(25))
                .build();

            for (file_name, url) in FILTER_LISTS {
                let cached_path = cache_dir.join(file_name);
                if let Ok(cached) = fs::read_to_string(&cached_path) {
                    if !cached.trim().is_empty() {
                        lists.push(cached);
                    }
                }

                if let Ok(client) = &client {
                    if let Ok(response) = client.get(url).send().and_then(|response| response.error_for_status()) {
                        if let Ok(text) = response.text() {
                            if !text.trim().is_empty() {
                                let _ = fs::write(&cached_path, &text);
                                lists.push(text);
                            }
                        }
                    }
                }
            }

            if !lists.is_empty() {
                let replacement = Arc::new(build_engine(&lists));
                if let Ok(mut current) = updater.engine.write() {
                    *current = replacement;
                }
            }
        });

        service
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn blocked_count(&self) -> u64 {
        self.blocked.load(Ordering::Relaxed)
    }

    pub fn should_block(&self, url: &str, source_url: &str, request_type: &str, method: &str) -> bool {
        if !self.is_enabled() || !(url.starts_with("http://") || url.starts_with("https://")) {
            return false;
        }

        let source = if source_url.is_empty() { url } else { source_url };
        let Ok(request) = AdBlockRequest::new(url, source, request_type, method) else {
            return false;
        };

        let blocked = self.engine.read()
            .map(|engine| engine.check_network_request(&request).should_block())
            .unwrap_or(false);

        if blocked {
            self.blocked.fetch_add(1, Ordering::Relaxed);
        }
        blocked
    }
}

impl Default for AdBlockService {
    fn default() -> Self {
        Self::new()
    }
}

fn build_engine(lists: &[String]) -> Engine {
    let mut filter_set = FilterSet::new(false);
    filter_set.add_filter_list(BUILTIN_FILTERS.to_owned(), ParseOptions::default());
    for list in lists {
        filter_set.add_filter_list(list.clone(), ParseOptions::default());
    }
    Engine::new_with_filter_set(filter_set)
}

fn filter_cache_directory() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Lucid")
        .join("FilterLists")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_known_advertising_hosts_but_allows_first_party_content() {
        let service = AdBlockService {
            engine: Arc::new(RwLock::new(Arc::new(build_engine(&[])))),
            enabled: Arc::new(AtomicBool::new(true)),
            blocked: Arc::new(AtomicU64::new(0)),
        };
        assert!(service.should_block(
            "https://doubleclick.net/ads.js",
            "https://example.com/",
            "script",
            "GET",
        ));
        assert!(!service.should_block(
            "https://example.com/app.js",
            "https://example.com/",
            "script",
            "GET",
        ));
        assert_eq!(service.blocked_count(), 1);
    }

    #[test]
    fn disabling_the_blocker_allows_requests() {
        let service = AdBlockService::new();
        service.set_enabled(false);
        assert!(!service.should_block(
            "https://doubleclick.net/ads.js",
            "https://example.com/",
            "script",
            "GET",
        ));
    }
}
