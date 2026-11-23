# Lucid Browser — Product and Quality Specification

This document is the product contract for Lucid. The goal is a browser with the depth of a gaming-focused power-user browser and an independently implemented web platform engine. It is not a claim that current experimental builds already have those capabilities.

## Product identity

### Lucid Calm — default
- Very clean, low-noise browser chrome with restrained graphite and lavender.
- A rich, rearrangeable home dashboard: speed dial, workspaces, cards, recent sessions, bookmarks, downloads and optional widgets.
- Tactile interactions: fast command palette, animated panels, keyboard shortcuts, hover feedback, tab previews, drag/reorder, tab search and contextual menus.
- Everything optional: no forced feeds, advertising, news cards, sound, background music, motion or decorative clutter.
- Calm must be feature-rich without becoming visually crowded.

### Lucid Cyberpunk — alternate
- An in-world personal-terminal/player-HUD experience, not just a dark theme with neon accents.
- Angular modules, dense technical labels, tactical status indicators, sharply structured panels and restrained magenta/cyan/red accents.
- Separate dashboard composition and visual language, sharing the same browser state and functionality as Calm.
- Original artwork and interface design; do not redistribute Cyberpunk 2077 proprietary assets.

## Feature parity target

The following list is the complete product scope. Status must be kept honest in release notes.

### Browser fundamentals
- [x] Multiple tabs and workspaces (prototype implementation).
- [x] Basic back/forward and address/search navigation.
- [x] Local profile persistence, tab-session restoration, bookmarks and history (0.0.3 implementation).
- [x] Tab pinning and manual background-tab sleeping (0.0.3 implementation).
- [ ] Tab islands/groups, drag-and-drop tab ordering, tab search and tab previews.
- [ ] Split-screen / side-by-side tab layout.
- [ ] Multiple browser profiles, guest profile and private sessions.
- [ ] Full session restore, crash restore and sync/Flow across devices.
- [ ] Accessible keyboard navigation, context menus and screen-reader semantics across every browser surface.

### Performance and gaming controls
- [x] Live Lucid-process CPU and resident-memory telemetry plus system-memory summary.
- [x] Manual background-tab sleeping.
- [x] Per-top-level-HTML-navigation bandwidth cap.
- [ ] True process-based per-site CPU budgets and RAM limits. Requires isolated renderer processes and OS-level enforcement.
- [ ] GPU and per-tab telemetry, network usage history, battery saver and automatic tab discard policies.
- [ ] Hot-tab killer, process/task manager and crash diagnostics.
- [ ] Startup, page-load, rendering, memory and battery benchmarks with regression budgets.

### Customization and mods
- [x] Calm/Cyberpunk modes, accent packs and reduced-motion setting.
- [ ] Wallpaper, live wallpaper, opacity/blur, contrast, typography, custom icons, cursors and tab treatments.
- [ ] Sound packs, keyboard sounds, background music and user-controlled audio settings.
- [ ] User-created mods with versioning, signing, permissions, review, safe install/uninstall and rollback.
- [ ] Shader/effect pipeline with GPU and accessibility safeguards.
- [ ] Mod discovery/marketplace only after a secure package format and permission model exist.

### Productivity and integrations
- [x] Quick links, bookmarks, history and direct-file download queue.
- [ ] Full downloads manager with pause/resume, integrity checks and dangerous-file handling.
- [ ] Pinboards/notes, snapshots, tab annotations and share sheets.
- [ ] Sidebar with user-pinned web apps and account integrations.
- [ ] First-class music player controls, media session integration and picture-in-picture.
- [ ] Calendar and configurable personal news.
- [ ] GX Corner-equivalent live game release calendar, deals and news with source attribution and user controls.
- [ ] Video pop-out, translation, reader view and page capture.
- [ ] Cross-device Flow-like transfer with end-to-end encryption and explicit pairing.

### Privacy and security
- [x] First-party top-level navigation protection list (prototype only).
- [ ] Standards-compatible filter-list parser and resource-level ad/tracker blocking.
- [ ] Per-site permissions, camera/microphone/location controls and permission history.
- [ ] Secure DNS / DoH, certificate handling, mixed-content rules and clear security UI.
- [ ] Cookie partitioning, tracking protection, storage controls and private browsing.
- [ ] Safe downloads, phishing/malware reputation, sandboxing and exploit mitigations.
- [ ] VPN is not represented as a simple toggle: any VPN feature requires a real audited provider, routing implementation, privacy policy and leak testing.
- [ ] Public vulnerability reporting, security response process, fuzzing and external security reviews.

### Extensions
- [ ] A documented Lucid extension API with explicit permissions and versioned capabilities.
- [ ] Isolated extension execution, origin boundaries, signed packages and update integrity.
- [ ] Content scripts, extension UI, commands, storage, messaging and compatible WebExtensions subsets.
- [ ] Compatibility claims must be based on actual extension test suites; Firefox/Chrome compatibility must never be assumed.

## Independent engine quality target

Lucid Engine remains an original implementation. UI libraries and general-purpose infrastructure (TLS, image codecs, font shaping, GPU abstraction) may be reused; a complete third-party browser engine or embedded web view may not silently replace Lucid's parser, DOM, CSS, layout or renderer.

### Required engine subsystems
1. URL parser, navigation controller, fetch, HTTP/1.1, HTTP/2 and HTTP/3, TLS, caching, redirects and content decoding.
2. Standards-conformant HTML tokenizer and tree builder, DOM, mutation observers and event dispatch.
3. CSS tokenizer/parser, cascade, inheritance, computed values, selectors, media queries and CSSOM.
4. Inline formatting, block layout, intrinsic sizing, flexbox, grid, tables, positioning, fragmentation and writing modes.
5. Text shaping, font fallback, image decoding, SVG, canvas and a retained display list.
6. Compositor, GPU process, damage tracking, raster cache, scrolling and animation timelines.
7. A JavaScript engine and the browser host environment: DOM bindings, realms, jobs/event loop, promises, timers, fetch, storage and workers.
8. Web APIs: WebAssembly, WebSockets, WebRTC, WebAudio, media, service workers, notifications, file handling and permissions.
9. Accessibility tree, keyboard semantics, selection, find-in-page, developer tools and diagnostics.
10. Origins, same-origin policy, CORS, CSP, CORB/ORB-equivalent response protection, cookie model, sandboxing and site isolation.

## Quality gates — never use marketing claims as test results

- **HTML/CSS:** relevant Web Platform Tests pass; results tracked by test directory and release.
- **JavaScript:** ECMAScript Test262 results tracked by feature and test class.
- **Visual correctness:** reftests, screenshot comparisons, font/text shaping and viewport matrix.
- **Security:** parser and protocol fuzzing, sandbox tests, privilege-boundary tests, dependency review and independent security review.
- **Performance:** cold/warm startup, navigation, DOM/layout/paint, scrolling, memory per tab, CPU/GPU and battery benchmarks.
- **Reliability:** browser process crash recovery, renderer crash recovery, session recovery, long-duration tests and memory-leak testing.
- **Accessibility:** automated and manual keyboard, contrast, semantic tree and screen-reader testing.
- **Compatibility:** maintain a versioned matrix of real sites and workflows, with reproducible failures and no claims based on one successful page.

## Release policy

- Experimental versions may expose only capabilities that are implemented and tested.
- Every feature must be labeled as implemented, experimental, planned or unavailable.
- A green Windows build proves that the code compiles and its tests pass; it does not prove standards compliance, performance parity or security.
- Lucid 1.0 is not released until the engine quality gates, security boundaries, recovery behavior, update system and compatibility criteria have been met.

## Current 0.0.3 implementation notes

The initial GX-style foundations include persistent local preferences/session metadata, bookmarks/history, workspace-aware quick links, pinned tabs, manual tab sleeping, a live process/system metrics panel, a per-navigation HTML transfer governor, top-level navigation protection and direct-file downloads into the user's Downloads/Lucid directory.

Important current limitations:
- The renderer is still a small HTML/CSS subset and has no JavaScript.
- The navigation blocker only checks the top-level URL; it does not block page subresources.
- The network governor limits each top-level HTML transfer independently; it is not a global bandwidth scheduler.
- Memory Saver drops a tab's in-memory parsed document; there are no renderer processes to suspend.
- CPU and memory numbers currently describe the Lucid process, not isolated per-tab renderer processes.
- Downloads are user-initiated direct HTTP/HTTPS downloads and capped at 512 MiB.
- GX Corner shortcuts are not a live data feed.
- The Mod Center currently provides mode/accent/reduced-motion customization, not third-party executable mods.
