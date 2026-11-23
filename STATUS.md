# Project Status — 0.0.1 Experimental

This is the first source-code milestone for an independent browser engine and native browser UI.

## Implemented

- Native desktop shell with two selectable theme directions.
- Interactive tabs, address entry, basic navigation history, tab-separated workspaces, quick links, library and settings.
- Lucid-owned HTML subset parser, DOM tree structure, small CSS selector/declaration parser, top-to-bottom block layout and paint-operation generation.
- Local engine demo page.
- Experimental HTTPS/HTTP HTML fetching.
- Unit tests and a Windows build workflow.

## Not implemented

- JavaScript execution or DOM APIs exposed to JavaScript.
- Full WHATWG HTML parsing and CSS specifications.
- Inline formatting, flexbox, grid, robust text shaping, SVG, canvas and transforms.
- Images fetched from pages, audio/video, WebAssembly and workers.
- Cookies, storage, service workers, CSP, CORS and full origin model.
- Sandboxing, renderer process isolation, site isolation and security audit.
- Extensions, uBlock Origin, native filter-list blocking, production download manager and password manager.
- Full modern-site compatibility.

This build is for development and engine experimentation only. It must not be treated as a secure everyday browser.
