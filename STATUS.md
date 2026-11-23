# Project Status — 0.0.4 Experimental

Lucid is a Windows desktop browser shell with two distinct page-rendering paths.

## Live website rendering

- Normal HTTP/HTTPS tabs are hosted in Wry child WebViews backed by the Microsoft Edge WebView2 Runtime (Chromium).
- Each tab retains its own WebView while inactive views are hidden; views share the named Lucid WebView2 profile.
- The live web platform handles HTML, CSS, JavaScript, images, fonts, SVG, canvas, audio/video, iframes, subresources, cookies, storage and normal browser networking.
- New-window requests are routed to new Lucid tabs.
- Navigation and document-title events synchronize tab labels, address state and browsing history.
- DevTools and the WebView2 browser-extension capability are enabled.

This is not Google Chrome itself. It uses Microsoft's Chromium-based Edge WebView2 runtime, whose exact version and behavior can differ from a user's Chrome installation. The current Lucid extension UX is not a complete Chrome Web Store or Firefox XPI implementation.

## Lucid-owned user interface and services

- Native desktop UI with Lucid Calm and Lucid Cyberpunk.
- Persistent profile with theme/accent, workspaces, bookmarks, browsing history, quick links, tab session metadata and preferences.
- Tabs, workspace switching, address/search entry, command palette and keyboard shortcuts.
- Pinned tabs and manual background-tab sleeping.
- Library, history, settings and downloads panels.
- GX-style Control Center with live main-process CPU/RAM and system memory summary.
- Gaming shortcut hub.
- Windows build and portable ZIP through GitHub Actions.

Some power features are still prototypes: Lucid Protect is a small top-level host list, not a page-request filter; Memory Saver closes a hidden tab's WebView and reloads it when revisited; and Lucid Corner shortcuts are not a live news/deals feed.

## Original Lucid Engine

- Lucid-owned HTML tokenizer and small tree builder.
- Basic CSS selector/declaration parsing and a small cascade.
- Simple vertical block layout and paint operations.
- Scripts are retained as inert text; no JavaScript runtime or DOM host bindings.
- Used by `lucid://demo`, not the main renderer for arbitrary websites.

## Runtime requirement

The Windows app requires the Microsoft Edge WebView2 Runtime. It is included with Windows 11 on most devices; users on systems without it must install Microsoft's Evergreen Runtime. The GitHub artifact does not bundle the Fixed Version runtime.

## Explicit limitations

- The browser shell is experimental and has not had an independent security audit.
- Lucid-specific permissions UX, safe-browsing/download scanning, site-permission management, crash recovery and privacy controls need work.
- No completed uBlock-style network request interception/filter-list engine.
- No completed extension manager or verified compatibility with Firefox XPI add-ons.
- No verified parity with Google Chrome's account, policy, DRM or extension-store behavior.
- The original Lucid Engine is not a complete HTML/CSS/JavaScript browser engine and is not ready to replace WebView2.
- Accessibility and screen-reader behavior need dedicated testing.

Do not use early builds for sensitive browsing. Every advertised capability must be backed by implementation and tests; a successful build alone does not prove browser compatibility or security.
