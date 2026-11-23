# Project Status — 0.0.5 Native Renderer Prototype

Lucid is a Windows desktop browser with a simplified native interface and a Lucid-owned page rendering pipeline. **Wry and Microsoft Edge WebView2 have been removed from the application dependencies and normal navigation code.**

## Native page pipeline currently implemented

- Bounded HTTP/HTTPS document fetching with redirect and timeout limits.
- Lucid-owned basic HTML tokenizer and document tree.
- Discovery and fetching of common linked stylesheets and HTML resources.
- Basic CSS parser/cascade applied to inline and linked stylesheet rules.
- Simple vertical block layout and text/shape paint operations.
- Raster image decoding for PNG, JPEG, GIF, WebP and BMP into native egui textures.
- Asynchronous page-load worker threads and per-tab navigation-generation checks.
- Native egui page painting; no embedded WebView surface.

This is a working foundation, not a complete modern browser engine.

## Interface

- Lucid Calm is the default clean interface: compact header, native tabs, a single address/search bar, bookmark action and a small menu for secondary tools.
- Library, history, downloads, performance, appearance and settings are no longer pinned to a permanent vertical sidebar.
- Lucid Cyberpunk remains available as an alternate home-screen/interface theme.
- Bookmarks, history, workspaces, session metadata and downloads remain in the app.

## Not yet implemented

- JavaScript engine and browser DOM bindings; page scripts remain inert.
- Complete HTML Living Standard tree construction and DOM mutation/events.
- Standards-complete CSS selectors, cascade, variables, flexbox, grid, inline layout, positioning and painting.
- Web fonts, font shaping and downloadable font faces.
- SVG, canvas, iframe, embedded documents and CSS background images.
- Audio/video decoding, streaming and playback controls.
- Cookies, local/session storage, IndexedDB, service workers, WebSockets, WebRTC and other browser APIs.
- Accessibility tree, developer tools, extension APIs, multi-process renderer, sandboxing and site isolation.
- Full request-level tracker/ad filtering and mature safe-browsing/download protections.

## Compatibility

Lucid does not use WebView2 and does not require its runtime. The trade-off is that the independent renderer is currently far less compatible with modern websites than Chromium/WebView2. Complex, script-heavy sites are expected to fail or render incompletely until the missing engine subsystems are implemented.

## Security

Lucid processes hostile remote HTML and resource data in its own application process. There is no production-grade renderer sandbox or site isolation yet, and the application has not been independently security audited. Do not use early builds for sensitive accounts, payments or security-critical browsing. Every advertised capability must be backed by implementation and tests; a successful build alone does not prove browser compatibility or security.
