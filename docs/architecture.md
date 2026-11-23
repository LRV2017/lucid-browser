# Lucid Engine Architecture

## Product boundaries

Lucid has two cooperating codebases in one Cargo workspace:

- `apps/lucid-browser`: native desktop application, browser chrome, theme system, tab/workspace state, address handling and network request orchestration.
- `crates/lucid-engine`: original web document processing code. It owns its HTML parser, DOM representation, CSS subset, simple layout pass and paint-operation model.

`eframe`/`egui` is used only to create the native desktop application UI and present the engine's paint operations. No embedded web view or existing browser engine is used to render pages.

## Current document pipeline

```text
HTTP(S) response / local HTML
        |
        v
Lucid HTML tokenizer + tree builder
        |
        v
Lucid DOM data model
        |
        +----> collect <style> text
        |             |
        |             v
        |       CSS subset parser
        |             |
        +-------------+
                      v
             selector + style subset
                      |
                      v
             simple vertical layout
                      |
                      v
                paint operations
                      |
                      v
             native UI painter (egui)
```

## Current supported subset

### HTML
- Element start/end tags, text nodes and basic quoted/unquoted attributes.
- Comments and declarations skipped.
- Void-element list for common HTML elements.
- A few common omitted paragraph/list-item end-tag cases.
- Minimal named entity decoding.

This is not the WHATWG HTML parsing algorithm. Complex malformed markup, foster parenting, adoption agency handling, namespaces, templates, scripting insertion modes and full error recovery are not implemented.

### CSS
- Simple selectors: element, class and ID.
- Comma-separated selector lists.
- Small declaration set: display, color, background, font-size, font-weight, font-style, margin, padding and border-radius.
- Basic default styles for headings, paragraphs, anchors, emphasis and code.

Combinators, pseudo-classes/elements, variables, media queries, custom properties, calc(), full shorthand expansion, inheritance rules, cascade layers, animations and the complete CSS cascade are not implemented.

### Layout and painting

The current layout is top-to-bottom block flow with approximate text wrapping. It emits rectangles and text. It does not implement line boxes correctly, inline formatting contexts, intrinsic sizing, flexbox, grid, floats, fragmentation, transforms, stacking contexts, hit testing, selection or a real compositor.

## Process and security architecture target

The current prototype is single-process. Before untrusted-site use, the architecture must move towards isolated renderer processes and a brokered network/service layer. Site isolation, sandbox policies, GPU process boundaries, IPC validation, permission prompts, origin security, same-origin enforcement and robust crash recovery are release blockers, not optional polish.

## Dependencies policy

Using a GUI toolkit, TLS implementation, font rasterizer, image decoder, or GPU abstraction does not turn Lucid into a wrapper browser. Those are infrastructure libraries. The web-platform parser, DOM, CSS style system, layout, paint model and future script bindings remain Lucid-owned. Any future dependency that performs an entire browser-engine subsystem must be reviewed and documented before adoption.
