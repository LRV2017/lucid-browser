# Lucid Browser Roadmap

This roadmap is an engineering sequence with release gates, not a delivery-date promise or a claim of browser parity. Lucid is intentionally building its own renderer; it will not quietly fall back to WebView2.

## 0.0.5 — Native rendering foundation
- [x] Remove Wry/WebView2 from the application.
- [x] Use Lucid's own HTTP(S) document fetch and HTML parsing path for normal navigation.
- [x] Add bounded resource loading and linked stylesheet discovery.
- [x] Paint Lucid layout output through the native egui surface.
- [x] Add raster image decoding and texture painting for supported formats.
- [x] Make page loading asynchronous and guard against stale tab responses.
- [x] Simplify default Lucid Calm chrome and put secondary tools in a compact menu.
- [ ] Improve HTML parser correctness and document/DOM mutation model.
- [ ] Implement standards-based CSS parsing, cascade and computed style.
- [ ] Add inline formatting, line boxes, flexbox, grid, positioning and scrolling.
- [ ] Add reliable resource scheduling, cache, content-type checks and CSS asset fetching.

## 0.0.6 — Fonts, graphics and layout
- [ ] Download and decode web fonts with licensing and origin rules.
- [ ] Font fallback, shaping, script coverage and text metrics.
- [ ] Image intrinsic sizing, responsive srcset selection, SVG and CSS backgrounds.
- [ ] GPU compositing, clipping, stacking contexts and retained display lists.
- [ ] Layout/reftest suite across viewport sizes and device scale factors.

## 0.0.7 — JavaScript and the DOM platform
- [ ] Integrate a standalone JavaScript engine.
- [ ] Define realms, execution contexts, event loop, jobs and microtasks.
- [ ] Expose a secure DOM host API with mutation, events and observers.
- [ ] Add timers, promises, fetch, URL, storage and structured clone APIs.
- [ ] Add workers and WebAssembly incrementally.
- [ ] Use ECMAScript Test262 and Web Platform Tests to measure compatibility.

## 0.0.8 — Media and browser APIs
- [ ] Audio/video demux, decoding, rendering and playback controls.
- [ ] WebAudio, MediaSource and media-session foundations.
- [ ] Iframes and embedded browsing contexts.
- [ ] Cookies, local/session storage, IndexedDB and service workers.
- [ ] WebSocket and WebRTC APIs.
- [ ] Accessibility tree and assistive-technology support.

## 0.0.9 — Security, privacy and ecosystem
- [ ] HTTP cache, connection pooling, HTTP/2 and HTTP/3.
- [ ] Origin model, CSP, CORS, mixed-content and certificate behavior.
- [ ] Resource-level ad/tracker blocking with filter-list parsing.
- [ ] Permission manager, safe downloads and private browsing.
- [ ] Multi-process renderer, sandboxing, IPC validation and site isolation.
- [ ] Sandboxed extension API, permission model and signed packages.
- [ ] Fuzzing, dependency review and external security review.

## 1.0 — Release gates

No 1.0 release until broad web standards conformance, representative modern-site compatibility, JavaScript and media support, accessibility, site isolation, crash recovery, secure update infrastructure, reproducible benchmarks and a vulnerability response process have been demonstrated.
