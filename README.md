# Lucid Browser

A Windows desktop browser with a premium, customizable Lucid interface and a live Chromium-based website renderer.

## How website rendering works

Lucid uses **Microsoft Edge WebView2** through the Rust Wry host for normal websites. WebView2 is Microsoft's embeddable Chromium-based web platform. It renders real pages with their own HTML, CSS, JavaScript, images, fonts, SVG, canvas, video, iframes, network requests, cookies and browser storage. Those web standards are implemented by the WebView2 runtime, not by Lucid's small experimental renderer.

This is a deliberate engineering trade-off: it makes ordinary websites work as actual websites instead of drawing a simplified imitation of their HTML. It also means the current live renderer is the Edge/Chromium engine, **not a fully independent Lucid engine and not the Google Chrome application itself**. Rendering and compatibility should be close to current Chromium-based Edge, but exact Chrome behavior, extensions and site compatibility still need testing.

Lucid's original renderer remains in `crates/lucid-engine` and can be tried at `lucid://demo`. It is an independent research implementation, not yet a general-purpose web engine.

> Current development line: 0.0.4 Experimental. The WebView2 integration is being validated by the Windows CI build. Do not treat a successful compile as proof of Chrome-level compatibility or browser-grade security.

## Lucid features

### Interface
- Lucid Calm: clean graphite/lavender appearance, quick links, workspace dashboard, sidebar and restrained motion.
- Lucid Cyberpunk: separate tactical player-HUD-inspired theme with angular panels and magenta/cyan accents.
- Theme switching, accent packs and reduced-motion preference.

### Browser shell
- Native tabs and workspaces.
- A separate live WebView2 page surface for each tab; inactive views are hidden rather than replaced with screenshots.
- Back/forward, reload, address/search navigation and persistent session metadata.
- `target="_blank"` and `window.open()` requests are routed into new Lucid tabs.
- Shared Lucid WebView2 profile for site cookies and storage.
- DevTools support in the embedded web renderer.
- Basic in-page find using the page's browser DOM.
- Bookmarks, browsing history, downloads, pinned tabs, Memory Saver and command palette.

### Power-user foundations
- GX-style Control Center with Lucid-process CPU/memory and system memory summary.
- Mod Center with themes, accent packs and motion preference.
- Lucid Corner gaming shortcut hub.

These are first-party foundations, not full Opera GX equivalents. CPU/memory figures are not per tab; Lucid Protect is still a small top-level navigation list, not an uBlock-style request filter engine; and Lucid Corner is not a live gaming news feed. The WebView2 extension capability is enabled, but Lucid does not yet have a complete extension-store or Firefox XPI compatibility layer.

## Windows runtime requirement

Lucid's website renderer needs the **Microsoft Edge WebView2 Runtime**. It is included with Windows 11 on most systems. If Lucid reports that the runtime is missing, install Microsoft's Evergreen Runtime and relaunch Lucid:

- [WebView2 Runtime download](https://developer.microsoft.com/microsoft-edge/webview2/)
- [Microsoft runtime deployment guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution)

The portable ZIP does not bundle the 250+ MB Fixed Version runtime. The Evergreen Runtime is independently serviced by Microsoft.

## Download the Windows executable without installing Rust

1. Open GitHub Actions: https://github.com/LRV2017/lucid-browser/actions
2. Select the latest successful **Build Windows** run.
3. Download the `Lucid-Browser-Windows-x64` artifact.
4. Extract the ZIP and launch `lucid-browser.exe`.

GitHub builds Lucid on a hosted Windows machine. You do not need Rust, Cargo, Visual Studio or build tools installed locally.

## Try the original Lucid renderer

Launch Lucid and enter `lucid://demo`. The demo is parsed, styled, laid out and painted by Lucid's own experimental HTML/CSS code. This is intentionally separate from normal live website browsing.

## Architecture

```text
Lucid Browser (Rust / egui desktop shell)
  ├─ tabs, workspaces, toolbar, profile, downloads
  ├─ Wry native host
  │    └─ Microsoft Edge WebView2 Runtime (Chromium)
  │         └─ normal live websites
  └─ Lucid Engine (independent research path)
       └─ lucid://demo only for now
```

See `docs/architecture.md`, `docs/product-quality-spec.md`, `docs/design-system.md`, `docs/roadmap.md` and `STATUS.md`.

## Security and compatibility

The WebView2 runtime supplies a mature Chromium-based web platform, but Lucid is still an experimental browser application. Its own permission UX, download protections, ad/tracker filtering, extension management, crash recovery, privacy controls and security configuration need more work and audit. Do not use early builds for sensitive accounts, payments or security-critical browsing.

Lucid is **not a Google Chrome clone**. It currently relies on the Edge WebView2 runtime; Chrome-specific features and policies, DRM/Widevine behavior, Google account integration, extension-store flows and every individual website have not been verified. The independent Lucid renderer is not standards-complete.

A successful compile proves only that the code builds and its tests pass. It does not establish broad compatibility, security or performance parity.

## Build locally (optional)

Install Rust and the Windows C++ build tools, then run `cargo test --workspace --all-targets` and `cargo build --release --package lucid-browser`.

## License

Lucid source is MIT. Third-party runtime and dependency terms are described in `THIRD_PARTY_NOTICES.md`.
