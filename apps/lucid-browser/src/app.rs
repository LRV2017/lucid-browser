mod downloads;
#[cfg(target_os = "windows")]
#[path = "cef_host.rs"]
mod cef_host;
mod performance;
mod persistence;
mod protection;

use eframe::egui::{self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit, Vec2};
use downloads::{DownloadUpdate, run_download};
use performance::{PerformanceMonitor, format_bytes};
use persistence::{BrowserProfile, SavedTab};
use std::time::Instant;
use std::collections::BTreeMap;
use lucid_engine::{Document, Node, PaintOp, Stylesheet, collect_styles, document_title, layout_document, parse_html};
use lucid_engine::resource::{ResourceKind, ResourceLoader, discover_document_resources, load_document_stylesheets};
use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme { Calm, Cyberpunk }

#[derive(Clone, Copy)]
struct Palette {
    background: Color32,
    surface: Color32,
    surface_raised: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    accent: Color32,
    accent_alt: Color32,
    good: Color32,
}

impl Theme {
    fn palette(self) -> Palette {
        match self {
            Theme::Calm => Palette {
                background: Color32::from_rgb(17, 15, 22),
                surface: Color32::from_rgb(27, 24, 34),
                surface_raised: Color32::from_rgb(37, 32, 46),
                border: Color32::from_rgb(57, 49, 69),
                text: Color32::from_rgb(244, 240, 250),
                muted: Color32::from_rgb(164, 155, 180),
                accent: Color32::from_rgb(183, 153, 236),
                accent_alt: Color32::from_rgb(132, 113, 181),
                good: Color32::from_rgb(137, 205, 174),
            },
            Theme::Cyberpunk => Palette {
                background: Color32::from_rgb(8, 9, 14),
                surface: Color32::from_rgb(17, 18, 27),
                surface_raised: Color32::from_rgb(29, 20, 34),
                border: Color32::from_rgb(81, 40, 75),
                text: Color32::from_rgb(244, 244, 249),
                muted: Color32::from_rgb(145, 151, 169),
                accent: Color32::from_rgb(255, 65, 119),
                accent_alt: Color32::from_rgb(53, 221, 210),
                good: Color32::from_rgb(100, 226, 183),
            },
        }
    }
}

#[derive(Clone)]
struct EngineImage { width: usize, height: usize, rgba: Vec<u8> }

#[derive(Clone)]
struct LoadedPage {
    url: String,
    title: String,
    document: Document,
    stylesheet: Stylesheet,
    images: BTreeMap<String, EngineImage>,
}

struct Tab {
    id: u64,
    workspace: String,
    title: String,
    url: String,
    page: Option<LoadedPage>,
    error: Option<String>,
    history: Vec<String>,
    history_index: usize,
    request_generation: u64,
    loading: bool,
    pinned: bool,
    discarded: bool,
    #[cfg(target_os = "windows")]
    cef_browser: Option<cef::Browser>,
    #[cfg(target_os = "windows")]
    cef_client: Option<cef::Client>,
    #[cfg(target_os = "windows")]
    cef_child_hwnd: Option<isize>,
}

impl Tab {
    fn new(id: u64, workspace: String) -> Self {
        Self { id, workspace, title: "New tab".into(), url: "lucid://home".into(), page: None, error: None, history: vec!["lucid://home".into()], history_index: 0, request_generation: 0, loading: false, pinned: false, discarded: false,
            #[cfg(target_os = "windows")]
            cef_browser: None,
            #[cfg(target_os = "windows")]
            cef_client: None,
            #[cfg(target_os = "windows")]
            cef_child_hwnd: None,
        }
    }
}

enum BrowserEvent {
    PageLoaded { tab_id: u64, generation: u64, page: LoadedPage },
    NavigationFailed { tab_id: u64, generation: u64, message: String },
    CefAddressChanged { tab_id: u64, url: String },
    CefTitleChanged { tab_id: u64, title: String },
}

struct DownloadItem {
    id: u64,
    url: String,
    received: u64,
    total: Option<u64>,
    status: String,
    saved_path: Option<String>,
}

struct LucidApp {
    theme: Theme,
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: u64,
    next_request_generation: u64,
    address: String,
    tx: Sender<BrowserEvent>,
    rx: Receiver<BrowserEvent>,
    image_textures: BTreeMap<(u64, String), egui::TextureHandle>,
    cef_view_rect: Option<egui::Rect>,
    #[cfg(target_os = "windows")]
    parent_hwnd: Option<isize>,
    #[cfg(target_os = "windows")]
    cef_app: Option<cef_host::LucidCefApp>,
    status: String,
    show_library: bool,
    show_settings: bool,
    show_history: bool,
    show_gx_control: bool,
    show_mods: bool,
    show_corner: bool,
    show_downloads: bool,
    show_command_palette: bool,
    command_query: String,
    show_find: bool,
    find_query: String,
    find_focus: bool,
    active_workspace: String,
    workspace_name_input: String,
    bookmarks_bar: bool,
    profile: BrowserProfile,
    performance: PerformanceMonitor,
    session_restore_started: bool,
    last_persist: Instant,
    download_input: String,
    download_tx: Sender<DownloadUpdate>,
    download_rx: Receiver<DownloadUpdate>,
    downloads: Vec<DownloadItem>,
    next_download_id: u64,
}

impl Default for LucidApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let (download_tx, download_rx) = mpsc::channel();
        let profile = BrowserProfile::load();
        let theme = if profile.theme.eq_ignore_ascii_case("cyberpunk") { Theme::Cyberpunk } else { Theme::Calm };
        let active_workspace = profile.active_workspace.clone();
        let mut tabs: Vec<Tab> = profile.session_tabs.iter().map(|saved| {
            let mut tab = Tab::new(0, saved.workspace.clone());
            tab.title = saved.title.clone();
            tab.url = saved.url.clone();
            tab.pinned = saved.pinned;
            tab.history = if saved.history.is_empty() { vec![saved.url.clone()] } else { saved.history.clone() };
            tab.history_index = saved.history_index.min(tab.history.len().saturating_sub(1));
            tab
        }).collect();
        if tabs.is_empty() { tabs.push(Tab::new(1, active_workspace.clone())); }
        for (index, tab) in tabs.iter_mut().enumerate() { tab.id = index as u64 + 1; }
        let address = tabs.first().map(|tab| tab.url.clone()).unwrap_or_else(|| "lucid://home".into());

        Self {
            theme, tabs, active_tab: 0, next_tab_id: 100, next_request_generation: 1,
            address, tx, rx,
            image_textures: BTreeMap::new(),
            cef_view_rect: None,
            #[cfg(target_os = "windows")]
            parent_hwnd: None,
            #[cfg(target_os = "windows")]
            cef_app: None,
            status: "Your browser, in its natural state.".into(),
            show_library: false, show_settings: false, show_history: false,
            show_gx_control: false, show_mods: false, show_corner: false, show_downloads: false,
            show_command_palette: false, command_query: String::new(),
            show_find: false, find_query: String::new(), find_focus: false,
            active_workspace, workspace_name_input: String::new(),
            bookmarks_bar: profile.bookmarks_bar, profile,
            performance: PerformanceMonitor::new(),
            session_restore_started: false, last_persist: Instant::now() - Duration::from_secs(5),
            download_input: String::new(), download_tx, download_rx, downloads: Vec::new(), next_download_id: 1,
        }
    }
}

impl LucidApp {
    #[cfg(target_os = "windows")]
    fn new(
        cc: &eframe::CreationContext<'_>,
        main_args: &cef::MainArgs,
        sandbox_info: *mut u8,
    ) -> Result<Self, String> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let handle = cc.window_handle().map_err(|error| format!("Could not access Lucid's native window: {error}"))?;
        let parent_hwnd = match handle.as_raw() {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => return Err("Lucid's Chromium host currently requires a Win32 window handle.".into()),
        };

        let cef_app = cef_host::initialize(cc.egui_ctx.clone(), main_args, sandbox_info)?;
        let mut app = Self::default();
        app.parent_hwnd = Some(parent_hwnd);
        app.cef_app = Some(cef_app);
        Ok(app)
    }

    #[cfg(not(target_os = "windows"))]
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn active(&self) -> &Tab { &self.tabs[self.active_tab] }

    fn close_panels(&mut self) {
        self.show_library = false;
        self.show_settings = false;
        self.show_history = false;
        self.show_gx_control = false;
        self.show_mods = false;
        self.show_corner = false;
        self.show_downloads = false;
    }

    fn open_panel(&mut self, panel: &str) {
        self.close_panels();
        match panel {
            "library" => self.show_library = true,
            "history" => self.show_history = true,
            "control" => self.show_gx_control = true,
            "mods" => self.show_mods = true,
            "corner" => self.show_corner = true,
            "downloads" => self.show_downloads = true,
            "settings" => self.show_settings = true,
            _ => {}
        }
    }

    fn persist_profile(&mut self) {
        self.profile.theme = if self.theme == Theme::Calm { "calm" } else { "cyberpunk" }.into();
        self.profile.active_workspace = self.active_workspace.clone();
        self.profile.bookmarks_bar = self.bookmarks_bar;
        self.profile.session_tabs = self.tabs.iter().map(|tab| SavedTab {
            title: tab.title.clone(),
            url: tab.url.clone(),
            workspace: tab.workspace.clone(),
            pinned: tab.pinned,
            history: tab.history.clone(),
            history_index: tab.history_index,
        }).collect();
        self.profile.session_tabs.truncate(30);
        let _ = self.profile.save();
        self.last_persist = Instant::now();
    }

    fn persist_if_due(&mut self) {
        if self.last_persist.elapsed() >= Duration::from_secs(3) {
            self.persist_profile();
        }
    }

    fn toggle_current_bookmark(&mut self) {
        let tab = self.active();
        if tab.url == "lucid://home" || tab.url == "lucid://demo" { return; }
        let title = tab.title.clone();
        let url = tab.url.clone();
        self.profile.toggle_bookmark(title, url.clone());
        self.status = if self.profile.is_bookmarked(&url) { "Added to Lucid Library".into() } else { "Removed from Lucid Library".into() };
    }

    fn sleep_background_tabs(&mut self) {
        let active_id = self.active().id;
        let mut slept = 0usize;
        for tab in self.tabs.iter_mut() {
            if tab.id != active_id && !tab.pinned {
                if tab.page.take().is_some() { slept += 1; }
                #[cfg(target_os = "windows")]
                if let Some(browser) = tab.cef_browser.take() {
                    if let Some(host) = browser.host() { host.close_browser(1); }
                    tab.cef_client = None;
                    tab.cef_child_hwnd = None;
                    slept += 1;
                }
                tab.discarded = true;
            }
        }
        self.status = format!("Memory Saver slept {slept} background tab(s)");
    }

    fn poll_downloads(&mut self) {
        while let Ok(update) = self.download_rx.try_recv() {
            match update {
                DownloadUpdate::Progress { id, received, total } => {
                    if let Some(item) = self.downloads.iter_mut().find(|item| item.id == id) {
                        item.received = received; item.total = total; item.status = "Downloading".into();
                    }
                }
                DownloadUpdate::Completed { id, path, bytes } => {
                    if let Some(item) = self.downloads.iter_mut().find(|item| item.id == id) {
                        item.received = bytes; item.total = Some(bytes); item.status = "Complete".into();
                        item.saved_path = Some(path.display().to_string());
                    }
                }
                DownloadUpdate::Failed { id, message } => {
                    if let Some(item) = self.downloads.iter_mut().find(|item| item.id == id) {
                        item.status = format!("Failed: {message}");
                    }
                }
            }
        }
    }
    fn active_mut(&mut self) -> &mut Tab { &mut self.tabs[self.active_tab] }

    fn new_tab(&mut self) {
        let tab = Tab::new(self.next_tab_id, self.active_workspace.clone());
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.address.clear();
        self.status = format!("New tab · {} workspace", self.active_workspace);
    }

    #[cfg(target_os = "windows")]
    fn sync_cef_views(&mut self, pixels_per_point: f32) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, ShowWindow, SW_HIDE, SW_SHOWNA, SWP_NOACTIVATE, SWP_NOZORDER,
        };
        let active_id = self.active().id;
        for tab in &self.tabs {
            let Some(child) = tab.cef_child_hwnd else { continue };
            if tab.id == active_id && self.cef_view_rect.is_some() {
                let rect = self.cef_view_rect.unwrap();
                let x = (rect.min.x * pixels_per_point).round() as i32;
                let y = (rect.min.y * pixels_per_point).round() as i32;
                let width = (rect.width() * pixels_per_point).round().max(1.0) as i32;
                let height = (rect.height() * pixels_per_point).round().max(1.0) as i32;
                unsafe {
                    SetWindowPos(child as _, std::ptr::null_mut(), x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
                    ShowWindow(child as _, SW_SHOWNA);
                }
            } else {
                unsafe { ShowWindow(child as _, SW_HIDE); }
            }
        }
    }

    fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            let closed_tab_id = self.tabs[index].id;
            self.tabs.remove(index);
            self.image_textures.retain(|(tab_id, _), _| *tab_id != closed_tab_id);
            #[cfg(target_os = "windows")]
            {
                if let Some(browser) = self.tabs.get(index).and_then(|tab| tab.cef_browser.as_ref()) {
                    if let Some(host) = browser.host() { host.close_browser(0); }
                }
            }
        }
        if let Some((active_index, _)) = self.tabs.iter().enumerate().find(|(_, tab)| tab.workspace == self.active_workspace) {
            self.active_tab = active_index;
        } else {
            self.new_tab();
        }
        self.address = self.active().url.clone();
    }

    fn switch_workspace(&mut self, workspace: &str) {
        self.active_workspace = workspace.to_owned();
        if let Some((index, tab)) = self.tabs.iter().enumerate().find(|(_, tab)| tab.workspace == self.active_workspace) {
            self.active_tab = index;
            self.address = tab.url.clone();
        } else {
            self.new_tab();
        }
        self.status = format!("{} workspace", self.active_workspace);
    }

    fn navigate(&mut self, input: &str, ctx: egui::Context, push_history: bool) {
        let url = normalize_address(input);
        let (tab_id, generation) = {
            let tab = self.active_mut();
            tab.request_generation = tab.request_generation.wrapping_add(1).max(1);
            tab.url = url.clone();
            tab.title = if url == "lucid://home" { "New tab".into() } else { "Loading…".into() };
            tab.page = None;
            tab.error = None;
            tab.loading = is_web_url(&url) || url == "lucid://demo";
            tab.discarded = false;
            if push_history {
                if tab.history_index + 1 < tab.history.len() {
                    tab.history.truncate(tab.history_index + 1);
                }
                if tab.history.last() != Some(&url) { tab.history.push(url.clone()); }
                tab.history_index = tab.history.len().saturating_sub(1);
            }
            (tab.id, tab.request_generation)
        };

        self.image_textures.retain(|(image_tab_id, _), _| *image_tab_id != tab_id);
        self.address = url.clone();
        self.close_panels();

        if url == "lucid://home" {
            self.status = "Ready".into();
            ctx.request_repaint();
            return;
        }

        if url != "lucid://demo" && is_web_url(&url) {
            if self.profile.navigation_protection && protection::is_blocked_navigation(&url) {
                let tab = self.active_mut();
                tab.error = Some("Blocked by Lucid Protect: this address matches the built-in navigation blocklist.".into());
                tab.loading = false;
                self.status = "Navigation blocked".into();
                return;
            }
            #[cfg(target_os = "windows")]
            {
                if self.active().cef_browser.is_some() {
                    if let Some(frame) = self.active().cef_browser.as_ref().and_then(|browser| browser.main_frame()) {
                        let cef_url = cef::CefString::from(url.as_str());
                        frame.load_url(Some(&cef_url));
                        self.status = format!("Loading {url} · Chromium");
                    }
                } else {
                    let Some(parent_hwnd) = self.parent_hwnd else {
                        self.active_mut().error = Some("Lucid could not obtain its native Windows host window.".into());
                        self.active_mut().loading = false;
                        return;
                    };
                    let callbacks = cef_host::CefCallbacks {
                        tab_id,
                        sender: self.tx.clone(),
                        context: ctx.clone(),
                    };
                    match cef_host::create_browser(parent_hwnd, callbacks, &url) {
                        Ok(hosted) => {
                            let tab = self.active_mut();
                            tab.cef_browser = Some(hosted.browser);
                            tab.cef_client = Some(hosted.client);
                            tab.cef_child_hwnd = Some(hosted.child_hwnd);
                            self.status = format!("Loading {url} · Chromium");
                        }
                        Err(message) => {
                            let tab = self.active_mut();
                            tab.error = Some(message);
                            tab.loading = false;
                            self.status = "Chromium could not start this page".into();
                        }
                    }
                }
                ctx.request_repaint();
                return;
            }
            #[cfg(not(target_os = "windows"))]
            {
                let tab = self.active_mut();
                tab.error = Some("The Chromium renderer is currently packaged for Windows only.".into());
                tab.loading = false;
                return;
            }
        }

        self.status = "Rendering Lucid Engine preview…".into();
        let sender = self.tx.clone();
        let repaint = ctx.clone();
        let protection_enabled = false;
        let network_limit = 0;
        thread::spawn(move || {
            let event = match load_page(&url, protection_enabled, network_limit) {
                Ok(page) => BrowserEvent::PageLoaded { tab_id, generation, page },
                Err(message) => BrowserEvent::NavigationFailed { tab_id, generation, message },
            };
            let _ = sender.send(event);
            repaint.request_repaint();
        });
        ctx.request_repaint();
    }

    fn poll_loads(&mut self, ctx: egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                BrowserEvent::CefAddressChanged { tab_id, url } => {
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if !is_web_url(&url) { continue; }
                    let changed = tab.url != url;
                    tab.url = url.clone();
                    tab.loading = false;
                    if changed && tab.history.last() != Some(&url) {
                        if tab.history_index + 1 < tab.history.len() { tab.history.truncate(tab.history_index + 1); }
                        tab.history.push(url.clone());
                        tab.history_index = tab.history.len().saturating_sub(1);
                    }
                    if self.tabs.get(self.active_tab).is_some_and(|active| active.id == tab_id) {
                        self.address = url.clone();
                    }
                }
                BrowserEvent::CefTitleChanged { tab_id, title } => {
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if !title.trim().is_empty() { tab.title = title.clone(); }
                    tab.loading = false;
                    self.profile.add_history(tab.title.clone(), tab.url.clone());
                    if self.tabs.get(self.active_tab).is_some_and(|active| active.id == tab_id) {
                        self.status = "Page ready · Chromium engine".into();
                    }
                }
                BrowserEvent::PageLoaded { tab_id, generation, page } => {
                    let is_active_tab = self.tabs.get(self.active_tab).is_some_and(|tab| tab.id == tab_id);
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if tab.request_generation != generation { continue; }
                    tab.title = page.title.clone();
                    tab.url = page.url.clone();
                    tab.loading = false;
                    tab.error = None;
                    tab.discarded = false;
                    for (source, image) in &page.images {
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [image.width, image.height], &image.rgba
                        );
                        let texture = ctx.load_texture(
                            format!("lucid-{tab_id}-{source}"),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.image_textures.insert((tab_id, source.clone()), texture);
                    }
                    tab.page = Some(page.clone());
                    self.profile.add_history(page.title.clone(), page.url.clone());
                    if is_active_tab {
                        self.address = page.url.clone();
                        self.status = format!("Rendered by Lucid Engine · {} image(s)", page.images.len());
                    }
                }
                BrowserEvent::NavigationFailed { tab_id, generation, message } => {
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if tab.request_generation != generation { continue; }
                    tab.error = Some(message);
                    tab.loading = false;
                    self.status = "Page could not be loaded".into();
                }
            }
        }
    }

    fn find_in_page(&mut self) {
        #[cfg(target_os = "windows")]
        if let Some(browser) = self.active().cef_browser.as_ref() {
            if let Some(frame) = browser.main_frame() {
                let script = cef::CefString::from(format!("window.find({});", serde_json::to_string(&self.find_query).unwrap_or_else(|_| "\"\"".into())).as_str());
                let source = cef::CefString::from("lucid://find");
                frame.execute_java_script(Some(&script), Some(&source), 0);
            }
            return;
        }
        let count = self.active().page.as_ref()
            .map(|page| count_text_matches(&page.document.children, &self.find_query))
            .unwrap_or(0);
        self.status = if self.find_query.trim().is_empty() {
            "Find closed".into()
        } else {
            format!("{count} match(es)")
        };
    }

    fn execute_command(&mut self, command: &str, ctx: egui::Context) {
        match command {
            "New tab" => self.new_tab(),
            "Toggle theme" => self.theme = if self.theme == Theme::Calm { Theme::Cyberpunk } else { Theme::Calm },
            "Open library" => self.open_panel("library"),
            "History" => self.open_panel("history"),
            "GX Control" => self.open_panel("control"),
            "Mods" => self.open_panel("mods"),
            "GX Corner" => self.open_panel("corner"),
            "Downloads" => self.open_panel("downloads"),
            "Settings" => self.open_panel("settings"),
            "Home" => self.navigate("lucid://home", ctx, true),
            "Bookmark current page" => self.toggle_current_bookmark(),
            "Sleep background tabs" => self.sleep_background_tabs(),
            _ => {}
        }
        self.show_command_palette = false;
        self.command_query.clear();
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("◈").size(19.0).strong().color(palette.accent));
            ui.label(RichText::new("Lucid").size(18.0).strong().color(palette.text));
            ui.add_space(6.0);
            ui.label(RichText::new("NATIVE ENGINE").size(9.0).monospace().color(palette.muted));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.menu_button(RichText::new("⋯").size(18.0).color(palette.muted), |ui| {
                    for (label, panel) in [
                        ("Library", "library"), ("History", "history"), ("Downloads", "downloads"),
                        ("Performance", "control"), ("Appearance", "mods"), ("Settings", "settings"),
                    ] {
                        if ui.button(label).clicked() { self.open_panel(panel); ui.close(); }
                    }
                    ui.separator();
                    if ui.button("Toggle Calm / Cyberpunk").clicked() {
                        self.theme = if self.theme == Theme::Calm { Theme::Cyberpunk } else { Theme::Calm };
                        ui.close();
                    }
                    if ui.button("Sleep background tabs").clicked() { self.sleep_background_tabs(); ui.close(); }
                    if ui.button("Command palette  Ctrl+K").clicked() { self.show_command_palette = true; ui.close(); }
                });
            });
        });
        ui.add_space(5.0);
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            let mut close_index = None;
            let mut set_pin: Option<(usize, bool)> = None;
            let mut indices: Vec<usize> = self.tabs.iter().enumerate()
                .filter(|(_, tab)| tab.workspace == self.active_workspace)
                .map(|(index, _)| index).collect();
            indices.sort_by_key(|index| !self.tabs[*index].pinned);

            for index in indices {
                let tab = &self.tabs[index];
                let selected = index == self.active_tab;
                let fill = if selected { palette.surface_raised } else { palette.background };
                let label = format!("{}{}", if tab.pinned { "◆ " } else { "" }, tab.title.chars().take(18).collect::<String>());
                let button = egui::Button::new(RichText::new(label).size(12.0).color(if selected { palette.text } else { palette.muted }))
                    .fill(fill).stroke(Stroke::new(if selected { 1.0 } else { 0.0 }, palette.border))
                    .corner_radius(if self.theme == Theme::Calm { 9 } else { 2 });
                let response = ui.add(button);
                if response.clicked() {
                    self.active_tab = index;
                    self.address = self.tabs[index].url.clone();
                }
                response.context_menu(|ui| {
                    if ui.button(if tab.pinned { "Unpin tab" } else { "Pin tab" }).clicked() {
                        set_pin = Some((index, !tab.pinned));
                        ui.close();
                    }
                    if ui.button("Close tab").clicked() {
                        close_index = Some(index);
                        ui.close();
                    }
                });
            }
            if let Some((index, pinned)) = set_pin { self.tabs[index].pinned = pinned; }
            if let Some(index) = close_index { self.close_tab(index); }
            if ui.add(egui::Button::new(RichText::new("＋").size(18.0).color(palette.accent)).fill(palette.surface).corner_radius(9)).clicked() {
                self.new_tab();
            }
        });
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.horizontal(|ui| {
            let can_back = self.active().history_index > 0;
            if ui.add_enabled(can_back, egui::Button::new("‹").fill(palette.surface).corner_radius(9)).clicked() {
                let tab = self.active_mut();
                tab.history_index = tab.history_index.saturating_sub(1);
                let target = tab.history[tab.history_index].clone();
                self.navigate(&target, ctx.clone(), false);
            }
            let can_forward = self.active().history_index + 1 < self.active().history.len();
            if ui.add_enabled(can_forward, egui::Button::new("›").fill(palette.surface).corner_radius(9)).clicked() {
                let tab = self.active_mut();
                tab.history_index = (tab.history_index + 1).min(tab.history.len() - 1);
                let target = tab.history[tab.history_index].clone();
                self.navigate(&target, ctx.clone(), false);
            }
            if ui.add(egui::Button::new("↻").fill(palette.surface).corner_radius(9)).clicked() {
                let target = self.active().url.clone();
                self.navigate(&target, ctx.clone(), false);
            }
            let address_width = (ui.available_width() - 104.0).max(160.0);
            let address = ui.add_sized([address_width, 36.0],
                TextEdit::singleline(&mut self.address)
                    .id(egui::Id::new("lucid-address-bar"))
                    .hint_text("Search or enter an address")
                    .desired_width(f32::INFINITY)
                    .margin(Vec2::new(12.0, 8.0)));
            if address.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.navigate(&self.address.clone(), ctx.clone(), true);
            }
            let bookmarked = self.profile.is_bookmarked(&self.active().url);
            if ui.add(egui::Button::new(RichText::new(if bookmarked { "★" } else { "☆" }).color(palette.accent))
                .fill(palette.surface).corner_radius(9)).on_hover_text("Save to Library").clicked() {
                self.toggle_current_bookmark();
            }
            ui.menu_button(RichText::new("＋").color(palette.muted), |ui| {
                if ui.button("New tab").clicked() { self.new_tab(); ui.close(); }
                if ui.button("Find on page  Ctrl+F").clicked() { self.show_find = true; self.find_focus = true; ui.close(); }
                if ui.button("Open Library").clicked() { self.open_panel("library"); ui.close(); }
                if ui.button("Open Home").clicked() { self.navigate("lucid://home", ctx.clone(), true); ui.close(); }
            });
        });
    }

    fn draw_home(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        if self.theme == Theme::Cyberpunk {
            self.draw_cyberpunk_home(ui, palette, ctx);
            return;
        }
        ui.add_space(74.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("◈").size(38.0).color(palette.accent));
            ui.add_space(7.0);
            ui.label(RichText::new("Lucid").size(34.0).strong().color(palette.text));
            ui.label(RichText::new("A clearer place to browse.").size(13.0).color(palette.muted));
            ui.add_space(22.0);
            let search = ui.add_sized([ui.available_width().min(640.0), 46.0],
                TextEdit::singleline(&mut self.address)
                    .hint_text("Search the web or enter a website")
                    .margin(Vec2::new(16.0, 12.0)));
            if search.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.navigate(&self.address.clone(), ctx.clone(), true);
            }
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {
                for link in self.profile.quick_links.clone() {
                    let label = link.label.clone();
                    let url = link.url.clone();
                    if ui.add(egui::Button::new(RichText::new(label).color(palette.text))
                        .fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(9)).clicked() {
                        self.navigate(&url, ctx.clone(), true);
                    }
                }
            });
            ui.add_space(24.0);
            ui.label(RichText::new("Try the engine preview:  lucid://demo").size(11.0).color(palette.muted));
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Workspace").size(10.0).color(palette.muted));
                egui::ComboBox::from_id_salt("lucid-workspace")
                    .selected_text(&self.active_workspace)
                    .show_ui(ui, |ui| {
                        for workspace in self.profile.workspaces.clone() {
                            if ui.selectable_label(self.active_workspace == workspace, &workspace).clicked() {
                                self.switch_workspace(&workspace);
                            }
                        }
                    });
            });
        });
    }

    fn draw_cyberpunk_home(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.add_space(22.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("PERSONAL TERMINAL  //  NIGHT CITY NETWORK").size(10.0).strong().monospace().color(palette.accent_alt));
                ui.label(RichText::new("LINK ESTABLISHED").size(30.0).strong().color(palette.text));
                ui.label(RichText::new("Your system is online. Choose a module or jack into the net.").size(12.0).color(palette.muted));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Frame::new().fill(palette.surface_raised).stroke(Stroke::new(1.0, palette.accent)).corner_radius(3).inner_margin(Margin::same(12)).show(ui, |ui| {
                    ui.label(RichText::new("SYSTEM STATUS").size(9.0).monospace().color(palette.muted));
                    ui.label(RichText::new("● ONLINE").size(15.0).strong().monospace().color(palette.good));
                });
            });
        });
        ui.add_space(18.0);
        Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.accent_alt)).corner_radius(3).inner_margin(Margin::same(14)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("NET ACCESS").size(10.0).strong().monospace().color(palette.accent_alt));
                ui.label(RichText::new("// QUICK CONNECT").size(9.0).monospace().color(palette.muted));
            });
            ui.add_space(8.0);
            let response = ui.add_sized([ui.available_width() - 80.0, 40.0], TextEdit::singleline(&mut self.address).hint_text("Enter destination or search the net...").margin(Vec2::new(12.0, 10.0)));
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(&self.address.clone(), ctx.clone(), true); }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("CONNECT  ↗").strong().monospace().color(palette.text)).fill(palette.accent).corner_radius(3)).clicked() { self.navigate(&self.address.clone(), ctx.clone(), true); }
            });
        });
        ui.add_space(17.0);
        ui.label(RichText::new("SYSTEM MODULES").size(10.0).strong().monospace().color(palette.muted));
        ui.add_space(8.0);
        ui.columns(2, |columns| {
            tactical_card(&mut columns[0], palette, "01  //  FAVORITES", "Pinned destinations", "Your frequently accessed network locations.", "OPEN FAVORITES");
            tactical_card(&mut columns[1], palette, "02  //  ACTIVITY LOG", "Recent activity", "Browse the destinations in your session history.", "VIEW ACTIVITY");
        });
        ui.add_space(10.0);
        ui.columns(2, |columns| {
            Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3).inner_margin(Margin::same(13)).show(&mut columns[0], |ui| {
                ui.label(RichText::new("03  //  ACTIVE SESSION").size(9.0).monospace().color(palette.accent));
                ui.label(RichText::new(format!("{} OPEN TAB(S)", self.tabs.iter().filter(|tab| tab.workspace == self.active_workspace).count())).size(18.0).strong().color(palette.text));
                ui.label(RichText::new(format!("WORKSPACE: {}", self.active_workspace.to_ascii_uppercase())).size(10.0).monospace().color(palette.muted));
            });
            Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3).inner_margin(Margin::same(13)).show(&mut columns[1], |ui| {
                ui.label(RichText::new("04  //  ENGINE STATUS").size(9.0).monospace().color(palette.accent_alt));
                ui.label(RichText::new("LUCID ENGINE 0.0.1").size(18.0).strong().color(palette.text));
                ui.label(RichText::new("HTML/CSS EXPERIMENTAL RENDERER").size(10.0).monospace().color(palette.muted));
            });
        });
        ui.add_space(15.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(RichText::new("RUN ENGINE DEMO").monospace().color(palette.text)).fill(palette.surface_raised).stroke(Stroke::new(1.0, palette.accent)).corner_radius(3)).clicked() { self.navigate("lucid://demo", ctx.clone(), true); }
            if ui.add(egui::Button::new(RichText::new("OPEN LIBRARY").monospace().color(palette.text)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3)).clicked() { self.open_panel("library"); }
            if ui.add(egui::Button::new(RichText::new("SYSTEM CONFIG").monospace().color(palette.text)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3)).clicked() { self.open_panel("settings"); }
        });
    }

    fn draw_page(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if let Some(error) = self.active().error.clone() {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(RichText::new("Unable to load this page").size(24.0).strong().color(Color32::from_rgb(40, 36, 46)));
                ui.add_space(8.0);
                ui.label(RichText::new(error).color(Color32::from_rgb(105, 100, 112)));
                ui.label(RichText::new("Check the address and your internet connection.").size(12.0).color(Color32::from_rgb(105, 100, 112)));
            });
            return;
        }
        if self.active().loading {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.spinner();
                ui.label(RichText::new("Loading with Lucid Engine…").color(Color32::from_rgb(105, 100, 112)));
            });
            return;
        }
        let ctx = ui.ctx().clone();
        let Some(page) = self.active().page.clone() else {
            if self.active().url != "lucid://home" {
                ui.vertical_centered(|ui| {
                    ui.add_space(70.0);
                    ui.label(RichText::new("This tab is sleeping").size(23.0).strong().color(palette.text));
                    ui.label(RichText::new("Lucid released this page's document memory. Restore it when needed.").color(palette.muted));
                    if ui.button("Restore tab").clicked() {
                        let url = self.active().url.clone();
                        self.navigate(&url, ctx.clone(), false);
                    }
                });
            } else {
                self.draw_home(ui, palette, ctx);
            }
            return;
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(&page.title).strong().color(Color32::from_rgb(40, 36, 46)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(&page.url).size(10.0).color(Color32::from_rgb(105, 100, 112)));
            });
        });
        ui.separator();
        let width = ui.available_width().max(320.0);
        let layout = layout_document(&page.document, &page.stylesheet, width);
        let mut followed_link: Option<String> = None;
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(Vec2::new(width, layout.height.max(280.0)), Sense::hover());
            for (operation_index, operation) in layout.operations.iter().enumerate() {
                match operation {
                    PaintOp::Rect { x, y, width, height, color, radius } => {
                        let rect = egui::Rect::from_min_size(response.rect.min + Vec2::new(*x, *y), Vec2::new(*width, *height));
                        painter.rect_filled(rect, CornerRadius::same((*radius).clamp(0.0, 255.0) as u8), Color32::from_rgb(color.0, color.1, color.2));
                    }
                    PaintOp::Image { x, y, width: image_width, height: image_height, src, alt } => {
                        let rect = egui::Rect::from_min_size(
                            response.rect.min + Vec2::new(*x, *y),
                            Vec2::new(*image_width, *image_height),
                        );
                        let texture_key = resolve_link(&page.url, src).unwrap_or_else(|| src.clone());
                        if let Some(texture) = self.image_textures.get(&(self.active().id, texture_key)) {
                            painter.image(
                                texture.id(),
                                rect,
                                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            painter.rect_filled(rect, CornerRadius::same(5), Color32::from_rgb(244, 243, 247));
                            painter.rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.0, Color32::from_rgb(220, 216, 227)), egui::StrokeKind::Inside);
                            painter.text(rect.center(), Align2::CENTER_CENTER, alt, FontId::proportional(13.0), Color32::from_rgb(100, 96, 108));
                        }
                    }
                    PaintOp::Text { x, y, text, size, color, bold, italic, href } => {
                        let pos = response.rect.min + Vec2::new(*x, *y);
                        let font = FontId::proportional(*size);
                        let text_color = Color32::from_rgb(color.0, color.1, color.2);
                        let _emphasis = (*bold, *italic);
                        let is_find_match = !self.find_query.trim().is_empty() && text.to_lowercase().contains(&self.find_query.to_lowercase());
                        if is_find_match {
                            let highlight = egui::Rect::from_min_size(pos, Vec2::new((width - *x).max(80.0), (*size * 1.55).max(18.0)));
                            painter.rect_filled(highlight, CornerRadius::same(3), palette.accent.gamma_multiply(0.22));
                        }
                        let text_rect = painter.text(pos, Align2::LEFT_TOP, text, font, text_color);
                        if let Some(href) = href {
                            let link_response = ui.interact(
                                text_rect,
                                ui.id().with(("page-link", operation_index)),
                                Sense::click(),
                            );
                            if link_response.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if link_response.clicked() {
                                followed_link = resolve_link(&page.url, href);
                            }
                        }
                    }
                }
            }
        });
        if let Some(target) = followed_link {
            self.navigate(&target, ui.ctx().clone(), true);
        }
    }

    fn draw_library(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Your Library").size(24.0).strong().color(palette.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Bookmark current page").clicked() { self.toggle_current_bookmark(); }
            });
        });
        ui.label(RichText::new("Saved places, kept in your Lucid profile.").color(palette.muted));
        ui.add_space(14.0);
        let mut open_url = None;
        let mut remove_index = None;
        for (index, bookmark) in self.profile.bookmarks.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.button(RichText::new(&bookmark.title).strong().color(palette.text)).clicked() { open_url = Some(bookmark.url.clone()); }
                ui.label(RichText::new(&bookmark.url).size(10.0).color(palette.muted));
                if ui.small_button("Remove").clicked() { remove_index = Some(index); }
            });
            ui.separator();
        }
        if self.profile.bookmarks.is_empty() {
            ui.label(RichText::new("No bookmarks yet. Use the star beside the address bar to save a page.").color(palette.muted));
        }
        if let Some(index) = remove_index { self.profile.bookmarks.remove(index); }
        if let Some(url) = open_url { self.navigate(&url, ctx, true); }
    }

    fn draw_history(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.label(RichText::new("Browsing History").size(24.0).strong().color(palette.text));
        ui.label(RichText::new("Your recent visits, stored locally.").color(palette.muted));
        ui.add_space(12.0);
        let mut open_url = None;
        for entry in self.profile.history.iter().rev().take(100) {
            ui.horizontal(|ui| {
                if ui.button(RichText::new(&entry.title).color(palette.text)).clicked() { open_url = Some(entry.url.clone()); }
                ui.label(RichText::new(&entry.url).size(10.0).color(palette.muted));
            });
            ui.separator();
        }
        if self.profile.history.is_empty() {
            ui.label(RichText::new("Your visited pages will appear here.").color(palette.muted));
        }
        if let Some(url) = open_url { self.navigate(&url, ctx, true); }
    }

    fn draw_control_center(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let snapshot = self.performance.snapshot();
        ui.label(RichText::new("Lucid Control Center").size(24.0).strong().color(palette.text));
        ui.label(RichText::new("Live measurements for the Lucid desktop process.").color(palette.muted));
        ui.add_space(14.0);
        ui.columns(3, |columns| {
            stat_card(&mut columns[0], palette, "◉", "Lucid CPU", &format!("{:.1}%", snapshot.browser_cpu_percent), "Main process");
            stat_card(&mut columns[1], palette, "▣", "Lucid memory", &format_bytes(snapshot.browser_memory_bytes), "Resident memory");
            stat_card(&mut columns[2], palette, "▤", "System RAM", &format!("{} / {}", format_bytes(snapshot.system_memory_used_bytes), format_bytes(snapshot.system_memory_total_bytes)), "Used / total");
        });
        ui.add_space(16.0);
        ui.label(RichText::new("MEMORY SAVER").strong().color(palette.text));
        ui.checkbox(&mut self.profile.memory_saver, "Enable manual background-tab sleeping");
        if ui.button("Sleep inactive tabs now").clicked() { self.sleep_background_tabs(); }
        ui.label(RichText::new(format!("{} open tabs · pinned tabs are kept awake", self.tabs.len())).size(11.0).color(palette.muted));
        ui.add_space(14.0);
        ui.label(RichText::new("NETWORK GOVERNOR").strong().color(palette.text));
        ui.add(egui::Slider::new(&mut self.profile.navigation_limit_kib, 0..=4096).text("Per-navigation HTML cap (KiB/s)"));
        ui.label(RichText::new(if self.profile.navigation_limit_kib == 0 { "Unlimited" } else { "Applied to top-level HTML transfers; resource-level governance is still being developed." }).size(11.0).color(palette.muted));
        ui.add_space(14.0);
        ui.label(RichText::new("LUCID PROTECT").strong().color(palette.text));
        ui.checkbox(&mut self.profile.navigation_protection, "Block known advertising and tracking hosts on top-level navigation");
        ui.label(RichText::new("This first-party list only filters page URLs. Full ad blocking requires resource loading, filter-list parsing and per-origin request interception.").size(11.0).color(palette.muted));
        ui.add_space(14.0);
        ui.label(RichText::new("ENGINE ISOLATION").strong().color(palette.text));
        ui.label(RichText::new("Per-site CPU/RAM limits require renderer processes. Lucid currently renders in one process, so no fake per-tab limits are shown.").size(11.0).color(palette.muted));
    }

    fn draw_mods(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: &egui::Context) {
        ui.label(RichText::new("Lucid Mods").size(24.0).strong().color(palette.text));
        ui.label(RichText::new("Customize the atmosphere without changing the engine or your open tabs.").color(palette.muted));
        ui.add_space(14.0);
        ui.label(RichText::new("INTERFACE MODE").strong().color(palette.text));
        ui.horizontal(|ui| {
            if ui.selectable_label(self.theme == Theme::Calm, "Lucid Calm").clicked() { self.theme = Theme::Calm; }
            if ui.selectable_label(self.theme == Theme::Cyberpunk, "Lucid Cyberpunk").clicked() { self.theme = Theme::Cyberpunk; }
        });
        ui.add_space(12.0);
        ui.label(RichText::new("ACCENT PACK").strong().color(palette.text));
        ui.horizontal_wrapped(|ui| {
            for (name, value, swatch) in [
                ("Lavender", "lavender", Color32::from_rgb(183, 153, 236)),
                ("Violet", "violet", Color32::from_rgb(151, 105, 245)),
                ("Cyan", "cyan", Color32::from_rgb(73, 208, 216)),
                ("Magenta", "magenta", Color32::from_rgb(239, 91, 184)),
                ("Mint", "mint", Color32::from_rgb(112, 218, 172)),
                ("Amber", "amber", Color32::from_rgb(241, 183, 93)),
            ] {
                let button = egui::Button::new(RichText::new(name).color(palette.text))
                    .fill(swatch.gamma_multiply(0.22))
                    .stroke(Stroke::new(if self.profile.accent == value { 2.0 } else { 1.0 }, swatch))
                    .corner_radius(10);
                if ui.add(button).clicked() { self.profile.accent = value.into(); }
            }
        });
        ui.add_space(12.0);
        ui.checkbox(&mut self.profile.animations_enabled, "Enable interface motion");
        ctx.style_mut_of(egui::Theme::Dark, |style| { style.animation_time = if self.profile.animations_enabled { 0.15 } else { 0.0 }; });
        ui.add_space(12.0);
        ui.label(RichText::new("MOD SUPPORT STATUS").strong().color(palette.text));
        ui.label(RichText::new("Theme presets and accent packs are active. User-installed shaders, sounds, icon packs and third-party mods need a permissioned mod runtime and are staged for later.").size(11.0).color(palette.muted));
    }

    fn draw_gx_corner(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.label(RichText::new("Lucid Corner").size(24.0).strong().color(palette.text));
        ui.label(RichText::new("A gaming launchpad. These shortcuts open source sites; live game news and release feeds are not connected yet.").color(palette.muted));
        ui.add_space(14.0);
        let items = [
            ("PC game deals", "https://store.steampowered.com/specials"),
            ("Steam news", "https://store.steampowered.com/news"),
            ("Epic Games Store", "https://store.epicgames.com"),
            ("PC gaming news", "https://www.pcgamer.com"),
            ("Release calendar", "https://www.igdb.com"),
            ("Twitch", "https://www.twitch.tv"),
        ];
        let mut open_url = None;
        ui.columns(2, |columns| {
            for (index, (label, url)) in items.iter().enumerate() {
                if columns[index % 2].button(RichText::new(*label).color(palette.text)).clicked() {
                    open_url = Some((*url).to_owned());
                }
            }
        });
        if let Some(url) = open_url { self.navigate(&url, ctx, true); }
    }

    fn draw_downloads(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.label(RichText::new("Downloads").size(24.0).strong().color(palette.text));
        ui.label(RichText::new("Download a direct HTTP/HTTPS file into Downloads/Lucid. Maximum file size: 512 MiB.").color(palette.muted));
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_sized([ui.available_width() - 110.0, 34.0], TextEdit::singleline(&mut self.download_input).hint_text("https://example.com/file.zip"));
            if ui.button("Download").clicked() {
                let url = self.download_input.trim().to_owned();
                if !url.is_empty() {
                    let id = self.next_download_id;
                    self.next_download_id += 1;
                    self.downloads.push(DownloadItem { id, url: url.clone(), received: 0, total: None, status: "Queued".into(), saved_path: None });
                    let sender = self.download_tx.clone();
                    thread::spawn(move || run_download(id, url, sender));
                    self.download_input.clear();
                }
            }
        });
        ui.add_space(12.0);
        for item in &self.downloads {
            ui.group(|ui| {
                ui.label(RichText::new(&item.url).strong().color(palette.text));
                if let Some(total) = item.total.filter(|total| *total > 0) {
                    ui.add(egui::ProgressBar::new((item.received as f32 / total as f32).clamp(0.0, 1.0))
                        .show_percentage().text(format!("{} / {}", format_bytes(item.received), format_bytes(total))));
                } else {
                    ui.label(RichText::new(format_bytes(item.received)).size(11.0).color(palette.muted));
                }
                ui.label(RichText::new(&item.status).size(11.0).color(palette.muted));
                if let Some(path) = &item.saved_path {
                    ui.label(RichText::new(path).size(10.0).color(palette.good));
                }
            });
        }
        if self.downloads.is_empty() {
            ui.label(RichText::new("Your downloads will appear here.").color(palette.muted));
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.label(RichText::new("Settings").size(25.0).strong().color(palette.text));
        ui.label(RichText::new("Lucid's interface, privacy and startup preferences").color(palette.muted));
        ui.add_space(18.0);
        ui.label(RichText::new("APPEARANCE").size(10.0).strong().color(palette.muted));
        ui.horizontal(|ui| {
            if ui.selectable_label(self.theme == Theme::Calm, "Lucid Calm").clicked() { self.theme = Theme::Calm; }
            if ui.selectable_label(self.theme == Theme::Cyberpunk, "Lucid Cyberpunk").clicked() { self.theme = Theme::Cyberpunk; }
        });
        ui.checkbox(&mut self.bookmarks_bar, "Show bookmarks bar");
        ui.checkbox(&mut self.profile.memory_saver, "Enable manual background-tab sleeping");
        ui.checkbox(&mut self.profile.navigation_protection, "Enable Lucid Protect navigation filter");
        ui.checkbox(&mut self.profile.animations_enabled, "Enable interface animations");
        ui.add_space(16.0);
        ui.label(RichText::new("ENGINE STATUS").size(10.0).strong().color(palette.muted));
        ui.label(RichText::new("Lucid Engine · Native renderer prototype").color(palette.text));
        ui.label(RichText::new("Lucid-owned HTML tree, CSS subset, native layout, image quads and page painting").size(11.0).color(palette.muted));
        ui.label(RichText::new("Still under construction: JavaScript, full HTML/CSS, fonts, media, storage APIs, sandboxing and extensions.").size(11.0).color(palette.muted));
        ui.add_space(12.0);
        if ui.button("Save settings now").clicked() { self.persist_profile(); }
    }
}

impl eframe::App for LucidApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_loads(ui.ctx().clone());
        self.poll_downloads();
        if !self.session_restore_started {
            self.session_restore_started = true;
            if self.active().url != "lucid://home" {
                let url = self.active().url.clone();
                self.navigate(&url, ui.ctx().clone(), false);
            }
        }
        self.persist_if_due();
        let mut palette = self.theme.palette();
        palette.accent = match self.profile.accent.as_str() {
            "violet" => Color32::from_rgb(151, 105, 245),
            "cyan" => Color32::from_rgb(73, 208, 216),
            "magenta" => Color32::from_rgb(239, 91, 184),
            "mint" => Color32::from_rgb(112, 218, 172),
            "amber" => Color32::from_rgb(241, 183, 93),
            _ => if self.theme == Theme::Calm { Color32::from_rgb(183, 153, 236) } else { Color32::from_rgb(255, 65, 119) },
        };
        let ctx = ui.ctx().clone();
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = palette.background;
        visuals.window_fill = palette.surface;
        visuals.override_text_color = Some(palette.text);
        visuals.widgets.noninteractive.bg_fill = palette.surface;
        visuals.widgets.noninteractive.fg_stroke.color = palette.text;
        visuals.widgets.inactive.bg_fill = palette.surface_raised;
        visuals.widgets.inactive.fg_stroke.color = palette.text;
        visuals.widgets.hovered.bg_fill = palette.border;
        visuals.widgets.hovered.fg_stroke.color = palette.text;
        visuals.widgets.active.bg_fill = palette.accent_alt;
        visuals.widgets.active.fg_stroke.color = palette.text;
        ctx.set_visuals(visuals);
        ctx.style_mut_of(egui::Theme::Dark, |style| { style.animation_time = if self.profile.animations_enabled { 0.15 } else { 0.0 }; });

        ui.set_min_size(ui.available_size());
        ui.vertical(|ui| {
            ui.add_space(9.0);
            self.draw_header(ui, palette);
            self.draw_tabs(ui, palette);
            ui.add_space(7.0);
            self.draw_toolbar(ui, palette, ctx.clone());
            if self.bookmarks_bar {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("BOOKMARKS").size(9.0).color(palette.muted));
                    for (label, url) in [("GitHub", "https://github.com"), ("Wikipedia", "https://wikipedia.org"), ("Rust", "https://rust-lang.org")] {
                        if ui.small_button(label).clicked() { self.navigate(&url, ctx.clone(), true); }
                    }
                });
            }
            if self.show_find {
                ui.horizontal(|ui| {
                    let response = ui.add_sized([280.0, 30.0],
                        TextEdit::singleline(&mut self.find_query).hint_text("Find in page…"));
                    if self.find_focus { response.request_focus(); self.find_focus = false; }
                    if response.changed() { self.find_in_page(); }
                    if ui.small_button("×").clicked() {
                        self.show_find = false;
                        self.find_query.clear();
                    }
                    ui.label(RichText::new(&self.status).size(10.0).color(palette.muted));
                });
                ui.add_space(6.0);
            }
            let panels_open = self.show_settings || self.show_gx_control || self.show_history
                || self.show_mods || self.show_corner || self.show_downloads || self.show_library;
            if !panels_open && is_web_url(&self.active().url) {
                let rect = ui.available_rect_before_wrap();
                ui.allocate_rect(rect, Sense::hover());
                self.cef_view_rect = Some(rect);
            } else {
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    if self.show_settings { self.draw_settings(ui, palette); }
                    else if self.show_gx_control { self.draw_control_center(ui, palette); }
                    else if self.show_history { self.draw_history(ui, palette, ctx.clone()); }
                    else if self.show_mods { self.draw_mods(ui, palette, &ctx); }
                    else if self.show_corner { self.draw_gx_corner(ui, palette, ctx.clone()); }
                    else if self.show_downloads { self.draw_downloads(ui, palette); }
                    else if self.show_library { self.draw_library(ui, palette, ctx.clone()); }
                    else if self.active().page.is_some() || self.active().error.is_some() || self.active().loading {
                        Frame::new().fill(Color32::WHITE).inner_margin(Margin::same(18)).corner_radius(10)
                            .show(ui, |ui| self.draw_page(ui, palette));
                    } else {
                        self.draw_home(ui, palette, ctx.clone());
                    }
                });
            }
        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) { self.show_command_palette = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) && (self.active().page.is_some() || is_web_url(&self.active().url)) { self.show_find = true; self.find_focus = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.show_find { self.show_find = false; self.find_query.clear(); }
        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) { self.new_tab(); }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) { self.close_tab(self.active_tab); }
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) { self.open_panel("control"); }
        if self.show_command_palette {
            egui::Window::new("Lucid Command Palette").collapsible(false).resizable(false).anchor(Align2::CENTER_TOP, [0.0, 90.0]).show(&ctx, |ui| {
                ui.set_width(390.0);
                ui.add(TextEdit::singleline(&mut self.command_query).hint_text("Search commands…").desired_width(f32::INFINITY));
                ui.separator();
                let query = self.command_query.to_ascii_lowercase();
                for command in ["New tab", "Toggle theme", "Open library", "History", "GX Control", "Mods", "GX Corner", "Downloads", "Settings", "Home", "Bookmark current page", "Sleep background tabs"] {
                    if !query.is_empty() && !command.to_ascii_lowercase().contains(&query) { continue; }
                    if ui.button(command).clicked() { self.execute_command(command, ctx.clone()); }
                }
            });
        }
        #[cfg(target_os = "windows")]
        self.sync_cef_views(ctx.pixels_per_point());
        #[cfg(target_os = "windows")]
        if self.tabs.iter().any(|tab| tab.cef_browser.is_some()) {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

fn is_web_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn count_text_matches(nodes: &[Node], query: &str) -> usize {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() { return 0; }
    nodes.iter().map(|node| match node {
        Node::Text(text) => text.to_lowercase().matches(&needle).count(),
        Node::Element(element) => count_text_matches(&element.children, &needle),
    }).sum()
}

fn stat_card(ui: &mut egui::Ui, palette: Palette, icon: &str, label: &str, value: &str, detail: &str) {
    Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(13).inner_margin(Margin::same(12)).show(ui, |ui| {
        ui.label(RichText::new(icon).size(18.0).color(palette.accent));
        ui.label(RichText::new(label).size(10.0).color(palette.muted));
        ui.label(RichText::new(value).size(18.0).strong().color(palette.text));
        ui.label(RichText::new(detail).size(9.0).color(palette.muted));
    });
}

fn tactical_card(ui: &mut egui::Ui, palette: Palette, code: &str, title: &str, detail: &str, action: &str) {
    Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3).inner_margin(Margin::same(13)).show(ui, |ui| {
        ui.label(RichText::new(code).size(9.0).monospace().color(palette.accent_alt));
        ui.add_space(4.0);
        ui.label(RichText::new(title).size(17.0).strong().color(palette.text));
        ui.label(RichText::new(detail).size(11.0).color(palette.muted));
        ui.add_space(5.0);
        ui.label(RichText::new(action).size(9.0).monospace().color(palette.accent));
    });
}

fn normalize_address(input: &str) -> String {
    let value = input.trim();
    if value.is_empty() || value == "lucid://home" { return "lucid://home".into(); }
    if value == "lucid://demo" { return value.into(); }
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") { return value.to_owned(); }
    if value.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty() && scheme.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
    }) && !value.chars().any(char::is_whitespace) {
        return encode_search_query(value);
    }
    if !value.chars().any(char::is_whitespace) && value.contains('.') {
        return format!("https://{value}");
    }
    encode_search_query(value)
}

fn encode_search_query(value: &str) -> String {
    let encoded = value.bytes().map(|b| {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' { (b as char).to_string() }
        else { format!("%{b:02X}") }
    }).collect::<Vec<_>>().join("");
    format!("https://duckduckgo.com/?q={encoded}")
}

fn resolve_link(base: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('#') { return None; }
    let base = reqwest::Url::parse(base).ok()?;
    let resolved = base.join(href).ok()?;
    match resolved.scheme() {
        "http" | "https" => Some(resolved.to_string()),
        _ => None,
    }
}

const MAX_HTML_DOCUMENT_BYTES: u64 = 5 * 1024 * 1024;

struct RateLimitedReader<R> {
    inner: R,
    bytes_per_second: u64,
    started: Instant,
    bytes_read: u64,
}

impl<R: Read> RateLimitedReader<R> {
    fn new(inner: R, kib_per_second: u32) -> Self {
        Self {
            inner,
            bytes_per_second: u64::from(kib_per_second).saturating_mul(1024),
            started: Instant::now(),
            bytes_read: 0,
        }
    }
}

impl<R: Read> Read for RateLimitedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.inner.read(buffer)?;
        if count == 0 || self.bytes_per_second == 0 { return Ok(count); }
        self.bytes_read = self.bytes_read.saturating_add(count as u64);
        let target = Duration::from_secs_f64(self.bytes_read as f64 / self.bytes_per_second as f64);
        if let Some(delay) = target.checked_sub(self.started.elapsed()) { thread::sleep(delay); }
        Ok(count)
    }
}

const MAX_IMAGE_RESOURCE_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 16_000_000;

/// Load a page through Lucid's own network, HTML, CSS, image and paint pipeline.
/// Script elements remain inert until the JavaScript runtime is implemented.
fn load_page(url: &str, protection_enabled: bool, network_limit_kib: u32) -> Result<LoadedPage, String> {
    let html = fetch_document(url, protection_enabled, network_limit_kib)?;
    let document = parse_html(&html);
    let inline_styles = collect_styles(&document);
    let mut stylesheet = Stylesheet::default();
    let mut images = BTreeMap::new();

    if is_web_url(url) {
        let base = reqwest::Url::parse(url).map_err(|error| format!("Invalid page URL: {error}"))?;
        let loader = ResourceLoader::new(MAX_IMAGE_RESOURCE_BYTES)
            .map_err(|error| format!("Could not initialize Lucid's resource loader: {error}"))?;

        // External sheets are loaded by Lucid and parsed by the native CSS pipeline.
        if let Ok(external_styles) = load_document_stylesheets(&document, &base, &loader) {
            stylesheet = external_styles;
        }
        // Inline author styles are appended after linked sheets in this first cascade pass.
        stylesheet.rules.extend(inline_styles.rules);

        // Decode HTML image resources and pass RGBA pixels to Lucid's native painter.
        for reference in discover_document_resources(&document, &base)
            .into_iter()
            .filter(|resource| resource.kind == ResourceKind::Image)
            .take(32)
        {
            let Ok(resource) = loader.fetch(&reference) else { continue };
            let Ok(decoded) = image::load_from_memory(&resource.bytes) else { continue };
            if u64::from(decoded.width()) * u64::from(decoded.height()) > MAX_IMAGE_PIXELS {
                continue;
            }
            let rgba = decoded.to_rgba8();
            images.insert(reference.url.to_string(), EngineImage {
                width: rgba.width() as usize,
                height: rgba.height() as usize,
                rgba: rgba.into_raw(),
            });
        }
    } else {
        stylesheet = inline_styles;
    }

    let title = document_title(&document).unwrap_or_else(|| {
        reqwest::Url::parse(url).ok()
            .and_then(|parsed| parsed.host_str().map(str::to_owned))
            .unwrap_or_else(|| "Lucid page".into())
    });

    Ok(LoadedPage {
        url: url.to_owned(),
        title,
        document,
        stylesheet,
        images,
    })
}

fn fetch_document(url: &str, protection_enabled: bool, network_limit_kib: u32) -> Result<String, String> {
    if protection_enabled && protection::is_blocked_navigation(url) {
        return Err("Blocked by Lucid Protect: destination matches the built-in navigation blocklist.".into());
    }
    if url == "lucid://demo" {
        let html = r##"<!doctype html><html><head><title>Lucid Engine Demo</title><style>
            body { color: #282431; }
            .hero { color: #7452b0; font-size: 34px; font-weight: 700; }
            .panel { background: #f0ebf8; padding: 18px; border-radius: 14px; margin: 12px; }
            .note { color: #655d71; font-size: 14px; }
            a { color: #7650bd; }
        </style></head><body>
            <h1 class="hero">Welcome to Lucid Engine</h1>
            <p>This page is being parsed and laid out by Lucid's own experimental HTML and CSS code.</p>
            <div class="panel"><h2>What is working?</h2><p>HTML tokenization, a small DOM tree, basic CSS selectors, simple declarations, and a top-to-bottom layout prototype.</p></div>
            <h2>What comes next?</h2><p class="note">Inline layout, flexbox, grid, proper HTML5 parsing, JavaScript, standards tests, security isolation, and many more web APIs.</p>
            <p><a href="https://www.rust-lang.org">Learn about Rust →</a></p>
        </body></html>"##;
        return Ok(html.into());
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Lucid currently supports HTTP and HTTPS web addresses only. Local-file access is not enabled in this prototype.".into());
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent("LucidBrowser/0.0.2 (experimental independent engine)")
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|e| format!("Could not initialize network client: {e}"))?;
    let response = client.get(url).send().map_err(|e| format!("Network request failed: {e}"))?;
    if !response.status().is_success() { return Err(format!("Server returned {}", response.status())); }
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok()).unwrap_or("");
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml+xml") {
        return Err(format!("This prototype currently displays HTML documents only (received {content_type})."));
    }
    if response.content_length().is_some_and(|length| length > MAX_HTML_DOCUMENT_BYTES) {
        return Err("The HTML document is larger than Lucid's current 5 MiB prototype limit.".into());
    }
    let mut body = String::new();
    let mut limited = RateLimitedReader::new(response.take(MAX_HTML_DOCUMENT_BYTES + 1), network_limit_kib);
    let bytes_read = limited.read_to_string(&mut body)
        .map_err(|e| format!("Could not read HTML response: {e}"))?;
    if bytes_read as u64 > MAX_HTML_DOCUMENT_BYTES {
        return Err("The HTML document exceeded Lucid's current 5 MiB prototype limit.".into());
    }
    Ok(body)
}

pub fn run_lucid(main_args: &cef::MainArgs, sandbox_info: *mut u8) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucid Browser")
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([860.0, 620.0]),
        ..Default::default()
    };
    let result = eframe::run_native("Lucid Browser", options, Box::new(move |cc| {
        LucidApp::new(cc, main_args, sandbox_info)
            .map(|app| Box::new(app) as Box<dyn eframe::App>)
            .map_err(|message| Box::new(std::io::Error::other(message)) as Box<dyn std::error::Error + Send + Sync>)
    }));
    cef::shutdown();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_normalization_handles_sites_and_searches() {
        assert_eq!(normalize_address("example.com"), "https://example.com");
        assert_eq!(normalize_address("hello world"), "https://duckduckgo.com/?q=hello%20world");
        assert_eq!(normalize_address("lucid://demo"), "lucid://demo");
        assert!(normalize_address("file:///C:/Users/test/private.html").starts_with("https://duckduckgo.com/?q="));
        assert!(normalize_address("javascript:alert(1)").starts_with("https://duckduckgo.com/?q="));
    }

    #[test]
    fn links_resolve_relative_to_the_current_document() {
        assert_eq!(resolve_link("https://example.com/docs/page.html", "../guide"), Some("https://example.com/guide".into()));
        assert_eq!(resolve_link("https://example.com", "javascript:alert(1)"), None);
        assert_eq!(resolve_link("https://example.com", "file:///C:/secret.txt"), None);
    }

    #[test]
    fn navigation_protection_blocks_known_ad_hosts() {
        assert!(protection::is_blocked_navigation("https://doubleclick.net/ads"));
        assert!(!protection::is_blocked_navigation("https://example.com"));
    }

    #[test]
    fn demo_page_parses_through_lucid_engine() {
        let html = fetch_document("lucid://demo", false, 0).expect("demo should be local");
        let document = parse_html(&html);
        assert_eq!(document_title(&document).as_deref(), Some("Lucid Engine Demo"));
        let styles = collect_styles(&document);
        assert!(!styles.rules.is_empty());
        let layout = layout_document(&document, &styles, 900.0);
        assert!(!layout.operations.is_empty());
    }
}
