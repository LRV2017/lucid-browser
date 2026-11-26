# Lucid Browser

Lucid is a Windows desktop browser with a native, customizable interface and a long-term goal of a Lucid-owned standards-based web engine.

## Current architecture

- **Native Lucid shell:** eframe/egui UI, tabs, workspaces, address/search, profile, bookmarks, history, downloads and browser controls.
- **Current website compatibility:** Chromium Embedded Framework (CEF) supplies the modern web-platform implementation for ordinary websites in the Windows build. CEF is Chromium-based; it is not Microsoft Edge WebView2.
- **Lucid Engine prototype:** `crates/lucid-engine` contains Lucid-owned HTML parsing, early DOM structures, CSS subset, layout and paint experiments. It is not yet a complete replacement for Chromium and does not provide a standards-complete DOM or JavaScript runtime.

The intended architecture keeps Lucid's UI and product behavior independent from the page engine. The independent engine will replace the compatibility path only after it passes documented compatibility, conformance, performance and security gates. See [the architecture](docs/architecture.md), [engineering roadmap](docs/roadmap.md) and [full web-platform specification](docs/WEB_PLATFORM_SPEC.md).

## Current capabilities

- Native tabs, workspaces, back/forward/reload and address/search navigation.
- Chromium-powered rendering for ordinary modern websites through the bundled CEF runtime.
- Initial request interception and filter-list-based network ad blocking; this is not a complete uBlock Origin replacement.
- Lucid-owned experimental renderer path for engine development and the local `lucid://demo` experience.
- Persistent profile foundations, bookmarks, history, downloads, appearance settings and performance controls.

Feature availability depends on the packaged CEF build and Windows environment. Firefox XPI compatibility is not a drop-in feature and is not currently promised.

## Independent engine status

The Lucid Engine is an early prototype. It does not yet implement the complete WHATWG HTML parser, DOM/Web IDL, CSS standards, JavaScript runtime, full layout algorithms, media stack, browser storage, service workers, site isolation or production-grade sandboxing. The complete target is documented in [docs/WEB_PLATFORM_SPEC.md](docs/WEB_PLATFORM_SPEC.md).

Do not describe the Lucid-owned renderer as Chrome/Firefox/WebView2-equivalent until it passes the release gates and broad compatibility/security testing. Experimental builds should not be used for sensitive accounts, payments or security-critical browsing.

## Windows build

GitHub Actions packages the Windows x64 browser and its CEF runtime. Users do not need Rust, Cargo or compiler tools installed locally.

1. Open [GitHub Actions](https://github.com/LRV2017/lucid-browser/actions).
2. Select the latest successful Windows build.
3. Download the `Lucid-Browser-Windows-x64` artifact.
4. Extract the ZIP and launch `lucid-browser.exe`.

## Build locally (optional)

Install Rust and the Windows C++ build tools, then run:

```powershell
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
cargo build --release --package lucid-browser
```

## License

Lucid source is MIT. Third-party dependency notices are described in `THIRD_PARTY_NOTICES.md`.
