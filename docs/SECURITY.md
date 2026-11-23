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

The current prototype does not fetch remote pages, execute scripts, access local files through page content, or provide a secure browsing environment. It is a native UI shell and engine research project. Do not use it for sensitive browsing.

## Security reporting

Do not publish exploit details before maintainers can assess and coordinate a fix. A private security contact and formal disclosure policy must be configured before a public stable release.
