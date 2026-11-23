# Lucid Browser Roadmap

This roadmap is an engineering sequence with release gates, not a delivery-date promise or a claim of parity with mature browsers. The complete feature and quality contract is in product-quality-spec.md.

## 0.0.3 — Power-user UI foundations (current)
- [x] Calm and Cyberpunk themes.
- [x] Persistent profile, workspaces, bookmarks, history and session metadata.
- [x] Pinned tabs and manual background-tab sleeping.
- [x] GX-style Control Center with process/system metrics.
- [x] Per-navigation HTML bandwidth governor.
- [x] Top-level navigation host protection.
- [x] Downloads queue, Mod Center and gaming shortcut hub.
- [x] Automated Windows build workflow.

## 0.0.4 — Browser application completeness
- Tab groups/islands, tab search, previews, drag/drop and split-screen.
- Full session/crash restore, multiple profiles, guest and private sessions.
- User-manageable speed dial, widgets and keyboard shortcut editor.
- Downloads pause/resume, integrity checking and safe-file handling.
- Pinboards, notes, snapshots, reader view, video pop-out and media controls.
- Sidebar integration architecture and configurable game content feeds.

## Independent engine foundation — in progress\n- [x] Add bounded HTTP(S) resource fetching with redirects, timeouts and byte limits.\n- [x] Discover common external CSS, JavaScript, image, font-preload and media references in HTML.\n- [ ] Connect resource discovery and loading to a page/document lifecycle.\n- [ ] Parse external CSS and CSS `@import` / `url()` resources.\n- [ ] Decode and paint images and load real web fonts.\n- [ ] Add a JavaScript runtime with Lucid-owned DOM bindings and event-loop integration.\n- [ ] Add streaming media loading, audio/video decoding and playback controls.\n- [ ] Replace WebView2 only after compatibility and security gates are met.\n\n## 0.0.5 — HTML and DOM correctness
- Token stream with source locations and deterministic error recovery.
- HTML tree construction and malformed-markup handling against standards tests.
- DOM identity, traversal, mutation, attributes, events and fragments.
- URL parsing, document base URL and navigation lifecycle.
- Parser fuzzing and WPT parser tests.

## 0.0.6 — CSS, layout and rendering
- Token-based CSS parser, selector matching, cascade, inheritance and computed values.
- Inline formatting, line boxes, font shaping and fallback.
- Box model, intrinsic sizing, flexbox and grid.
- Positioning, overflow, stacking contexts and scrolling.
- Image decoding, SVG, retained paint lists and GPU compositing.
- Pixel/reftest comparisons and viewport matrices.

## 0.0.7 — JavaScript and modern web APIs
- Choose and integrate a standalone JavaScript runtime or develop LucidScript.
- Define realms, host bindings and event-loop semantics.
- DOM integration, events, timers, promises, fetch, storage, workers and WebAssembly.
- Media, WebSockets, WebRTC and service-worker APIs incrementally.
- Use ECMAScript Test262 and WPT to measure compatibility.

## 0.0.8 — Networking, privacy and security
- HTTP cache, cookies, TLS, HTTP/2 and HTTP/3.
- Origin model, CSP, CORS, mixed-content and certificate behavior.
- Resource-level ad/tracker blocking with filter-list parsing.
- Permission manager, secure downloads and private browsing.
- Multi-process renderer, sandboxing, IPC validation and site isolation.
- Threat model, fuzzing, dependency review and external security review.

## 0.0.9 — Extensions, mods and ecosystem
- Sandboxed extension API, permission model and signed packages.
- Documented WebExtensions-compatible subset and extension conformance tests.
- Permissioned mod packs, signing, rollback and safe installation.
- Media/session integration, accessibility tree, developer tools and diagnostics.
- True per-site CPU/RAM budgets after renderer isolation exists.

## 1.0 — Release gates

No 1.0 release until broad web standards conformance, representative modern-site compatibility, JavaScript and media support, accessibility, site isolation, crash recovery, secure update infrastructure, reproducible benchmarks and a vulnerability response process have been demonstrated.
