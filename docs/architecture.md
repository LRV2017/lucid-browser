# Lucid Engine Architecture

## Product boundaries

Lucid has two cooperating codebases in one Cargo workspace:

- `apps/lucid-browser`: native desktop application, browser chrome, theme system, tab/workspace state, address handling and network request orchestration.
- `crates/lucid-engine`: original web document processing code. It owns its HTML parser, DOM representation, CSS subset, simple layout pass and paint-operation model.

`eframe`/`egui` is used only to create the native desktop application UI and present Lucid Engine paint operations. No embedded web view or existing browser engine is used to render pages.

## Current document pipeline

```text
HTTP(S) response / local demo
        |
        v
Lucid HTML tokenizer + tree builder subset
        |
        v
Lucid DOM data model
        |
        +----> collect <style> raw text
        |             |
        |             v
        |       CSS subset parser
        |             |
        +-------------+
                      v
             selector + style subset
                      |
                      v
             simple vertical block layout
                      |
                      v
                paint operations
                      |
                      v
             native UI painter + link hit tests
```

## Current supported subset

### HTML
- Element start/end tags, text nodes and basic quoted/unquoted attributes.
- Comments and declarations skipped.
- Common void elements.
- A few omitted paragraph/list-item end-tag cases.
- Minimal named entity decoding.
- Raw-text preservation for `style` and `script` blocks; scripts are inert and never executed.

This is not the WHATWG HTML parsing algorithm. Complex malformed markup, foster parenting, adoption agency handling, namespaces, templates, scripting insertion modes and full error recovery are not implemented.

### CSS
- Simple selectors: element, class and ID.
- Comma-separated selector lists.
- Small declaration set: display, color, background, font-size, font-weight, font-style, margin, padding and border-radius.
- Basic default styles and retained anchor href values.

Combinators, pseudo-classes/elements, variables, media queries, custom properties, calc(), full shorthand expansion, inheritance rules, cascade layers, animations and the complete CSS cascade are not implemented.

### Layout and painting

The current layout is top-to-bottom block flow with approximate text wrapping. It emits rectangles and text. It does not implement line boxes correctly, inline formatting contexts, intrinsic sizing, flexbox, grid, floats, fragmentation, transforms, stacking contexts, text selection or a real compositor. Anchor paint operations retain href metadata so the desktop UI can hit-test and navigate basic HTTP/HTTPS links.

## Navigation and network guardrails

Each tab carries a request generation. A navigation increments that generation, and late responses are discarded if a newer request has superseded them. HTTP response bodies are capped at 5 MiB even if the server omits or misreports Content-Length. Unsupported explicit schemes (including `file:` and `javascript:`) are not opened. This is not a security sandbox; external network requests still need a comprehensive browser security model.

## Process and security architecture target

The current prototype is single-process. Before untrusted-site use, the architecture must move towards isolated renderer processes and a brokered network/service layer. Site isolation, sandbox policies, GPU process boundaries, IPC validation, permission prompts, origin security, same-origin enforcement and robust crash recovery are release blockers, not optional polish.

## Dependencies policy

Using a GUI toolkit, TLS implementation, font rasterizer, image decoder, or GPU abstraction does not turn Lucid into a wrapper browser. Those are infrastructure libraries. The web-platform parser, DOM, CSS style system, layout, paint model and future script bindings remain Lucid-owned. Any future dependency that performs an entire browser-engine subsystem must be reviewed and documented before adoption.
