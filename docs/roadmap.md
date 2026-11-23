# Lucid Browser Roadmap

This is a technical sequence, not a promise of dates. Every stage should have tests, demos and documented gaps before being called complete.

## 0.0.1 — Foundation (current)

- [x] Cargo workspace and native Windows desktop application.
- [x] Two independent visual treatments: Lucid Calm and Lucid Cyberpunk.
- [x] Tabs, address field, basic history navigation, workspaces, library, settings and command palette.
- [x] Original HTML tokenizer/tree builder prototype.
- [x] CSS selector/declaration subset.
- [x] Simple block layout and paint commands.
- [x] Local engine demonstration page.
- [x] HTTP(S) HTML fetching for experiments.
- [x] Unit tests and Windows CI build workflow.
- [ ] Verify compilation and runtime on a real Windows 11 machine.

## 0.0.2 — Parser correctness
- Token stream with source locations and diagnostics.
- Expand tree construction and error recovery against a documented HTML subset.
- Entity handling and UTF-8 edge-case tests.
- Parser fuzzing and regression corpora.
- Relevant Web Platform Tests (WPT) cases.

## 0.0.3 — DOM fundamentals
- Stable node identity and document ownership.
- DOM traversal and mutation APIs.
- Text content, attributes and element lookup.
- Document lifecycle and mutation invalidation.
- DOM conformance unit tests.

## 0.0.4 — CSS style system
- Token-based CSS parser and error recovery.
- Selector matching, specificity, source order and inheritance.
- Computed and used values.
- Stylesheet loading and URL resolution.
- Media queries and viewport changes.
- CSS WPT coverage.

## 0.0.5 — Layout and text
- Inline formatting contexts and line boxes.
- Font shaping and text metrics.
- Box model and intrinsic sizing.
- Flexbox, then grid.
- Positioning, overflow and scrolling.
- Bidi and writing modes.
- Pixel and reftest comparisons.

## 0.0.6 — Rendering pipeline
- Retained display lists and invalidation.
- Image decoding and SVG subset.
- Paint ordering, clipping and stacking contexts.
- Compositing, transforms and opacity.
- GPU acceleration with a tested CPU fallback.

## 0.0.7 — JavaScript and Web APIs
- Evaluate the trade-off between an external JavaScript language runtime and a Lucid-owned runtime.
- Define a host boundary for DOM bindings and the browser event loop.
- Implement timers, events, fetch, URL, storage and other APIs incrementally.
- Run ECMAScript Test262 for language compatibility and WPT for web APIs.

## 0.0.8 — Networking and security
- URL parser, HTTP cache, cookies and content decoding.
- Origin model, CSP, CORS, mixed-content rules and certificate handling.
- Download manager and permission system.
- Multi-process renderer, sandboxing, IPC and site isolation.
- Fuzzing, dependency review, threat model and security audit.

## 0.0.9 — Browser features
- Profiles, session restore, bookmarks, history and downloads.
- Extension API design and sandboxed extension processes.
- Native ad/tracker filtering with user-configurable lists.
- Media pipeline, accessibility, keyboard support and developer tools.
- Performance monitoring and privacy dashboard.

## 1.0 — Release criteria

No 1.0 release until the engine has broad standards conformance, reliable rendering of representative modern sites, media support, accessibility, security isolation, crash recovery, update strategy, robust automated tests, documented performance and a public vulnerability-reporting process.
