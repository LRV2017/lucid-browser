# Lucid Browser Engineering Roadmap

This is the dependency-ordered implementation plan for Lucid's long-term browser-engine and desktop-product goals. It is a multi-year program, not a delivery-date promise. The detailed feature-level acceptance backlog is in [Web Platform and Browser Engine Specification](WEB_PLATFORM_SPEC.md).

## Current architecture and honest status

- [x] Native Windows browser shell with tabs, workspaces, address/search navigation and Lucid visual themes.
- [x] CEF/Chromium compatibility path integrated for ordinary modern websites in the Windows build.
- [x] Initial CEF request interception and filter-list-based network ad blocking.
- [x] Early Lucid-owned HTML/CSS/layout/paint prototype in `crates/lucid-engine`, used for engine experimentation and the local demo path.
- [ ] Standards-complete independent HTML/CSS/DOM/JavaScript engine.
- [ ] Production-grade sandbox, site isolation and browser security model.
- [ ] Broad WPT/Test262 conformance and independent security review.

CEF supplies Chromium web-platform behavior today; it is not the same thing as implementing those standards inside `lucid-engine`. The independent renderer remains experimental. Keep that distinction visible in product copy, settings and release notes.

## Phase 1 — Foundation and architecture
- [ ] Make subsystem ownership and CEF-to-Lucid migration boundaries explicit.
- [ ] Establish feature-status tracking, WPT/Test262 harnesses and reproducible CI reports.
- [ ] Add parser/layout fuzzing, resource limits, memory budgets and a threat model.
- [ ] Define origin, browsing-context, process, IPC and permission abstractions before API expansion.

## Phase 2 — HTML and DOM
- [ ] WHATWG tokenizer/tree builder, insertion modes, adoption agency, foster parenting and malformed HTML recovery.
- [ ] SVG/MathML namespaces, templates, parser lifecycle and streaming.
- [ ] Core DOM/Web IDL, mutation, traversal, events, observers and serialization.
- [ ] Forms, validation, submission encoding, dialogs, iframes, postMessage and Shadow DOM.
- [ ] Continuous HTML/DOM WPT coverage.

## Phase 3 — CSS, fonts and layout
- [ ] CSS Syntax parser, selectors, cascade, layers, custom properties and computed values.
- [ ] Media/supports/container rules, modern units/colors, animations and font-face.
- [ ] Font fallback, HarfBuzz shaping, Unicode line breaking and bidirectional text.
- [ ] Block/inline, flexbox, grid, tables, floats, columns, positioning, stacking and scrolling.
- [ ] Visual reftests and differential rendering.

## Phase 4 — JavaScript and core Web APIs
- [ ] Select and integrate a maintained JS VM with a documented update/security plan.
- [ ] Realms, event loop, tasks, microtasks, promises, timers and animation callbacks.
- [ ] DOM/Web IDL host bindings and core APIs: URL, fetch, XHR, streams and structured clone.
- [ ] Workers, WebAssembly and cross-origin-isolated SharedArrayBuffer/Atomics.
- [ ] Test262 and JavaScript/Web API WPT coverage.

## Phase 5 — Networking and storage
- [ ] HTTP/1.1, HTTP/2, HTTP/3, TLS, DNS-over-HTTPS, proxies and content decoding.
- [ ] HTTP cache, ETag/Cache-Control revalidation, HSTS and Alt-Svc.
- [ ] Cookie jar, SameSite/Partitioned cookies, origin storage and quota model.
- [ ] IndexedDB, Cache API, OPFS and service-worker lifecycle.
- [ ] SOP, CORS, CSP, COOP/COEP/CORP, Permissions-Policy and mixed-content enforcement.

## Phase 6 — Graphics and media
- [ ] Retained display lists, invalidation, tiled rasterization and GPU compositing.
- [ ] SVG, Canvas 2D, CSS effects, WebGL and WebGPU.
- [ ] AVIF/JPEG XL and animated image support; responsive images and font formats.
- [ ] Audio/video decode, WebAudio, WebCodecs, MSE and PiP.
- [ ] EME/Widevine only if licensed and supported; no assumption of DRM availability.

## Phase 7 — Security and process isolation
- [ ] Browser, renderer, network, GPU and utility process boundaries.
- [ ] Typed IPC, shared-memory frame transport and crash containment.
- [ ] Windows AppContainer/restricted-token/Job Object and applicable mitigation policies.
- [ ] Site isolation, permission controls, safe downloads and phishing/malware protection.
- [ ] Fuzzing, dependency review and independent security assessment.

## Phase 8 — DevTools and ecosystem
- [ ] Elements, console, source debugger, network, performance, memory and storage tools.
- [ ] WebExtensions/Manifest V3 runtime and package/permission management.
- [ ] DPAPI/Credential Manager password storage, autofill and WebAuthn/passkeys.
- [ ] UI Automation accessibility, screen-reader support, IME/TSF, spellcheck and find markers.
- [ ] PDFium, printing, Save as PDF, crash restore, private profiles and background updates.

## Phase 9 — Compatibility and release
- [ ] Broad WPT, Test262, WebGL/WebGPU and media conformance results.
- [ ] Differential visual tests and representative top-site compatibility matrix.
- [ ] Performance budgets for startup, navigation, script, layout, paint, memory and scrolling.
- [ ] Security audit, signed update infrastructure, crash recovery and vulnerability response.
- [ ] Only then evaluate retiring CEF for general browsing in favor of the Lucid-owned engine.

## Release rule

No capability is marked complete until implementation, tests, security behavior and platform coverage are documented. A successful compile or a working demo page is not evidence of standards compliance or browser parity.
