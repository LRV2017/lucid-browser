# Lucid Browser Architecture

## Product boundaries

The Rust workspace contains two separate rendering paths:

- `apps/lucid-browser`: native desktop application, browser chrome, theme system, tab/workspace state, profiles, address handling and downloads.
- `crates/lucid-engine`: Lucid-owned experimental HTML parser, DOM representation, CSS subset, basic layout and paint-operation model. It currently powers `lucid://demo` only.
- The normal live web surface is hosted by Wry and rendered by Microsoft's WebView2 Runtime, which uses the Chromium-based Microsoft Edge web platform.

The WebView2 integration is an explicit compatibility choice. It replaces the old top-level-HTML-only fetch-and-paint path for normal HTTP/HTTPS tabs. Lucid's hand-written renderer is not currently responsible for modern websites.

## Live website pipeline

```text
Lucid native UI (tabs / address bar / workspaces / controls)
        |
        v
Wry host and native child WebView
        |
        v
Microsoft Edge WebView2 Runtime (Chromium)
        |
        +---- HTML parsing and DOM
        +---- CSS cascade, layout and paint
        +---- JavaScript and DOM bindings
        +---- images, fonts, SVG, canvas and media
        +---- subresources, iframes and networking
        +---- cookies, storage and browser APIs
        |
        v
Native website surface inside the Lucid content viewport
```

Those web-platform subsystems are implemented by WebView2, not by `crates/lucid-engine`. WebView2's behavior tracks its installed Evergreen Runtime and may differ from the specific Chrome version on a user's machine.

## Tab lifecycle

Each Lucid tab can own a WebView2 child surface. The active tab's child surface is positioned inside the browser content region; inactive surfaces are hidden. Switching tabs reuses their existing view. Closing a tab drops its view. Manual Memory Saver drops hidden WebViews and causes the tab to reload when revisited. All tabs currently use the same named Lucid WebView2 profile so their cookies and site storage are shared in the conventional same-profile way.

Top-level navigation callbacks update the tab URL and session history. Document-title callbacks update the tab title. Requests to open new windows are routed to new Lucid tabs.

## Independent Lucid Engine path

The local demo uses the Lucid-owned pipeline:

```text
HTML string
   |
   v
Lucid HTML tokenizer + small tree builder
   |
   v
Lucid DOM
   |
   +----> <style> collection
                |
                v
          CSS subset parser
                |
                v
          selector and style subset
                |
                v
          simple vertical block layout
                |
                v
          paint operations
                |
                v
          egui painter and basic anchor hit tests
```

### Supported subset

- Basic element start/end tags, text nodes and attributes.
- Comments/declarations skipped, common void elements and limited entity decoding.
- Raw-text preservation for `style` and `script`; scripts remain inert.
- Simple element/class/ID selectors and a small set of CSS declarations.
- Top-to-bottom block layout with approximate text wrapping and basic rectangle/text painting.

This is not the WHATWG HTML parser, a complete CSS implementation, or a JavaScript runtime.

## Navigation and security considerations

WebView2 handles normal page navigation and subresource loading. Lucid's existing top-level hostname list does not filter all network requests, and the former HTML-download bandwidth limiter does not control WebView2 page traffic. These must not be advertised as full request-level ad blocking or total bandwidth governance.

The Windows app depends on the Microsoft Edge WebView2 Runtime. Windows 11 includes the Evergreen Runtime on most devices; production distribution should detect the runtime and guide users to Microsoft's Evergreen installer if it is missing. The portable Lucid ZIP does not bundle the Fixed Version runtime.

The WebView2 Runtime supplies a mature Chromium-based web platform, but Lucid's own permission UI, safe-browsing/download controls, extension management, privacy controls, and crash recovery remain experimental. Do not claim Chrome-equivalent security or compatibility before testing and audit.

## Future fully independent engine

If Lucid later replaces WebView2, the independent engine must implement and pass broad HTML, CSS, DOM, JavaScript, image, media, networking, accessibility and security test suites. That is a multi-year engine program, not a UI change. WebView2 is the current practical website renderer while that research continues.
