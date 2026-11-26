# Lucid Browser Engineering Roadmap

The detailed dependency-ordered plan is maintained in [docs/roadmap.md](roadmap.md). The complete capability checklist and acceptance gates are in [Web Platform and Browser Engine Specification](WEB_PLATFORM_SPEC.md).

Lucid's native desktop shell and Chromium compatibility path are distinct from the Lucid-owned experimental engine. Chromium/CEF capabilities must not be counted as features implemented by `crates/lucid-engine`.

## Product quality bar

The ambition is a polished native browser shell and, over time, a standards-based engine with compatibility, performance, accessibility and security measured against established conformance suites. This is a multi-year engineering program.

## Current verified foundations

- [x] Native browser shell, tabs, address bar and workspace UI.
- [x] Windows CEF integration for Chromium-powered website compatibility.
- [x] Initial request interception and network filter-list ad blocking.
- [x] Experimental Lucid-owned HTML/CSS/layout/paint code.

## Long-term release gates

- [ ] WHATWG HTML, DOM, CSS and Web API conformance.
- [ ] ECMAScript conformance through Test262.
- [ ] Web Platform Tests coverage and published failure tracking.
- [ ] Differential rendering tests and representative site compatibility.
- [ ] GPU/media conformance and performance budgets.
- [ ] Multi-process site isolation, Windows sandbox and security audit.
- [ ] Accessibility, DevTools, extensions, PDF/printing and session recovery.

No capability is considered complete until it is implemented, tested and accurately represented in the UI and documentation. A successful compile is not proof of web compatibility or security.
