mod downloads;
mod performance;
mod persistence;
mod protection;

use eframe::egui::{self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit, Vec2};
use downloads::{DownloadUpdate, run_download};
use performance::{PerformanceMonitor, format_bytes};
use persistence::{BrowserProfile, SavedTab};
use std::time::Instant;
use lucid_engine::{Document, PaintOp, Stylesheet, collect_styles, document_title, layout_document, parse_html};
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
struct LoadedPage {
    url: String,
    title: String,
    document: Document,
    stylesheet: Stylesheet,
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
}

impl Tab {
    fn new(id: u64, workspace: String) -> Self {
        Self { id, workspace, title: "New tab".into(), url: "lucid://home".into(), page: None, error: None, history: vec!["lucid://home".into()], history_index: 0, request_generation: 0, loading: false, pinned: false, discarded: false }
    }
}

enum LoadResult {
    Success { tab_id: u64, request_generation: u64, url: String, html: String },
    Failure { tab_id: u64, request_generation: u64, url: String, message: String },
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
    tx: Sender<LoadResult>,
    rx: Receiver<LoadResult>,
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
            status: "Your browser, in its natural state.".into(),
            show_library: false, show_settings: false, show_history: false,
            show_gx_control: false, show_mods: false, show_corner: false, show_downloads: false,
            show_command_palette: false, command_query: String::new(),
            active_workspace, workspace_name_input: String::new(),
            bookmarks_bar: profile.bookmarks_bar, profile,
            performance: PerformanceMonitor::new(),
            session_restore_started: false, last_persist: Instant::now() - Duration::from_secs(5),
            download_input: String::new(), download_tx, download_rx, downloads: Vec::new(), next_download_id: 1,
        }
    }
}

impl LucidApp {
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
        for tab in &mut self.tabs {
            if tab.id != active_id && !tab.pinned && tab.page.is_some() {
                tab.page = None;
                tab.discarded = true;
                slept += 1;
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

    fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() { self.tabs.remove(index); }
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
        let request_generation = self.next_request_generation;
        self.next_request_generation = self.next_request_generation.wrapping_add(1).max(1);
        let tab_id = self.active().id;
        {
            let tab = self.active_mut();
            tab.request_generation = request_generation;
            tab.loading = url != "lucid://home";
            tab.url = url.clone();
            tab.title = if url == "lucid://home" { "New tab".into() } else { "Loading…".into() };
            tab.page = None;
            tab.error = None;
            if push_history {
                if tab.history_index + 1 < tab.history.len() {
                    tab.history.truncate(tab.history_index + 1);
                }
                if tab.history.last() != Some(&url) {
                    tab.history.push(url.clone());
                }
                tab.history_index = tab.history.len().saturating_sub(1);
            }
        }
        self.address = url.clone();
        self.close_panels();
        if url == "lucid://home" {
            self.status = "Ready".into();
            return;
        }
        self.status = format!("Connecting to {url}");
        let tx = self.tx.clone();
        let protection_enabled = self.profile.navigation_protection;
        let network_limit_kib = self.profile.navigation_limit_kib;
        thread::spawn(move || {
            let fetched = fetch_document(&url, protection_enabled, network_limit_kib);
            let event = match fetched {
                Ok(html) => LoadResult::Success { tab_id, request_generation, url, html },
                Err(message) => LoadResult::Failure { tab_id, request_generation, url, message },
            };
            let _ = tx.send(event);
            ctx.request_repaint_after(Duration::from_millis(20));
        });
    }

    fn poll_loads(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                LoadResult::Success { tab_id, request_generation, url, html } => {
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if tab.request_generation != request_generation { continue; }
                    let document = parse_html(&html);
                    let stylesheet = collect_styles(&document);
                    let title = document_title(&document).unwrap_or_else(|| url.clone());
                    tab.title = title.clone();
                    tab.url = url.clone();
                    tab.page = Some(LoadedPage { url, title, document, stylesheet });
                    tab.error = None;
                    tab.loading = false;
                    tab.discarded = false;
                    self.profile.add_history(title.clone(), url.clone());
                    if self.tabs.get(self.active_tab).is_some_and(|active| active.id == tab_id) {
                        self.status = "Loaded with Lucid Engine · JavaScript not yet supported".into();
                    }
                }
                LoadResult::Failure { tab_id, request_generation, url, message } => {
                    let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) else { continue };
                    if tab.request_generation != request_generation { continue; }
                    tab.title = "Page unavailable".into();
                    tab.url = url;
                    tab.error = Some(message);
                    tab.page = None;
                    tab.loading = false;
                    if self.tabs.get(self.active_tab).is_some_and(|active| active.id == tab_id) {
                        self.status = "Navigation failed".into();
                    }
                }
            }
        }
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
            ui.label(RichText::new("L").size(23.0).strong().color(palette.accent));
            ui.label(RichText::new("lucid").size(21.0).strong().color(palette.text));
            ui.label(RichText::new("ENGINE 0.0.3").size(9.0).color(palette.muted).monospace());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let theme_label = if self.theme == Theme::Calm { "Cyberpunk" } else { "Calm" };
                if ui.add(egui::Button::new(RichText::new(theme_label).color(palette.text)).fill(palette.surface_raised).stroke(Stroke::new(1.0, palette.border)).corner_radius(10)).clicked() {
                    self.theme = if self.theme == Theme::Calm { Theme::Cyberpunk } else { Theme::Calm };
                }
                if ui.add(egui::Button::new(RichText::new("⚙").color(palette.muted)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border))).clicked() { self.show_settings = !self.show_settings; }
                if ui.add(egui::Button::new(RichText::new("⌘K").color(palette.muted).size(11.0)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border))).clicked() { self.show_command_palette = true; }
            });
        });
        ui.add_space(7.0);
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            let mut close_index = None;
            let mut set_pin: Option<(usize, bool)> = None;
            let mut restore_url = None;
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
                    if self.tabs[index].page.is_none() && self.tabs[index].url != "lucid://home" {
                        restore_url = Some(self.tabs[index].url.clone());
                    }
                }
                response.context_menu(|ui| {
                    if ui.button(if tab.pinned { "Unpin tab" } else { "Pin tab" }).clicked() {
                        set_pin = Some((index, !tab.pinned));
                        ui.close_menu();
                    }
                    if ui.button("Close tab").clicked() {
                        close_index = Some(index);
                        ui.close_menu();
                    }
                });
                if ui.add(egui::Button::new(RichText::new("×").color(palette.muted).size(12.0)).fill(fill).corner_radius(9)).clicked() {
                    close_index = Some(index);
                }
            }
            if let Some((index, pinned)) = set_pin { self.tabs[index].pinned = pinned; }
            if let Some(index) = close_index { self.close_tab(index); }
            if let Some(url) = restore_url { self.navigate(&url, ui.ctx().clone(), false); }
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
            let address = ui.add_sized([ui.available_width() - 128.0, 34.0], TextEdit::singleline(&mut self.address).id(egui::Id::new("lucid-address-bar")).hint_text("Search or enter a website address…").desired_width(f32::INFINITY).margin(Vec2::new(12.0, 7.0)));
            if address.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(&self.address.clone(), ctx.clone(), true); }
            if ui.add(egui::Button::new(RichText::new("Go").color(palette.text)).fill(palette.accent_alt).corner_radius(9)).clicked() { self.navigate(&self.address.clone(), ctx.clone(), true); }
            let bookmarked = self.profile.is_bookmarked(&self.active().url);
            if ui.add(egui::Button::new(RichText::new(if bookmarked { "★" } else { "☆" }).color(palette.accent)).fill(palette.surface).corner_radius(9))
                .on_hover_text("Bookmark this page").clicked() {
                self.toggle_current_bookmark();
            }
            if ui.add(egui::Button::new("◇").fill(palette.surface).corner_radius(9)).on_hover_text("Open Library").clicked() {
                self.open_panel("library");
            }
        });
    }

    fn draw_home(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        if self.theme == Theme::Cyberpunk {
            self.draw_cyberpunk_home(ui, palette, ctx);
            return;
        }
        ui.add_space(34.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("L").size(62.0).strong().color(palette.accent));
            ui.label(RichText::new("lucid.").size(40.0).strong().color(palette.text));
            ui.label(RichText::new("A little more clarity.").size(14.0).color(palette.muted));
            ui.add_space(21.0);
            let search = ui.add_sized([ui.available_width().min(660.0), 48.0], TextEdit::singleline(&mut self.address).hint_text("Search anything or enter a URL").margin(Vec2::new(17.0, 13.0)));
            if search.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(&self.address.clone(), ctx.clone(), true); }
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("WORKSPACE").size(10.0).color(palette.muted).strong());
                for workspace in self.profile.workspaces.clone() {
                    let selected = self.active_workspace == workspace;
                    if ui.add(egui::Button::new(RichText::new(workspace).size(11.0).color(if selected { palette.text } else { palette.muted })).fill(if selected { palette.surface_raised } else { palette.background }).stroke(Stroke::new(if selected { 1.0 } else { 0.0 }, palette.border)).corner_radius(10)).clicked() {
                        self.switch_workspace(workspace);
                    }
                }
            });
            ui.add_space(5.0);
            ui.add_sized([140.0, 26.0], TextEdit::singleline(&mut self.workspace_name_input).hint_text("New workspace"));
            if ui.small_button("＋").clicked() {
                let name = self.workspace_name_input.trim().to_owned();
                if !name.is_empty() && !self.profile.workspaces.iter().any(|workspace| workspace.eq_ignore_ascii_case(&name)) {
                    self.profile.workspaces.push(name.clone());
                    self.switch_workspace(&name);
                    self.workspace_name_input.clear();
                }
            }
        });
        ui.add_space(34.0);
        ui.horizontal(|ui| { ui.label(RichText::new("QUICK ACCESS").size(10.0).strong().color(palette.muted)); ui.add_space(5.0); ui.label(RichText::new("Your favorite places").size(11.0).color(palette.muted)); });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            for (label, sub, url, glyph) in self.profile.quick_links.clone().into_iter().map(|link| {
                let glyph = link.label.chars().next().unwrap_or('•').to_string();
                (link.label, link.subtitle, link.url, glyph)
            }) {
                let card = Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(14).inner_margin(Margin::same(15));
                card.show(ui, |ui| {
                    ui.set_min_width(122.0);
                    ui.set_min_height(60.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(glyph).size(20.0).color(palette.accent).strong());
                        if ui.button(RichText::new(label).color(palette.text).strong()).clicked() { self.navigate(url, ctx.clone(), true); }
                        ui.label(RichText::new(sub).size(10.0).color(palette.muted));
                    });
                });
            }
        });
        ui.add_space(22.0);
        ui.horizontal(|ui| { ui.label(RichText::new("YOUR BROWSER").size(10.0).strong().color(palette.muted)); });
        ui.add_space(9.0);
        ui.columns(3, |columns| {
            let workspace_tabs = self.tabs.iter().filter(|tab| tab.workspace == self.active_workspace).count();
            stat_card(&mut columns[0], palette, "◫", "Open tabs", &workspace_tabs.to_string(), "In this workspace");
            stat_card(&mut columns[1], palette, "◈", "Engine", "0.0.3", "Independent prototype");
            stat_card(&mut columns[2], palette, "◇", "Protect", if self.profile.navigation_protection { "Enabled" } else { "Off" }, "Top-level URL filter");
        });
        ui.add_space(18.0);
        let info = Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(12).inner_margin(Margin::same(13));
        info.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✦").color(palette.accent).size(18.0));
                ui.vertical(|ui| {
                    ui.label(RichText::new("Try the Lucid Engine demo").color(palette.text).strong());
                    ui.label(RichText::new("A small local page rendered through Lucid's own HTML/CSS prototype.").color(palette.muted).size(11.0));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(RichText::new("Open demo").color(palette.text)).clicked() { self.navigate("lucid://demo", ctx.clone(), true); }
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
                ui.label(RichText::new("Unable to load this page").size(24.0).strong().color(palette.text));
                ui.add_space(8.0);
                ui.label(RichText::new(error).color(palette.muted));
                ui.label(RichText::new("Check the address and your internet connection.").size(12.0).color(palette.muted));
            });
            return;
        }
        if self.active().loading {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.spinner();
                ui.label(RichText::new("Loading with Lucid Engine…").color(palette.muted));
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
            ui.label(RichText::new(&page.title).strong().color(palette.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(&page.url).size(10.0).color(palette.muted));
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
                    PaintOp::Text { x, y, text, size, color, bold, italic, href } => {
                        let pos = response.rect.min + Vec2::new(*x, *y);
                        let font = FontId::proportional(*size);
                        let text_color = Color32::from_rgb(color.0, color.1, color.2);
                        let _emphasis = (*bold, *italic);
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
        ui.label(RichText::new(if self.profile.navigation_limit_kib == 0 { "Unlimited" } else { "Applied separately to each top-level HTML transfer; subresources are not loaded by this prototype." }).size(11.0).color(palette.muted));
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
        ctx.style_mut(|style| { style.animation_time = if self.profile.animations_enabled { 0.15 } else { 0.0 }; });
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
        ui.label(RichText::new("Lucid Engine 0.0.3 · Experimental").color(palette.text));
        ui.label(RichText::new("Original HTML subset parser · CSS subset · block layout · paint operations").size(11.0).color(palette.muted));
        ui.label(RichText::new("Not yet supported: JavaScript, standards-complete HTML/CSS, images, media, site isolation, extensions and full subresource loading.").size(11.0).color(palette.muted));
        ui.add_space(12.0);
        if ui.button("Save settings now").clicked() { self.persist_profile(); }
    }
}

impl eframe::App for LucidApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_loads();
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
        ctx.style_mut(|style| { style.animation_time = if self.profile.animations_enabled { 0.15 } else { 0.0 }; });

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
                        if ui.small_button(label).clicked() { self.navigate(url, ctx.clone(), true); }
                    }
                });
            }
            ui.add_space(8.0);
            let status = Frame::new().fill(palette.surface).corner_radius(10).inner_margin(Margin::symmetric(11, 6));
            status.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if self.theme == Theme::Calm { "●" } else { "◆" }).color(palette.good).size(9.0));
                    ui.label(RichText::new(&self.status).size(10.0).color(palette.muted));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let tab_count = self.tabs.iter().filter(|tab| tab.workspace == self.active_workspace).count();
                        ui.label(RichText::new(format!("{} TAB{}", tab_count, if tab_count == 1 { "" } else { "S" })).size(9.0).monospace().color(palette.muted));
                        ui.label(RichText::new(if self.theme == Theme::Calm { "CALM MODE" } else { "NIGHT CITY · LOCAL SYSTEM" }).size(9.0).monospace().color(palette.accent_alt));
                    });
                });
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(12).inner_margin(Margin::same(6)).show(ui, |ui| {
                        ui.set_width(46.0);
                        ui.vertical_centered(|ui| {
                            for (glyph, label, panel) in [
                                ("⌂", "Home", "home"), ("☆", "Library", "library"), ("◷", "History", "history"),
                                ("◉", "GX Control", "control"), ("✦", "Mods", "mods"), ("◈", "GX Corner", "corner"),
                                ("⇩", "Downloads", "downloads"), ("⚙", "Settings", "settings"),
                            ] {
                                let active = match panel {
                                    "library" => self.show_library, "history" => self.show_history,
                                    "control" => self.show_gx_control, "mods" => self.show_mods,
                                    "corner" => self.show_corner, "downloads" => self.show_downloads,
                                    "settings" => self.show_settings,
                                    _ => !self.show_library && !self.show_history && !self.show_gx_control && !self.show_mods && !self.show_corner && !self.show_downloads && !self.show_settings,
                                };
                                let button = egui::Button::new(RichText::new(glyph).size(20.0).color(if active { palette.accent } else { palette.muted }))
                                    .fill(if active { palette.surface_raised } else { palette.surface }).corner_radius(10);
                                if ui.add(button).on_hover_text(label).clicked() {
                                    if panel == "home" {
                                        self.close_panels();
                                        self.navigate("lucid://home", ctx.clone(), true);
                                    } else {
                                        self.open_panel(panel);
                                    }
                                }
                            }
                        });
                    });

                let available = ui.available_width();
                Frame::new().fill(palette.background).inner_margin(Margin::same(8)).show(ui, |ui| {
                    ui.set_min_width(available);
                    ui.set_min_height(ui.available_height().max(500.0));
                    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        if self.show_settings { self.draw_settings(ui, palette); }
                        else if self.show_gx_control { self.draw_control_center(ui, palette); }
                        else if self.show_history { self.draw_history(ui, palette, ctx.clone()); }
                        else if self.show_mods { self.draw_mods(ui, palette, &ctx); }
                        else if self.show_corner { self.draw_gx_corner(ui, palette, ctx.clone()); }
                        else if self.show_downloads { self.draw_downloads(ui, palette); }
                        else if self.show_library { self.draw_library(ui, palette, ctx.clone()); }
                        else if self.active().page.is_some() || self.active().error.is_some() || self.active().loading || self.active().url != "lucid://home" {
                            self.draw_page(ui, palette);
                        } else {
                            self.draw_home(ui, palette, ctx.clone());
                        }
                    });
                });
            });
        });

        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) { self.show_command_palette = true; }
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
    }
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
    let limited = RateLimitedReader::new(response.take(MAX_HTML_DOCUMENT_BYTES + 1), network_limit_kib);
    let bytes_read = limited.read_to_string(&mut body)
        .map_err(|e| format!("Could not read HTML response: {e}"))?;
    if bytes_read as u64 > MAX_HTML_DOCUMENT_BYTES {
        return Err("The HTML document exceeded Lucid's current 5 MiB prototype limit.".into());
    }
    Ok(body)
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucid Browser")
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([860.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native("Lucid Browser", options, Box::new(|_cc| Ok(Box::<LucidApp>::default())))
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
