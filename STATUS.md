# Project Status — 0.0.2 Experimental

Lucid is an independent browser-engine research project and native desktop application.

## Implemented in this prototype

- Native desktop UI with Lucid Calm and Lucid Cyberpunk themes.
- Tabs, workspace switching, address entry, basic navigation history, library, quick access and settings.
- Lucid-owned HTML tokenizer/tree builder for a small supported subset.
- Raw-text preservation for `style` and `script` elements; scripts remain inert and are not executed.
- Basic CSS selector/declaration parsing and a simple style cascade.
- Top-to-bottom block layout and paint operation generation.
- Anchor href preservation and clickable HTTP/HTTPS links with relative URL resolution.
- Per-tab navigation generation checks to ignore stale asynchronous responses.
- HTTP response body limit of 5 MiB.
- Arbitrary local-file URL access disabled.
- Unit tests and GitHub Actions Windows build.

## Not implemented

- Full WHATWG HTML parsing or complete CSS specifications.
- JavaScript, script DOM bindings, event loop or browser Web APIs.
- Inline layout, correct line boxes, flexbox, grid, floats, complex positioning or standards-correct text shaping.
- Images, SVG, canvas, audio/video, WebAssembly, workers and service workers.
- Cookies, storage, CSP, CORS, full origin model and certificate UI.
- Renderer sandbox, process separation, site isolation, crash recovery and security audit.
- Extensions, filter-list blocking, password manager and production downloads.
- Accessibility tree, screen-reader support and broad keyboard navigation.
- Broad modern website compatibility.

**This build is experimental. Do not use it for sensitive browsing.**
