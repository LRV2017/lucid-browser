use eframe::egui::{self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit, Vec2};
use lucid_engine::{Document, PaintOp, Stylesheet, collect_styles, document_title, layout_document, parse_html};
use std::fs;
use std::path::PathBuf;
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
}

impl Tab {
    fn new(id: u64, workspace: String) -> Self {
        Self { id, workspace, title: "New tab".into(), url: "lucid://home".into(), page: None, error: None, history: vec!["lucid://home".into()], history_index: 0 }
    }
}

enum LoadResult { Success { tab_id: u64, url: String, html: String }, Failure { tab_id: u64, url: String, message: String } }

struct LucidApp {
    theme: Theme,
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: u64,
    address: String,
    tx: Sender<LoadResult>,
    rx: Receiver<LoadResult>,
    loading: bool,
    status: String,
    show_library: bool,
    show_settings: bool,
    show_command_palette: bool,
    command_query: String,
    active_workspace: String,
    bookmarks_bar: bool,
}

impl Default for LucidApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            theme: Theme::Calm,
            tabs: vec![Tab::new(1, "Personal".into())],
            active_tab: 0,
            next_tab_id: 2,
            address: String::new(),
            tx, rx,
            loading: false,
            status: "Your browser, in its natural state.".into(),
            show_library: false,
            show_settings: false,
            show_command_palette: false,
            command_query: String::new(),
            active_workspace: "Personal".into(),
            bookmarks_bar: false,
        }
    }
}

impl LucidApp {
    fn active(&self) -> &Tab { &self.tabs[self.active_tab] }
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
        let tab_id = self.active().id;
        {
            let tab = self.active_mut();
            tab.url = url.clone();
            tab.title = if url == "lucid://home" { "New tab".into() } else { "Loading…".into() };
            tab.page = None;
            tab.error = None;
            if push_history {
                if tab.history_index + 1 < tab.history.len() { tab.history.truncate(tab.history_index + 1); }
                if tab.history.last() != Some(&url) { tab.history.push(url.clone()); }
                tab.history_index = tab.history.len().saturating_sub(1);
            }
        }
        self.address = url.clone();
        self.show_library = false;
        self.show_settings = false;
        if url == "lucid://home" {
            self.loading = false;
            self.status = "Ready".into();
            return;
        }
        self.loading = true;
        self.status = format!("Connecting to {url}");
        let tx = self.tx.clone();
        thread::spawn(move || {
            let fetched = fetch_document(&url);
            let event = match fetched {
                Ok(html) => LoadResult::Success { tab_id, url, html },
                Err(message) => LoadResult::Failure { tab_id, url, message },
            };
            let _ = tx.send(event);
            ctx.request_repaint_after(Duration::from_millis(20));
        });
    }

    fn poll_loads(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                LoadResult::Success { tab_id, url, html } => {
                    let document = parse_html(&html);
                    let stylesheet = collect_styles(&document);
                    let title = document_title(&document).unwrap_or_else(|| url.clone());
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.title = title.clone();
                        tab.url = url.clone();
                        tab.page = Some(LoadedPage { url, title, document, stylesheet });
                        tab.error = None;
                    }
                    self.loading = false;
                    self.status = "Loaded with Lucid Engine · JavaScript not yet supported".into();
                }
                LoadResult::Failure { tab_id, url, message } => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.title = "Page unavailable".into();
                        tab.url = url;
                        tab.error = Some(message.clone());
                        tab.page = None;
                    }
                    self.loading = false;
                    self.status = "Navigation failed".into();
                }
            }
        }
    }

    fn execute_command(&mut self, command: &str, ctx: egui::Context) {
        match command {
            "New tab" => self.new_tab(),
            "Toggle theme" => self.theme = if self.theme == Theme::Calm { Theme::Cyberpunk } else { Theme::Calm },
            "Open library" => self.show_library = !self.show_library,
            "Settings" => self.show_settings = true,
            "Home" => self.navigate("lucid://home", ctx, true),
            _ => {}
        }
        self.show_command_palette = false;
        self.command_query.clear();
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("L").size(23.0).strong().color(palette.accent));
            ui.label(RichText::new("lucid").size(21.0).strong().color(palette.text));
            ui.label(RichText::new("ENGINE 0.0.1").size(9.0).color(palette.muted).monospace());
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
            for (index, tab) in self.tabs.iter().enumerate().filter(|(_, tab)| tab.workspace == self.active_workspace) {
                let selected = index == self.active_tab;
                let fill = if selected { palette.surface_raised } else { palette.background };
                let title = if tab.title.chars().count() > 18 { format!("{}…", tab.title.chars().take(17).collect::<String>()) } else { tab.title.clone() };
                let radius = if self.theme == Theme::Calm { 9 } else { 2 };
                let button = egui::Button::new(RichText::new(title).size(12.0).color(if selected { palette.text } else { palette.muted })).fill(fill).stroke(Stroke::new(if selected { 1.0 } else { 0.0 }, palette.border)).corner_radius(radius);
                if ui.add(button).clicked() { self.active_tab = index; self.address = self.tabs[index].url.clone(); }
                if ui.add(egui::Button::new(RichText::new("×").color(palette.muted).size(12.0)).fill(fill).corner_radius(9)).clicked() { close_index = Some(index); }
            }
            if let Some(index) = close_index { self.close_tab(index); }
            if ui.add(egui::Button::new(RichText::new("＋").size(18.0).color(palette.accent)).fill(palette.surface).corner_radius(9)).clicked() { self.new_tab(); }
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
            let address = ui.add_sized([ui.available_width() - 128.0, 34.0], TextEdit::singleline(&mut self.address).hint_text("Search or enter a website address…").desired_width(f32::INFINITY).margin(Vec2::new(12.0, 7.0)));
            if address.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(&self.address.clone(), ctx.clone(), true); }
            if ui.add(egui::Button::new(RichText::new("Go").color(palette.text)).fill(palette.accent_alt).corner_radius(9)).clicked() { self.navigate(&self.address.clone(), ctx.clone(), true); }
            if ui.add(egui::Button::new("◇").fill(palette.surface).corner_radius(9)).clicked() { self.show_library = !self.show_library; }
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
                for workspace in ["Personal", "Gaming", "School", "Research"] {
                    let selected = self.active_workspace == workspace;
                    if ui.add(egui::Button::new(RichText::new(workspace).size(11.0).color(if selected { palette.text } else { palette.muted })).fill(if selected { palette.surface_raised } else { palette.background }).stroke(Stroke::new(if selected { 1.0 } else { 0.0 }, palette.border)).corner_radius(10)).clicked() {
                        self.switch_workspace(workspace);
                    }
                }
            });
        });
        ui.add_space(34.0);
        ui.horizontal(|ui| { ui.label(RichText::new("QUICK ACCESS").size(10.0).strong().color(palette.muted)); ui.add_space(5.0); ui.label(RichText::new("Your favorite places").size(11.0).color(palette.muted)); });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            for (label, sub, url, glyph) in [
                ("YouTube", "Video", "https://www.youtube.com", "▶"),
                ("GitHub", "Development", "https://github.com", "⌘"),
                ("Reddit", "Community", "https://www.reddit.com", "r/"),
                ("Wikipedia", "Knowledge", "https://www.wikipedia.org", "W"),
            ] {
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
            stat_card(&mut columns[1], palette, "◈", "Engine", "0.0.1", "Experimental renderer");
            stat_card(&mut columns[2], palette, "◇", "Privacy", "Local-first", "No Lucid account required");
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
            if ui.add(egui::Button::new(RichText::new("OPEN LIBRARY").monospace().color(palette.text)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3)).clicked() { self.show_library = true; }
            if ui.add(egui::Button::new(RichText::new("SYSTEM CONFIG").monospace().color(palette.text)).fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(3)).clicked() { self.show_settings = true; }
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
        if self.loading {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.spinner();
                ui.label(RichText::new("Loading with Lucid Engine…").color(palette.muted));
            });
            return;
        }
        let ctx = ui.ctx().clone();
        let Some(page) = self.active().page.clone() else { self.draw_home(ui, palette, ctx); return; };
        ui.horizontal(|ui| {
            ui.label(RichText::new(&page.title).strong().color(palette.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(&page.url).size(10.0).color(palette.muted));
            });
        });
        ui.separator();
        let width = ui.available_width().max(320.0);
        let layout = layout_document(&page.document, &page.stylesheet, width);
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(Vec2::new(width, layout.height.max(280.0)), Sense::hover());
            for operation in &layout.operations {
                match operation {
                    PaintOp::Rect { x, y, width, height, color, radius } => {
                        let rect = egui::Rect::from_min_size(response.rect.min + Vec2::new(*x, *y), Vec2::new(*width, *height));
                        painter.rect_filled(rect, CornerRadius::same((*radius).clamp(0.0, 255.0) as u8), Color32::from_rgb(color.0, color.1, color.2));
                    }
                    PaintOp::Text { x, y, text, size, color, bold, italic } => {
                        let pos = response.rect.min + Vec2::new(*x, *y);
                        let font = FontId::proportional(*size);
                        let text_color = Color32::from_rgb(color.0, color.1, color.2);
                        let _emphasis = (*bold, *italic);
                        painter.text(pos, Align2::LEFT_TOP, text, font, text_color);
                    }
                }
            }
        });
    }

    fn draw_library(&mut self, ui: &mut egui::Ui, palette: Palette, ctx: egui::Context) {
        ui.label(RichText::new("Your library").size(23.0).strong().color(palette.text));
        ui.label(RichText::new("Saved places, kept simple.").color(palette.muted));
        ui.add_space(15.0);
        for (name, url) in [("GitHub", "https://github.com"), ("Wikipedia", "https://wikipedia.org"), ("Rust documentation", "https://doc.rust-lang.org") ] {
            if ui.add(egui::Button::new(RichText::new(format!("◇   {name}")).color(palette.text)).fill(palette.surface).corner_radius(10)).clicked() { self.navigate(url, ctx.clone(), true); }
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.label(RichText::new("Settings").size(25.0).strong().color(palette.text));
        ui.label(RichText::new("Lucid's interface preferences").color(palette.muted));
        ui.add_space(18.0);
        ui.label(RichText::new("APPEARANCE").size(10.0).strong().color(palette.muted));
        ui.horizontal(|ui| {
            if ui.selectable_label(self.theme == Theme::Calm, RichText::new("Lucid Calm").color(palette.text)).clicked() { self.theme = Theme::Calm; }
            if ui.selectable_label(self.theme == Theme::Cyberpunk, RichText::new("Lucid Cyberpunk").color(palette.text)).clicked() { self.theme = Theme::Cyberpunk; }
        });
        ui.add_space(12.0);
        ui.checkbox(&mut self.bookmarks_bar, "Show bookmarks bar");
        ui.add_space(18.0);
        ui.label(RichText::new("ENGINE STATUS").size(10.0).strong().color(palette.muted));
        ui.label(RichText::new("Lucid Engine 0.0.1 · Experimental").color(palette.text));
        ui.label(RichText::new("HTML subset parser · Basic CSS cascade · Block layout · Paint list").size(11.0).color(palette.muted));
        ui.label(RichText::new("Not yet supported: JavaScript, full HTML5 parsing, modern CSS layout, site isolation, extension APIs.").size(11.0).color(palette.muted));
    }
}

impl eframe::App for LucidApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_loads();
        let palette = self.theme.palette();
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
                if self.show_library {
                    let frame = Frame::new().fill(palette.surface).stroke(Stroke::new(1.0, palette.border)).corner_radius(14).inner_margin(Margin::same(14));
                    frame.show(ui, |ui| { ui.set_width(190.0); self.draw_library(ui, palette, ctx.clone()); });
                }
                let available = ui.available_width();
                let frame = Frame::new().fill(palette.background).inner_margin(Margin::same(5));
                frame.show(ui, |ui| {
                    ui.set_min_width(available);
                    ui.set_min_height(ui.available_height().max(500.0));
                    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        if self.show_settings { self.draw_settings(ui, palette); }
                        else if self.active().page.is_some() || self.active().error.is_some() || self.loading { self.draw_page(ui, palette); }
                        else { self.draw_home(ui, palette, ctx.clone()); }
                    });
                });
            });
        });

        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) { self.show_command_palette = true; }
        if self.show_command_palette {
            egui::Window::new("Lucid Command Palette").collapsible(false).resizable(false).anchor(Align2::CENTER_TOP, [0.0, 90.0]).show(&ctx, |ui| {
                ui.set_width(390.0);
                ui.add(TextEdit::singleline(&mut self.command_query).hint_text("Search commands…").desired_width(f32::INFINITY));
                ui.separator();
                for command in ["New tab", "Toggle theme", "Open library", "Settings", "Home"] {
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
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("file://") { return value.into(); }
    if !value.chars().any(char::is_whitespace) && value.contains('.') { return format!("https://{value}"); }
    let encoded = value.bytes().map(|b| if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' { (b as char).to_string() } else { format!("%{b:02X}") }).collect::<Vec<_>>().join("");
    format!("https://duckduckgo.com/?q={encoded}")
}

fn fetch_document(url: &str) -> Result<String, String> {
    if url == "lucid://demo" {
        return Ok(r##"<!doctype html><html><head><title>Lucid Engine Demo</title><style>
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
            <p><a href="https://www.rust-lang.org">Learn about Rust (links are a planned interaction)</a></p>
        </body></html>"##.into());
    }
    if let Some(path) = url.strip_prefix("file://") {
        let path = if cfg!(windows) { path.trim_start_matches('/').replace('/', "\\") } else { path.to_owned() };
        return fs::read_to_string(PathBuf::from(path)).map_err(|e| format!("Could not read local file: {e}"));
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent("LucidBrowser/0.0.1 (experimental independent engine)")
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::limited(8))
        .build().map_err(|e| format!("Could not initialize network client: {e}"))?;
    let response = client.get(url).send().map_err(|e| format!("Network request failed: {e}"))?;
    if !response.status().is_success() { return Err(format!("Server returned {}", response.status())); }
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml+xml") {
        return Err(format!("This prototype currently displays HTML documents only (received {content_type})."));
    }
    response.text().map_err(|e| format!("Could not read response body: {e}"))
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
    }

    #[test]
    fn demo_page_parses_through_lucid_engine() {
        let html = fetch_document("lucid://demo").expect("demo should be local");
        let document = parse_html(&html);
        assert_eq!(document_title(&document).as_deref(), Some("Lucid Engine Demo"));
        let styles = collect_styles(&document);
        assert!(!styles.rules.is_empty());
        let layout = layout_document(&document, &styles, 900.0);
        assert!(!layout.operations.is_empty());
    }
}
