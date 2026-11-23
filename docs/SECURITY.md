# Lucid Security Principles

Lucid will process hostile network content. Security is a foundational requirement, not a final polish task.

## Required protections before general browsing

- Strict URL parsing and scheme allow-listing.
- TLS certificate verification; no silent downgrade or invalid-certificate bypass.
- Same-origin policy and origin-keyed storage.
- Renderer sandbox with least privilege.
- Separate network and privileged browser processes.
- Authenticated, validated IPC messages.
- Per-site permission prompts for camera, microphone, location, notifications and clipboard.
- Safe download handling and executable warnings.
- Content Security Policy processing and secure context rules.
- Memory-safe parsing and bounded resource consumption.
- Fuzzing, dependency auditing and security response process.

## Current prototype limitations

Normal HTTP/HTTPS tabs are rendered by the embedded WebView2 runtime. Lucid's independent engine now has an initial bounded HTTP(S) resource-loader module that can resolve and fetch referenced resources, but it is not yet connected to the visible page pipeline. The independent engine still does not execute page JavaScript, decode images/fonts/video, or provide a secure browsing environment. WebView2's platform security does not replace a security review of Lucid's own navigation, permissions, downloads, profile handling and process boundaries. Do not use experimental builds for sensitive browsing.

## Security reporting

Do not publish exploit details before maintainers can assess and coordinate a fix. A private security contact and formal disclosure policy must be configured before a public stable release.
