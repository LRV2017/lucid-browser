# Lucid Browser Architecture

## Product boundaries

- `apps/lucid-browser`: native eframe/egui desktop shell, tabs, workspaces, profile, address handling, bookmarks, history, downloads and browser controls.
- `crates/lucid-engine`: Lucid-owned document parsing, CSS subset, resource loading, layout and paint operation generation.
- There is no Wry/WebView2 dependency or embedded external browser surface in the current application.

## Native page lifecycle

```text
Address bar / navigation
       |
       v
HTTP(S) document fetch (reqwest + rustls)
       |
       v
Lucid HTML tokenizer and document tree
       |
       +----> inline style collection
       |
       +----> linked resource discovery
                    |
                    v
              HTTP(S) resource loader
                    |
          +---------+----------+
          |                    |
       CSS files           raster images
          |                    |
          v                    v
     CSS subset parser     image decoder
          |                    |
          +---------+----------+
                    |
                    v
           style and layout
                    |
                    v
            paint operations
                    |
                    v
          egui text/shape/image painter
```

Page requests execute on background threads. A tab ID and navigation-generation value prevent an older response from overwriting a newer navigation in the same tab. Image bytes are decoded away from the UI thread; the UI thread uploads RGBA data as egui textures and retains those textures by tab and source URL.

## Current implementation scope

### HTML and document tree

The independent parser recognizes a useful subset of tags, attributes, comments and raw text. It is not the WHATWG/HTML Living Standard parser and does not implement complete malformed-markup recovery, namespaces, templates or the complete DOM API. Script bodies are stored as inert text.

### CSS

Inline style blocks and linked stylesheets pass through a small Lucid CSS parser. Selectors and declarations are restricted to a small subset, and layout is primarily top-to-bottom block flow. CSSOM, custom properties, full specificity/cascade rules, pseudo-classes, flexbox, grid, intrinsic sizing, positioning, stacking contexts and standards-level line layout are not complete.

### Images and resources

The resource loader resolves relative HTTP(S) URLs, applies timeouts/redirect limits and enforces a per-resource byte cap. The app decodes selected raster image formats and paints them through egui textures. This does not yet cover SVG, CSS background images, responsive image selection, animation timing or video/audio streams. Resource discovery is not yet a complete browser fetch algorithm.

## Not implemented yet

- JavaScript execution, realms, event loop, DOM host bindings or Web APIs.
- Fonts, shaping, fallback, SVG, canvas, iframe and embedded browsing contexts.
- Audio/video codecs, streaming, media sessions and playback.
- Cookies, local/session storage, IndexedDB, service workers, WebSocket and WebRTC.
- Complete HTML/CSS compatibility and broad Web Platform Tests coverage.
- Multi-process rendering, sandboxing, origin isolation and a browser-grade security model.
- Accessibility tree, extension runtime and mature developer tools.

## Security considerations

All remote content is untrusted. The current engine runs in the browser application process and is not protected by a production-grade renderer sandbox or site isolation. URL scheme validation and bounded network resource loading are only parts of a broader security model. TLS, CSP, CORS, origin policy, mixed-content handling, downloads, permissions and hostile document parsing need dedicated verification. Do not claim Chrome/WebView2-equivalent security or compatibility.
