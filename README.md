# Lucid Browser

Lucid is a Windows desktop browser with a restrained, customizable interface and a **Lucid-owned native page-rendering pipeline**. It no longer depends on Microsoft Edge WebView2 or Wry.

## The Lucid Engine

Normal HTTP/HTTPS pages are processed by Lucid's own code:

```text
HTTP(S) network loader
        |
        v
Lucid HTML tokenizer and document tree
        |
        +----> linked stylesheet loader
        |             |
        |             v
        |       Lucid CSS subset parser
        |             |
        +----> style and layout
                      |
                      v
              Lucid paint operations
                      |
          +-----------+-----------+
          |                       |
      text / shapes          raster images
          |                       |
          +-----------+-----------+
                      |
                      v
               egui page surface
```

Current implementation includes bounded HTTP(S) document/resource fetching, relative resource URL resolution, a small HTML parser, a limited CSS parser and cascade, simple vertical layout, native text/shape painting, and raster image decoding for PNG, JPEG, GIF, WebP and BMP files. Page navigation and resource loading run off the UI thread. Linked stylesheets are fetched by Lucid and passed through its CSS parser; ordinary HTML image resources can be decoded and painted as native textures.

This is a genuine independent rendering path, not an embedded browser or a screenshot of another browser. It is still an **early engine prototype**, not a standards-complete browser. Many complex websites will not render correctly yet.

## Current engine limitations

The current renderer does not yet provide a JavaScript runtime or browser DOM bindings. Scripts remain inert. The CSS parser and layout system support only a small subset of modern CSS; flexbox, grid, robust inline layout, complex selectors, CSS variables, positioning and the full cascade are not complete. Fonts are not loaded from web pages, and there is no video/audio playback pipeline, SVG/canvas renderer, iframe support, service workers, browser storage/cookies, WebSockets, WebRTC, extension runtime or multi-process site isolation.

Lucid must not be described as WebView2/Chromium feature-equivalent until these subsystems exist and pass broad compatibility and security testing. The engine roadmap tracks the work in stages.

## Interface

- Lucid Calm is the default: compact tabs, a single address/search field, restrained colors and a quiet page canvas.
- Lucid Cyberpunk remains an optional alternate appearance.
- Library, history, downloads, performance controls and settings are kept in a compact menu rather than occupying a permanent sidebar.
- Keyboard shortcuts and the command palette remain available.

## Browser shell

- Native tabs, workspaces, back/forward/reload and address/search navigation.
- Persistent session metadata, bookmarks and browsing history.
- Basic page text search for the current parsed document.
- Asynchronous page loading with per-tab navigation generations.
- A native raster-image texture cache tied to tab lifetimes.
- Local file downloads and existing profile/settings foundations.

Some power features remain prototypes. The top-level navigation blocklist is not a full request-level ad blocker. There is no verified extension store or Firefox XPI compatibility. Download scanning, full site permissions, crash recovery, private-session isolation, accessibility coverage and a multi-process renderer are not complete.

## Runtime requirements

Lucid does **not** require Microsoft Edge WebView2 Runtime. The Windows build is packaged by GitHub Actions; users do not need Rust, Cargo, Visual Studio or compiler tools installed locally.

The portable artifact still depends on standard Windows system components and the libraries shipped with the executable. It does not require installing another browser engine.

## Download the Windows executable

1. Open GitHub Actions: https://github.com/LRV2017/lucid-browser/actions
2. Select the latest successful **Build Windows** run.
3. Download the `Lucid-Browser-Windows-x64` artifact.
4. Extract the ZIP and launch `lucid-browser.exe`.

## Try the Lucid renderer

Launch Lucid and enter `lucid://demo` to view a local page processed by Lucid's HTML, CSS, layout and painting code.

## Security and compatibility

Lucid now fetches and parses untrusted remote content itself. It is experimental and has not had an independent security audit. It currently lacks a separate sandboxed renderer process, mature site isolation, full browser permission controls, full CSP/CORS/mixed-content enforcement and a complete browser storage/security model. Do not use early builds for sensitive accounts, payments or security-critical browsing.

A successful compile proves only that the project builds and tests pass. It does not prove broad site compatibility, security, performance or feature parity with mature browsers.

## Build locally (optional)

Install Rust and the Windows C++ build tools, then run `cargo test --workspace --all-targets` and `cargo build --release --package lucid-browser`.

## License

Lucid source is MIT. Third-party dependency notices are described in `THIRD_PARTY_NOTICES.md`.
