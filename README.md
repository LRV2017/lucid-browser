# Lucid Browser

**An independent browser-engine experiment with a clean, interactive native desktop interface.**

Lucid is being developed as an original HTML/CSS rendering engine and native desktop browser. It does **not** embed Firefox, Chromium, Blink, WebKit, Electron, CEF, or WebView2. Its current renderer is a limited prototype—not a complete engine and not yet suitable for everyday browsing.

> **Status: experimental 0.0.2.** Current code implements a small Lucid-owned HTML tokenizer/tree builder, limited CSS rules, basic block layout, paint operations and simple link activation. It has no JavaScript execution, full HTML/CSS conformance, renderer sandbox or site isolation.

## Current features

- Native Windows UI using Rust and `eframe`/`egui` for browser chrome only; page content is drawn from Lucid Engine paint operations.
- **Lucid Calm**: clean graphite/lavender interface with interactive tabs, workspaces, shortcuts, library, settings and command palette.
- **Lucid Cyberpunk**: alternate tactical terminal/player-HUD-inspired interface with angular panels and magenta/cyan system accents.
- Tabs, workspace switching, navigation history, address/search entry, library panel and quick links.
- Lucid-owned HTML tokenizer/tree builder, raw-text preservation for `style`/`script`, CSS selector/declaration subset, basic block layout and painting.
- Clickable anchor text with relative HTTP/HTTPS URL resolution.
- Per-tab request generations that discard stale network responses.
- HTTP response body capped at 5 MiB; arbitrary `file://` paths are not opened.
- GitHub Actions Windows build and downloadable portable executable.

## Download the Windows executable — no developer tools needed

1. Open [GitHub Actions](https://github.com/LRV2017/lucid-browser/actions).
2. Select the latest successful **Build Windows** run.
3. Download the `Lucid-Browser-Windows-x64` artifact.
4. Extract the ZIP and launch `lucid-browser.exe`.

GitHub compiles Lucid on a hosted Windows runner. You do not need Rust, Cargo, Visual Studio or build tools installed locally.

## Optional local build

Install Rust through [rustup](https://rustup.rs/) and the Visual Studio C++ Build Tools, then run:

```powershell
cargo test --workspace --all-targets
cargo build --release --package lucid-browser
```

## Try the original renderer

Launch Lucid and enter `lucid://demo`. The local demonstration is parsed, styled, laid out and painted by Lucid's own experimental engine code.

## Architecture

```text
Lucid Browser (native desktop interface)
  ├── tabs / workspaces / navigation / library / settings
  └── Lucid Engine
       ├── HTML tokenizer + tree builder subset
       ├── DOM data model
       ├── CSS selector/declaration subset
       ├── basic cascade and style
       ├── block layout prototype
       ├── paint operations
       └── anchor URL resolution + hit testing
```

See [architecture](docs/architecture.md), [design system](docs/design-system.md), [roadmap](docs/roadmap.md) and [status](STATUS.md).

## Security and compatibility warning

**Do not use this prototype for passwords, payments, sensitive accounts or other security-critical browsing.** It has no browser-grade renderer sandbox, site isolation, complete origin security model or external security audit.

JavaScript, cookies, CSP, CORS, service workers, WebAssembly, media playback, extension APIs, image loading, comprehensive HTML/CSS support, accessibility integration and many essential browser capabilities are not implemented. A successful compile only confirms that this prototype builds; it does not establish modern-site compatibility.

## Quality policy

New capabilities should include tests, a documented support boundary and a reproducible demonstration. GitHub Actions runs tests, Clippy and a Windows release build on every push to `main`.

## License

MIT. See [LICENSE](LICENSE).
