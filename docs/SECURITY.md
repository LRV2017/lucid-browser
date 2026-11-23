# Lucid Security Principles

Lucid parses and paints untrusted network content directly. Security is a foundational requirement, not a final polish task.

## Required protections before general browsing

- Strict URL parsing, scheme allow-listing and safe redirect handling.
- TLS certificate verification and safe certificate-error behavior.
- Same-origin policy and origin-keyed cookies/storage.
- Sandboxed renderer with least privilege and process separation.
- Authenticated, validated IPC messages between browser and renderer.
- Per-site permission prompts for camera, microphone, location, notifications and clipboard.
- Safe download handling, file-type inspection and executable warnings.
- Content Security Policy, CORS, mixed-content and secure-context enforcement.
- Memory-safe parsers and bounded resource consumption.
- Fuzzing, dependency auditing, crash handling and a security response process.

## Current prototype limitations

Lucid no longer delegates page rendering to WebView2. Its own renderer currently fetches HTTP/HTTPS documents and selected resources, parses a limited HTML subset, applies a small CSS subset, decodes supported raster images and paints native egui operations.

The renderer currently runs in the browser application process. It has no production-grade renderer sandbox, site isolation, full origin policy, cookie/storage security model, JavaScript execution security model, mature permission manager or independent security audit. The bounded resource loader is not a substitute for complete hostile-input protections.

Do not use experimental Lucid builds for sensitive browsing, payments or security-critical accounts.

## Security reporting

Do not publish exploit details before maintainers can assess and coordinate a fix. A private security contact and formal disclosure policy must be configured before a public stable release.
