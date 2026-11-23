# Project Status — 0.0.3 Experimental

Lucid is an independent browser-engine research project and native desktop application.

## Implemented in the 0.0.3 UI and service layer

- Native desktop UI with Lucid Calm and Lucid Cyberpunk.
- Persistent user profile with theme/accent, workspaces, bookmarks, browsing history, quick links, tab session metadata and preferences.
- Tabs, workspace switching, back/forward, address/search entry and command palette.
- Pinned tabs and manual background-tab sleeping.
- Library and history panels.
- GX-style Control Center with live main-process CPU/RAM and system memory summary.
- Per-top-level-HTML-navigation bandwidth governor.
- First-party top-level host navigation protection list.
- Accent/theme Mod Center and reduced-motion preference.
- Gaming shortcut hub (not a live news/deals feed).
- Direct HTTP/HTTPS download queue (512 MiB per-file cap).
- Basic anchor href hit-testing and relative URL navigation.
- GitHub Actions Windows build and portable ZIP artifact.

## Engine behavior implemented

- Lucid-owned HTML tokenizer and small tree builder.
- Raw-text preservation for style/script; scripts remain inert.
- Basic CSS selector/declaration parsing and a small cascade.
- Simple top-to-bottom block layout and paint operations.
- HTTP/HTTPS HTML fetch with bounded document size; arbitrary local-file URL access is disabled.
- Per-tab navigation generation tracking to discard stale responses.

## Explicit limitations

- No JavaScript execution or DOM host bindings.
- Not full WHATWG HTML parsing or complete CSS support.
- No inline formatting, correct line boxes, flexbox, grid, floats, standards-compliant positioning, full text shaping or compositing.
- No images, SVG, canvas, audio/video, WebAssembly, workers, service workers or subresource loading.
- No cookies, storage model, CSP, CORS, complete origin model or certificate UI.
- No site isolation, renderer sandbox, multi-process architecture, per-tab process accounting or independent security audit.
- Lucid Protect only matches a small top-level hostname list, not page requests or filter-list syntax.
- Network governor limits each top-level HTML transfer, not all browser traffic.
- Memory Saver releases this prototype's in-memory document object; it does not suspend a renderer process.
- Mod Center changes themes/accents/motion, not third-party executable mods.
- GX Corner is a shortcut page, not live curated gaming content.
- Download manager supports direct HTTP/HTTPS file URLs, not browser download interception, pause/resume or safe-browsing scanning.
- No third-party extensions or Chrome/Firefox WebExtensions compatibility.
- No broad modern website compatibility, accessibility tree or screen-reader integration.

This build is experimental. Do not use it for sensitive browsing.
