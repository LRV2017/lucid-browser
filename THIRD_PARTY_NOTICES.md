# Third-party notices

Lucid's current architecture uses a native eframe/egui desktop shell and a Lucid-owned page renderer. **Wry and Microsoft Edge WebView2 are not dependencies of the current application.**

## Desktop UI and graphics

Lucid uses the Rust `eframe` / `egui` ecosystem for its native application interface and page painting. Their upstream repositories and license files are available at https://github.com/emilk/egui.

## Networking and serialization

- `reqwest` is used for HTTP(S) requests with rustls TLS support. See https://github.com/seanmonstar/reqwest.
- `serde` and `serde_json` are used for profile serialization. See https://github.com/serde-rs/serde.

## Image decoding

The `image` crate is used for supported raster image decoding. See https://github.com/image-rs/image.

## Lucid Engine

The HTML subset parser, CSS subset parser, resource discovery, layout primitives and paint-operation generation in `crates/lucid-engine` are Lucid-owned code.

Before public releases, generate and review a complete dependency/license inventory (for example with cargo-about), include required notices, and verify every dependency license and redistribution condition against the final packaging model.
