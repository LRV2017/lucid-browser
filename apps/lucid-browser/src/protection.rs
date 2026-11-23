//! A small first-party navigation protection list.
//!
//! This currently protects top-level navigation only. It is not a complete
//! EasyList/uBlock-compatible network request filter and does not inspect page
//! subresources, because the prototype renderer does not yet load subresources.
const BLOCKED_HOSTS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "adservice.google.com",
    "adnxs.com",
    "amazon-adsystem.com",
    "adsrvr.org",
    "taboola.com",
    "outbrain.com",
    "scorecardresearch.com",
    "adform.net",
    "adroll.com",
];

pub fn is_blocked_navigation(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else { return false };
    let Some(host) = parsed.host_str() else { return false };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    BLOCKED_HOSTS.iter().any(|blocked| {
        host == *blocked || host.ends_with(&format!(".{blocked}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_known_ad_hosts_and_subdomains() {
        assert!(is_blocked_navigation("https://doubleclick.net/path"));
        assert!(is_blocked_navigation("https://stats.adnxs.com/pixel"));
    }

    #[test]
    fn does_not_block_lookalike_or_normal_hosts() {
        assert!(!is_blocked_navigation("https://notdoubleclick.net"));
        assert!(!is_blocked_navigation("https://example.com"));
        assert!(!is_blocked_navigation("lucid://home"));
    }
}
