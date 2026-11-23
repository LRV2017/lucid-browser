# Third-party notices

## Desktop shell

Lucid uses the Rust `eframe` / `egui` ecosystem for its native application interface.

## Live website rendering

Lucid uses **Wry** as the native WebView host. Wry is distributed under MIT and Apache-2.0 licenses; see the Wry project repository and its bundled license files: https://github.com/tauri-apps/wry

On Windows, Wry hosts **Microsoft Edge WebView2 Runtime**. WebView2 is Microsoft's Chromium-based embeddable web platform and is installed/serviced separately from Lucid. Its runtime is governed by Microsoft's applicable terms and is not bundled into the portable Lucid ZIP. Runtime distribution guidance: https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution

## Lucid Engine

The `crates/lucid-engine` HTML/CSS prototype is Lucid-owned code. It is not used to render normal live websites.

Before public releases, generate and review a complete dependency/license inventory (for example with cargo-about), include required notices, and verify licenses and redistributable runtime terms against the final packaging model.
