# Lucid Browser

An independent browser-engine project with a premium, interactive desktop experience.

Lucid is being developed as an original HTML/CSS rendering engine and native desktop browser. It does not embed Firefox, Chromium, Blink, WebKit, Electron, CEF or WebView2.

> Current release: 0.0.3 Experimental. The browser UI has grown substantially, but the rendering engine remains a limited HTML/CSS prototype. It does not execute JavaScript and is not yet suitable for everyday or sensitive browsing.

## Implemented in 0.0.3

### Lucid interface
- Lucid Calm: clean graphite/lavender appearance, optional workspace dashboard, quick links, sidebar, command palette and restrained animation.
- Lucid Cyberpunk: separate tactical terminal/player-HUD-inspired interface with angular panels and magenta/cyan system accents.
- Theme switching without changing the underlying browser session.
- Accent packs, reduced-motion setting and bookmarks-bar preference.

### Browser features
- Tabs, separate workspaces, back/forward and address/search navigation.
- Persistent local profile under %APPDATA%\Lucid\profile.json on Windows.
- Session metadata restoration, saved bookmarks and browsing history.
- Pinned tabs and manual background-tab sleeping.
- Library, History, Settings and command palette panels.
- Direct HTTP/HTTPS download queue saved to Downloads\Lucid (512 MiB per-file prototype cap).
- Clickable anchor text with relative HTTP/HTTPS URL resolution.

### Gaming / power-user foundations
- GX-style Control Center with live Lucid-process CPU and memory, plus system RAM summary.
- Manual Memory Saver (drops in-memory parsed documents for unpinned background tabs; restoring reloads the page).
- Configurable bandwidth cap for each top-level HTML navigation.
- First-party top-level navigation hostname list.
- Mod Center with themes, accent packs and motion preference.
- Lucid Corner gaming shortcut hub.

These are first-party foundations, not full Opera GX equivalents. CPU/memory figures are currently per-process, not per-tab; the navigation list does not filter page subresources; and Lucid Corner shortcuts are not a live gaming news feed.

## Download the Windows executable without installing Rust

1. Open GitHub Actions: https://github.com/LRV2017/lucid-browser/actions
2. Select the latest successful Build Windows run.
3. Download the Lucid-Browser-Windows-x64 artifact.
4. Extract the ZIP and launch lucid-browser.exe.

GitHub builds Lucid on a hosted Windows machine. You do not need Rust, Cargo, Visual Studio or build tools installed locally.

## Try the original renderer

Launch Lucid and enter lucid://demo. The demo is parsed, styled, laid out and painted by Lucid's own experimental engine code.

## Independent engine architecture

Lucid Browser (native desktop UI)
- profiles / tabs / workspaces / sidebar / downloads
- Lucid Engine
  - HTML tokenizer + small tree builder
  - DOM data model
  - CSS selector/declaration subset
  - style cascade subset
  - basic block layout
  - paint operations
  - basic anchor hit testing

See docs/product-quality-spec.md, docs/architecture.md, docs/design-system.md, docs/roadmap.md and STATUS.md.

## Security and compatibility warning

Do not use this prototype for passwords, payments, private accounts or security-critical browsing. It has no browser-grade renderer sandbox, site isolation, complete origin security model or external security audit.

JavaScript, complete HTML/CSS standards support, images, SVG, canvas, media, workers, WebAssembly, cookies, CSP, CORS, service workers, a complete extension API, accessibility integration, and subresource loading are not implemented. Complex modern websites such as Google applications, YouTube and Discord are not expected to work.

A successful compile proves only that the code builds and its tests pass. It does not establish standards compliance, browser security or performance parity.

## Build locally (optional)

Install Rust and the Windows C++ build tools, then run cargo test --workspace --all-targets and cargo build --release --package lucid-browser.

## Quality policy

Capabilities are labeled implemented, experimental, planned or unavailable. New features require tests, documentation and a reproducible demo. Lucid 1.0 requires broad WPT/Test262 coverage, security isolation, accessibility, recovery behavior and benchmarked compatibility.

## License

MIT. See LICENSE.
