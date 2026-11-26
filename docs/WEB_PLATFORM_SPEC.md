# Lucid Web Platform and Browser Engine Specification

**Status:** Target requirements and engineering backlog. This document does not claim these features are implemented.
**Product direction:** A native Lucid browser shell and, over time, a Lucid-owned standards-based web engine. Chromium Embedded Framework (CEF) currently supplies modern website compatibility in the Windows build while the independent `lucid-engine` crate remains an early prototype. Any eventual replacement of CEF must pass the compatibility, security, performance and conformance gates in this document.

## Implementation truth and principles

- Keep the native Lucid UI, tab/workspace model, profile UX and product identity independent from the page engine.
- Treat the existing CEF path as the current compatibility path for ordinary websites; do not describe CEF capabilities as Lucid-engine implementations.
- Grow `crates/lucid-engine` behind explicit feature gates and test suites. Do not route everyday sites to an incomplete renderer merely to claim independence.
- Prefer standards and upstream conformance suites over ad-hoc compatibility claims: WHATWG HTML, CSSWG specifications, Web IDL, ECMAScript, Web Platform Tests (WPT), Test262, WebGL conformance and relevant media tests.
- Implement security boundaries before exposing powerful APIs. Permissions, origin isolation, sandboxing and safe defaults are architecture requirements, not optional polish.
- Track each capability as **Not started**, **Prototype**, **In progress**, or **Verified**. “Verified” requires automated tests and documented platform coverage; a compiling build alone is not verification.
- Build incrementally. The complete scope below is a multi-year browser-engine program, not one commit or a realistic single release.

## 1. HTML and DOM engine

### HTML parsing and tree construction
- [ ] WHATWG HTML Living Standard tokenizer and tree-construction algorithm.
- [ ] Insertion modes, implied end tags, adoption agency algorithm, foster parenting and malformed-markup recovery.
- [ ] `template` contents and template insertion modes.
- [ ] Foreign-content integration points and namespaces for SVG and MathML.
- [ ] Character references, raw-text/RCDATA states, scripting-enabled parsing behavior and parser re-entry rules.
- [ ] Streaming/incremental parsing and deterministic error recovery.

### DOM and Web IDL
- [ ] Live DOM with Node, Element, Document, DocumentFragment, DocumentType, Text, Comment and Window.
- [ ] Web IDL conversion, overload resolution, exceptions, iterable and collection semantics.
- [ ] Shadow DOM v1: ShadowRoot, slots, slot assignment, composed tree and event retargeting.
- [ ] CustomElementRegistry, custom-element lifecycle callbacks, upgrade and reaction queues.
- [ ] HTMLCollection, NodeList, NamedNodeMap, DOMTokenList, ranges, traversal and serialization.
- [ ] Mutation algorithms, adoption/import, cloning, document readiness and parsing lifecycle.

### Events and observers
- [ ] EventTarget, capturing, target and bubbling phases, composed events and cancellation.
- [ ] addEventListener/removeEventListener options, once, passive, AbortSignal and listener ordering.
- [ ] CustomEvent and trusted/user-input event distinctions.
- [ ] MutationObserver with microtask delivery and mutation records.
- [ ] ResizeObserver with loop protection and rendering lifecycle integration.
- [ ] IntersectionObserver with root margins, thresholds and visibility timing.

### Forms and embedded documents
- [ ] Constraint validation, validity states, form-associated elements and submission algorithms.
- [ ] Input types: text, search, password, email, URL, number, checkbox, radio, file, date/time, color, range and supported specialized controls.
- [ ] Select, option, textarea, label, fieldset, output, progress, meter and dialog behavior.
- [ ] URL-encoded and multipart/form-data submission, successful controls and submitter semantics.
- [ ] Nested iframe browsing contexts, sandbox attributes, origin separation and window.postMessage structured-clone messaging.
- [ ] Embed/object fallback and MIME handling; Shadow DOM encapsulation and focus behavior.

## 2. CSS and layout engine

### Parsing, selectors and cascade
- [ ] CSS Syntax tokenizer/parser, error recovery, CSSOM and stylesheet loading.
- [ ] Selectors Level 4: type/class/id/attribute, combinators, :hover, :active, :focus, :focus-visible, :nth-child(), :not(), :is(), :where() and :has().
- [ ] Pseudo-elements: ::before, ::after, ::placeholder, ::selection and applicable generated content behavior.
- [ ] Specificity, origin/importance, inheritance, source order, scoping and @layer cascade layers.
- [ ] `!important`, shorthand expansion, invalid-at-computed-value handling and computed/used values.
- [ ] Custom properties and var() substitution, including cycle detection and fallback.

### Values, at-rules and responsive behavior
- [ ] CSS Values and Units: calc(), min(), max(), clamp(), attr(), vw/vh/dvh, rem/em and container-relative units.
- [ ] Color functions/spaces: modern rgb/hsl, lab(), oklab(), lch(), oklch() and color-mix().
- [ ] @media for viewport dimensions, color scheme, reduced motion, print and supported device capabilities.
- [ ] @supports feature queries and @container size/style queries.
- [ ] @keyframes and animation timing; transitions and reduced-motion behavior.
- [ ] @font-face with WOFF2, WOFF, TTF and OTF loading, font matching and origin checks.
- [ ] @import ordering, media conditions, cycle prevention and fetch priority.

### Formatting and layout
- [ ] Block and inline formatting contexts, margin collapsing, inline box splitting and line box construction.
- [ ] Unicode Line Breaking Algorithm, Unicode Bidirectional Algorithm and complex text shaping with HarfBuzz (or equivalent).
- [ ] Font fallback, script coverage, glyph metrics, variable fonts and text decoration.
- [ ] Flexbox: direction, wrap, justify/align properties, grow/shrink/basis and order.
- [ ] Grid: explicit/implicit tracks, fr, minmax(), repeat(), auto-fit/auto-fill, named areas and subgrid.
- [ ] Table formatting, floats/clear and multi-column layout.
- [ ] Static, relative, absolute, fixed and sticky positioning; containing blocks and z-index stacking contexts.
- [ ] Overflow, scroll containers, scrollbars, scroll snapping and viewport behavior.
- [ ] Intrinsic sizing, replaced elements, fragmentation, writing modes and zoom/device scale.

## 3. Graphics, compositing and media

### Paint and compositing
- [ ] Retained display lists, invalidation, paint chunks and layer tree promotion.
- [ ] Tiled rasterization and GPU compositing using a deliberate backend choice (wgpu, Skia/Direct3D 12 or a measured combination).
- [ ] Compositor-thread scrolling and animations, frame pacing, vsync and adaptive 60–120+ FPS targets where hardware permits.
- [ ] Clipping, transforms, opacity, stacking contexts, hit testing and damage tracking.
- [ ] Resource budgets, GPU process isolation, device-loss recovery and software fallback.

### 2D/3D graphics and images
- [ ] SVG 2 parsing, DOM integration, styling, geometry, text and painting.
- [ ] Canvas 2D API, paths, gradients, patterns, image data and origin-clean/security rules.
- [ ] CSS 2D/3D transforms, filters, backdrop-filter, clip-path, masks, shadows, border radii and linear/radial/conic gradients.
- [ ] WebGL 1/2 through a maintained ANGLE/DirectX path and WebGL conformance.
- [ ] WebGPU with wgpu, adapter/device lifecycle, validation and security isolation.
- [ ] Image support: PNG, JPEG, GIF, WebP, BMP, AVIF, JPEG XL and SVG, subject to decoder maturity and platform support.
- [ ] Animated WebP/APNG timing, frame disposal and responsive `srcset`/`picture` selection.

### Audio and video
- [ ] Hardware-accelerated decode where available: H.264, HEVC, VP8, VP9 and AV1; software fallback and capability reporting.
- [ ] Audio decode/playback: Opus, AAC, MP3, FLAC and Vorbis.
- [ ] Media Source Extensions for adaptive streaming and robust buffering.
- [ ] Encrypted Media Extensions and Widevine only through licensed, policy-compliant integration; DRM is not assumed available.
- [ ] WebAudio graph, nodes, synthesis, spatial audio and autoplay policy.
- [ ] WebCodecs, codec capability checks, frame lifetime and worker integration.
- [ ] Picture-in-Picture, media controls, captions, tracks and accessibility.

## 4. JavaScript runtime and browser Web APIs

### ECMAScript and scheduling
- [ ] Select and integrate a maintained JavaScript engine (for example V8 via rusty_v8 or SpiderMonkey); document ABI, update and security strategy.
- [ ] ECMAScript language support through the current target edition, garbage collection, JIT policy and WebAssembly.
- [ ] Realms, execution contexts, modules, strict mode, exceptions and host hooks.
- [ ] HTML event loop: task sources, macrotasks, microtask checkpoints, Promise jobs, queueMicrotask, timers, requestAnimationFrame and requestIdleCallback.
- [ ] CSP-aware script loading, module graph fetch, import maps and dynamic import.

### Workers, networking and real-time
- [ ] Dedicated, Shared and Service Workers; lifecycle, registration, update and termination rules.
- [ ] SharedArrayBuffer and Atomics only under required cross-origin-isolation conditions.
- [ ] fetch(), XMLHttpRequest, AbortController, streams, Headers, Request and Response.
- [ ] WebSocket, Server-Sent Events, Beacon and WebTransport.
- [ ] WebRTC peer connections, audio/video/data channels, ICE, STUN/TURN and permission controls.

### Device and platform APIs
- [ ] History API, URL and URLSearchParams.
- [ ] Blob, File, FileReader and permissioned File System Access API.
- [ ] Clipboard API with secure-context and user-activation restrictions.
- [ ] Geolocation and Notifications with explicit per-origin permissions.
- [ ] Gamepad, WebMIDI, WebBluetooth, WebUSB and WebHID behind platform support and permission prompts.
- [ ] Pointer Lock, Fullscreen, Web Speech TTS/STT and user-activation policies.
- [ ] Structured clone, transferable objects, AbortSignal and common interoperable primitives.

## 5. Networking, caching and client storage

### Network stack
- [ ] HTTP/1.1 connection pooling/keep-alive, HTTP/2 multiplexing and header compression, HTTP/3 over QUIC.
- [ ] Brotli and Zstandard content decoding; gzip/deflate compatibility.
- [ ] DNS-over-HTTPS configuration, HSTS, Alt-Svc and proxy discovery.
- [ ] HTTP disk/memory cache with Cache-Control, validators, ETag revalidation, Vary and eviction.
- [ ] System proxy, PAC and SOCKS5 support; proxy authentication and bypass rules.
- [ ] TLS verification against trusted roots, certificate error UX, revocation/OCSP behavior and Certificate Transparency where supported.
- [ ] Redirect, referrer, content-type, download and mixed-content handling.

### Cookies and origin storage
- [ ] RFC 6265bis-compatible cookie parsing/jar: Domain, Path, Expires, Max-Age, Secure, HttpOnly, SameSite and Partitioned/CHIPS.
- [ ] Origin-scoped localStorage and sessionStorage with synchronous API semantics and durable profile persistence.
- [ ] IndexedDB transactions, object stores, indexes, structured clone and quota management.
- [ ] Cache API integrated with Service Workers.
- [ ] Origin Private File System with quotas and lifecycle semantics.
- [ ] Storage partitioning, eviction, clear-site-data, private profiles and anti-tracking boundaries.

## 6. Multi-process architecture and security

### Process model and IPC
- [ ] Browser process: privileged UI, tabs, windowing, profile disk I/O and process lifecycle.
- [ ] Renderer processes: isolated site/origin execution for HTML, CSS, layout, JS and DOM.
- [ ] Dedicated network process for sockets, TLS, cookies and HTTP cache.
- [ ] GPU process for graphics commands and video decode.
- [ ] Utility processes for audio, storage and extension execution where warranted.
- [ ] Typed, versioned IPC over Windows Named Pipes and shared memory; validate every message and handle lifetime.
- [ ] Shared-memory frame buffers with explicit ownership, synchronization and safe zero-copy design.
- [ ] Site Isolation policy, process reuse rules, crash containment and process resource limits.

### Windows sandbox and web security
- [ ] Least-privilege Windows sandbox using AppContainer, restricted tokens, Job Objects and supported Win32k lockdown/ACG/CFG mitigations.
- [ ] Defense-in-depth; no claim that a process is sandboxed until effective token/mitigation state is verified in tests.
- [ ] Same-Origin Policy across DOM, storage and network; CORS preflight and response enforcement.
- [ ] CSP directives including script-src, style-src, frame-ancestors, nonces and hashes.
- [ ] COOP, COEP, CORP, Permissions-Policy, Referrer-Policy and mixed-content blocking.
- [ ] Secure-context rules, certificate verification, CT/revocation behavior and safe TLS failure handling.
- [ ] Phishing/malware URL and download protection with a maintained Safe Browsing-style feed and privacy review.
- [ ] Fuzzing, memory-safety review, dependency auditing, security response process and independent assessment.

## 7. Browser shell, DevTools and ecosystem

### Developer tools
- [ ] Live Elements inspector, DOM tree, computed styles, box model and live style editor.
- [ ] Interactive JavaScript console and source debugger with breakpoints, call stack, scopes and watch expressions.
- [ ] Network waterfall, request/response headers, payloads and timing.
- [ ] Performance flamegraph/profiler, memory heap snapshots and Application storage viewer.
- [ ] DevTools protocol/API, remote debugging controls and secure attach permissions.

### Identity, extensions and ecosystem
- [ ] OS-protected password manager using Windows DPAPI/Credential Manager, password generator and breach alerts with privacy-safe handling.
- [ ] Autofill for addresses and payment cards; explicit user control and secure storage.
- [ ] WebAuthn/FIDO2/passkeys and Windows Hello integration.
- [ ] WebExtensions-compatible runtime, Manifest V3, isolated content scripts and extension background service workers.
- [ ] declarativeNetRequest and permission-scoped APIs; signed/validated extension packages and .crx/.zip import.
- [ ] Firefox XPI compatibility is a separate compatibility goal; do not imply drop-in support without API mapping and extension test coverage.
- [ ] Extension review, update, disable/uninstall and permission transparency.

### Accessibility, input, printing and power features
- [ ] Windows UI Automation accessibility tree for NVDA, Narrator and JAWS; keyboard navigation and semantic roles.
- [ ] Text Services Framework/IME support for East Asian and complex input, spellcheck and composition.
- [ ] Find-in-page with match counts and scrollbar markers.
- [ ] PDFium-based PDF viewer and print/Save as PDF pipeline.
- [ ] Crash recovery and session restore with safe recovery prompts.
- [ ] Incognito/ephemeral profiles with isolated in-memory storage/cache and no persistence after close.
- [ ] Tab audio indicators and per-tab mute.
- [ ] Picture-in-Picture video window and zoom-per-origin.
- [ ] Secure background updates, staged rollout, rollback and signed update artifacts.

## 8. Delivery phases and gates

These phases are dependency-ordered engineering milestones, not promised dates.

1. **Foundation and architecture:** authoritative subsystem boundaries, feature-status inventory, test harnesses, telemetry-free reproducible benchmarks, fuzzing setup, resource budgets and threat model.
2. **HTML and DOM:** standards tokenizer/tree builder, core DOM/Web IDL, events, forms, templates, namespaces, Shadow DOM and WPT coverage.
3. **CSS and text/layout:** CSS parser/cascade/selectors, font loading/shaping, block/inline, flexbox, grid, positioning and reftests.
4. **JavaScript and core Web APIs:** JS engine, realms/event loop, DOM bindings, fetch, timers, workers and Test262/WPT integration.
5. **Networking and storage:** HTTP cache/protocols, cookies, origin model, Web Storage, IndexedDB, Cache API and service workers.
6. **Graphics and media:** SVG, canvas, GPU compositing, WebGL/WebGPU, image formats, audio/video and streaming.
7. **Security and process isolation:** browser/network/renderer/GPU process boundaries, Windows sandbox, IPC hardening, CSP/CORS/SOP and Safe Browsing.
8. **DevTools and ecosystem:** inspector/debugger, extension runtime, password/autofill/passkeys, accessibility, PDF/printing and power features.
9. **Compatibility and release:** broad WPT/Test262/WebGL/media conformance, top-site compatibility, performance budgets, security audit, crash recovery and signed update process.

### Release gates
- A feature is not complete because an interface exists; behavior, edge cases, security and automated tests must pass.
- Run relevant WPT subsets continuously; publish pass/fail counts and known failures by platform.
- Run Test262 for ECMAScript and applicable WebGL/WebGPU/media conformance suites.
- Differential rendering tests must compare layout and pixels against reference browsers across viewport sizes, scale factors, fonts and color schemes.
- Fuzz HTML, CSS, URL, image, media, IPC and extension parsing boundaries.
- Benchmark startup, navigation, script, layout, paint, scrolling, memory, GPU and battery/CPU impact on documented hardware.
- No claim of Chrome/Firefox/WebView2 parity or suitability for sensitive daily browsing until site isolation, sandboxing, TLS/origin enforcement, permissions, crash handling and independent security review are demonstrated.

## 9. Explicit non-goals and constraints

- Do not bundle DRM capabilities such as Widevine without the necessary licensing and platform agreements.
- Do not expose hardware/device APIs without secure-context, permission, user-activation and revocation controls.
- Do not promise all codecs, GPU backends, OS sandbox mitigations or APIs on every Windows device; capability detection and graceful fallback are required.
- Do not remove the current CEF compatibility path until the independent engine passes the release gates and a migration decision is documented.
