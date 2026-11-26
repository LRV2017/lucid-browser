# Lucid Browser Architecture

## Product boundaries

- `apps/lucid-browser`: native eframe/egui desktop shell, tabs, workspaces, address handling, profile, bookmarks, history, downloads, browser controls and current CEF integration.
- `apps/lucid-browser/src/cef_host.rs`: Windows CEF host, browser client callbacks, keyboard bridge and request interception.
- `crates/lucid-engine`: Lucid-owned experimental HTML parsing, DOM data structures, CSS subset, resource loading, layout and paint operation generation.
- CEF currently supplies Chromium web-platform behavior for ordinary websites in the Windows build. It is not WebView2 and must not be represented as code implemented by `lucid-engine`.

## Current navigation/rendering paths

Ordinary website path:

```text
Lucid address bar and tab shell
          |
          v
Native browser process / CEF host
          |
          v
Bundled Chromium Embedded Framework
          |
          +--> Chromium HTML / DOM / CSS / JavaScript / networking / media
          |
          v
Native child browser surface inside Lucid window
```

Independent engine experimentation path:

```text
Lucid demo / engine test input
          |
          v
Lucid resource loader
          |
          v
Lucid HTML parser and early document tree
          |
          v
Limited CSS parser and cascade
          |
          v
Early block layout and paint operations
          |
          v
egui text / shape / image painter
```

The CEF path provides broad compatibility through Chromium, subject to the bundled CEF version, build configuration and platform support. The Lucid path is deliberately separate and experimental. Do not infer that a feature supported by CEF is already implemented in the Lucid engine.

## Lucid engine current scope

- HTML tokenizer/parser prototype and simple document node model.
- Limited style parsing/cascade and basic layout/paint structures.
- Bounded resource loading and selected raster image decoding in the experimental rendering path.
- No standards-complete HTML tree builder, DOM/Web IDL, CSSOM, JavaScript runtime, full CSS layout, Web APIs, browser storage or production media pipeline.

## Target architecture

The long-term engine target and subsystem requirements are maintained in [WEB_PLATFORM_SPEC.md](WEB_PLATFORM_SPEC.md). The planned architecture separates browser, renderer, network, GPU and utility processes, with typed IPC, shared-memory transport where appropriate, origin isolation, permission controls and Windows sandboxing. These are target requirements, not current guarantees.

## Security considerations

CEF's security properties depend on its build, sandbox bootstrap, process configuration and runtime. The independent Lucid renderer currently lacks production-grade process isolation and a complete browser security model. Do not assume that the experimental renderer has Chromium's sandbox or origin protections.

Before replacing CEF, Lucid must demonstrate strict URL handling, TLS verification, Same-Origin Policy, CORS, CSP, mixed-content enforcement, secure cookies/storage, permission prompts, sandboxed renderer processes, validated IPC, safe downloads, crash containment, fuzzing and independent security review. Do not use experimental engine builds for sensitive browsing.
