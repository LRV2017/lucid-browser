# Lucid Browser

**An independent browser-engine experiment with a deliberately clean, interactive desktop browser UI.**

Lucid is being developed as an original HTML/CSS rendering engine and native desktop browser. It does **not** embed Firefox, Chromium, Blink, WebKit, Electron, CEF, or WebView2.

> **Status: experimental 0.0.1 prototype.** The current engine contains a small Lucid-owned HTML tokenizer/tree builder, a limited CSS declaration/selector parser, a simple block layout pass, and paint-command generation. It does not execute JavaScript, implement the full HTML or CSS standards, isolate sites, or safely support complex modern websites yet.

## What exists today

- Native Windows desktop application built with Rust and `eframe`/`egui` (the UI toolkit only; it is not used as a web renderer).
- **Lucid Calm**: restrained graphite-and-lavender interface, with interactive tabs, address/search field, tab collections per workspace, quick links, library panel, settings and command palette.
- **Lucid Cyberpunk**: a distinct tactical terminal-style dashboard with angular panels, dense module readouts, and magenta/cyan system accents inspired by an in-world player HUD. It is an original homage, not copied game artwork.
- Lucid Engine prototype: HTML tokenization/tree construction, basic title/style extraction, simple selectors (`tag`, `.class`, `#id`), a limited set of CSS declarations, vertical block layout and paint operations.
- Local `lucid://demo` page rendered through Lucid Engine.
- Experimental HTTP/HTTPS document fetching for HTML responses. Page JavaScript is not executed; advanced site functionality will not work.
- Unit tests for parsing, style extraction, layout and address normalization.
- Windows GitHub Actions workflow that builds and uploads `lucid-browser.exe`.

## Quick start on Windows 11

### 1. Install Rust

Install Rust using [rustup](https://rustup.rs/) and choose the MSVC toolchain. You also need Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK.

### 2. Build

Open PowerShell in the repository folder:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\build-windows.ps1
```

Or use Cargo directly:

```powershell
cargo build --release -p lucid-browser
```

### 3. Run

```powershell
.\target\release\lucid-browser.exe
```

### 4. Try the engine demo

Enter `lucid://demo` in the address bar. This local demonstration is parsed, styled, laid out and painted by Lucid's own experimental engine code.

## Download a Windows build without compiling locally

Every push to `main` and every manual workflow run triggers the **Build Windows** GitHub Actions workflow. Open the repository's **Actions** tab, select a successful run, and download the `Lucid-Browser-Windows-x64` artifact. The ZIP contains the executable, icon and a readme.

## Architecture

```text
Lucid Browser (native desktop UI)
  ├── Tabs / workspaces / address bar / library / settings
  └── Lucid Engine
       ├── HTML tokenizer + tree builder
       ├── DOM data model
       ├── CSS selector/declaration subset
       ├── computed style subset
       ├── basic block layout
       └── paint-command list
```

See [docs/architecture.md](docs/architecture.md), [docs/roadmap.md](docs/roadmap.md), and [docs/design-system.md](docs/design-system.md).

## Important limitations and safety

**Do not use this prototype for sensitive accounts, payments, password entry, or other security-critical browsing.** The current renderer is incomplete and has not undergone browser-grade sandboxing, site isolation, exploit mitigation, or security review. Fetching arbitrary website HTML is not equivalent to safely supporting arbitrary websites.

JavaScript, cookies, CSP, CORS, service workers, WebAssembly, downloads, extensions, media pipelines and many other browser subsystems are not implemented. The compatibility claims will only expand when features have implementation and automated tests.

## License

MIT. See [LICENSE](LICENSE).
