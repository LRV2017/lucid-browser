# Lucid Browser Roadmap

This is an engineering sequence with explicit quality gates, not a promise of dates or a claim of parity with mature browsers.

## 0.0.2 — Correctness guardrails (current)
- [x] Native desktop shell and two distinct themes.
- [x] Original HTML tokenizer/tree-builder prototype.
- [x] Limited CSS cascade, block layout and paint operations.
- [x] Inert raw-text handling for style/script contents.
- [x] Anchor href preservation, click hit-testing and relative HTTP/HTTPS URL resolution.
- [x] Per-tab navigation generations to discard stale responses.
- [x] Bounded HTML response reads and no arbitrary local-file URL access.
- [x] Automated tests and Windows build workflow.

## 0.0.3 — HTML parsing
- Token stream with source locations and deterministic diagnostics.
- Improve tree construction and malformed-markup recovery.
- Numeric references and broader entity handling.
- Raw-text and RCDATA parsing states.
- Parser fuzzing and regression corpus.

## 0.0.4 — DOM and URLs
- Stable node identifiers and document ownership.
- DOM traversal, mutation, attributes and text content.
- URL parsing, document base URL and navigation lifecycle.
- Fragment navigation, link targets and error pages.

## 0.0.5 — CSS style system
- Token-based CSS parsing and error recovery.
- Selector matching, specificity, cascade order and inheritance.
- Computed/used values and stylesheet loading.
- Media queries and viewport changes.
- CSS conformance coverage.

## 0.0.6 — Layout and rendering
- Inline formatting contexts, line boxes and font shaping.
- Box model and intrinsic sizing.
- Flexbox, then grid.
- Positioning, overflow, stacking contexts and scrolling.
- Retained display lists, invalidation and GPU compositing.
- Pixel/reftest comparisons.

## 0.0.7 — JavaScript and Web APIs
- Make an explicit language-runtime decision.
- Define DOM bindings and browser event-loop semantics.
- Implement events, timers, fetch, URL, storage and other APIs incrementally.
- Test JavaScript with Test262 and browser APIs with WPT.

## 0.0.8 — Networking and security
- HTTP cache, cookies and content decoding.
- Origin model, CSP, CORS, mixed-content and certificate behavior.
- Permissions and downloads.
- Multi-process renderer, sandboxing, IPC validation and site isolation.
- Threat model, fuzzing, dependency review and external security review.

## 0.0.9 — Browser capabilities
- Profiles, session restore, bookmarks, history and downloads.
- Extension API and isolated extension process architecture.
- Native ad/tracker filter engine.
- Media pipeline, accessibility, keyboard support and developer tools.
- Performance monitoring and privacy dashboard.

## 1.0 — Release requirements

No 1.0 release until the engine demonstrates broad standards conformance, reliable rendering of representative modern websites, JavaScript and media support, accessibility, security isolation, crash recovery, update strategy, repeatable performance benchmarks and a public vulnerability-reporting process.
