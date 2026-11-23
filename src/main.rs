use eframe::egui;
use egui::{Color32, RichText, Vec2};

#[derive(Clone, Copy, PartialEq)]
enum Theme { Calm, Cyberpunk }

#[derive(Clone)]
struct Tab { title: String, address: String }

struct LucidApp {
    theme: Theme,
    tabs: Vec<Tab>,
    active_tab: usize,
    address: String,
    page_message: String,
    sidebar_open: bool,
    command_open: bool,
    settings_open: bool,
    downloads_open: bool,
    history_open: bool,
    accent_strength: f32,
    compact_mode: bool,
    animations: bool,
    workspace: String,
    search_query: String,
}

impl Default for LucidApp {
    fn default() -> Self {
        Self {
            theme: Theme::Calm,
            tabs: vec![Tab { title: "New tab".into(), address: "lucid://newtab".into() }],
            active_tab: 0,
            address: String::new(),
            page_message: String::new(),
            sidebar_open: true,
            command_open: false,
            settings_open: false,
            downloads_open: false,
            history_open: false,
            accent_strength: 0.55,
            compact_mode: false,
            animations: true,
            workspace: "Personal".into(),
            search_query: String::new(),
        }
    }
}

impl LucidApp {
    fn palette(&self) -> (Color32, Color32, Color32, Color32, Color32) {
        match self.theme {
            Theme::Calm => (
                Color32::from_rgb(19, 18, 24),
                Color32::from_rgb(27, 25, 34),
                Color32::from_rgb(39, 35, 49),
                Color32::from_rgb(195, 171, 239),
                Color32::from_rgb(237, 233, 245),
            ),
            Theme::Cyberpunk => (
                Color32::from_rgb(8, 9, 13),
                Color32::from_rgb(17, 17, 24),
                Color32::from_rgb(34, 23, 34),
                Color32::from_rgb(255, 64, 105),
                Color32::from_rgb(230, 240, 248),
            ),
        }
    }

    fn navigate(&mut self, value: String) {
        let value = value.trim().to_string();
        if value.is_empty() { return; }
        let url = if value.contains("://") || value.starts_with("about:") || value.starts_with("lucid:") {
            value
        } else if value.contains('.') && !value.contains(' ') {
            format!("https://{value}")
        } else {
            format!("https://www.google.com/search?q={}", value.replace(' ', "+"))
        };
        self.address = url.clone();
        self.page_message = format!(
            "Lucid Engine prototype\n\nNavigation target: {url}\n\nThe independent HTML, CSS, layout, paint and JavaScript pipeline is under active development. This build does not yet fetch or render arbitrary websites. No external browser engine is embedded."
        );
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.address = url.clone();
            tab.title = url;
        }
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab { title: "New tab".into(), address: "lucid://newtab".into() });
        self.active_tab = self.tabs.len() - 1;
        self.address.clear();
        self.page_message.clear();
    }

    fn header(&mut self, ctx: &egui::Context, bg: Color32, panel: Color32, accent: Color32, text: Color32) {
        egui::TopBottomPanel::top("tab_strip").exact_height(43.0).frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin::symmetric(12, 7))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("◉").color(accent).size(17.0));
                for i in 0..self.tabs.len() {
                    let selected = i == self.active_tab;
                    let fill = if selected { panel } else { bg };
                    let title = self.tabs[i].title.chars().take(19).collect::<String>();
                    let response = ui.add(egui::Button::new(RichText::new(title).color(if selected { text } else { Color32::GRAY }).size(12.0)).fill(fill).corner_radius(egui::CornerRadius::same(8)));
                    if response.clicked() { self.active_tab = i; self.address = self.tabs[i].address.clone(); }
                    if response.middle_clicked() && self.tabs.len() > 1 {
                        self.tabs.remove(i);
                        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
                    }
                }
                if ui.button(RichText::new("+").color(accent).size(18.0)).clicked() { self.new_tab(); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("—").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)); }
                    if ui.small_button("□").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true)); }
                    if ui.small_button("×").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
            });
        });

        egui::TopBottomPanel::top("navigation").exact_height(54.0).frame(egui::Frame::new().fill(panel).inner_margin(egui::Margin::symmetric(14, 9))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("‹").clicked() { self.page_message = "Back navigation history will be connected to the session controller.".into(); }
                if ui.button("›").clicked() { self.page_message = "Forward navigation history will be connected to the session controller.".into(); }
                if ui.button("↻").clicked() { self.page_message = "Reload will be connected to the document loader.".into(); }
                let response = ui.add_sized([ui.available_width() - 190.0, 32.0], egui::TextEdit::singleline(&mut self.address).hint_text("Search or enter an address").desired_width(f32::INFINITY));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(self.address.clone()); }
                if ui.button("Go").clicked() { self.navigate(self.address.clone()); }
                if ui.button("☆").clicked() { self.page_message = "Bookmark manager is part of the browser-shell roadmap.".into(); }
                if ui.button("⌘K").clicked() { self.command_open = true; }
            });
        });
    }

    fn home(&mut self, ui: &mut egui::Ui, accent: Color32, text: Color32, card: Color32) {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("lucid").size(46.0).color(accent).strong());
            ui.label(RichText::new("A little more clarity.").size(14.0).color(Color32::GRAY));
            ui.add_space(20.0);
            let search = ui.add_sized([ui.available_width().min(620.0), 46.0], egui::TextEdit::singleline(&mut self.search_query).hint_text("Search anything or enter a URL…").desired_width(f32::INFINITY));
            if search.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.navigate(self.search_query.clone()); }
            ui.add_space(22.0);
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("YOUR WORKSPACE").size(10.0).color(Color32::GRAY).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("workspace").selected_text(&self.workspace).show_ui(ui, |ui| {
                    for name in ["Personal", "Gaming", "School", "Development"] { ui.selectable_value(&mut self.workspace, name.to_string(), name); }
                });
            });
        });
        ui.add_space(10.0);
        egui::Grid::new("workspace_cards").num_columns(4).spacing([12.0, 12.0]).show(ui, |ui| {
            for (name, icon, tint) in [
                ("Gaming", "◈", Color32::from_rgb(172, 142, 225)),
                ("School", "▤", Color32::from_rgb(145, 169, 224)),
                ("Development", "</>", Color32::from_rgb(125, 190, 180)),
                ("Personal", "✧", accent),
            ] {
                egui::Frame::new().fill(card).corner_radius(12).inner_margin(14).show(ui, |ui| {
                    ui.set_min_width(125.0);
                    if ui.button(RichText::new(icon).size(22.0).color(tint)).clicked() { self.workspace = name.into(); }
                    ui.label(RichText::new(name).color(text).strong());
                    ui.label(RichText::new("Workspace").size(10.0).color(Color32::GRAY));
                });
            }
            ui.end_row();
        });
        ui.add_space(25.0);
        ui.label(RichText::new("QUICK ACCESS").size(10.0).color(Color32::GRAY).strong());
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for (name, url) in [("YouTube", "https://youtube.com"), ("GitHub", "https://github.com"), ("Discord", "https://discord.com"), ("Wikipedia", "https://wikipedia.org")] {
                if ui.add(egui::Button::new(RichText::new(format!("◇  {name}")).color(text)).fill(card).corner_radius(10)).clicked() { self.navigate(url.into()); }
            }
            if ui.button("+ Add shortcut").clicked() { self.page_message = "Custom shortcut management is planned; no shortcut was created yet.".into(); }
        });
        ui.add_space(25.0);
        ui.label(RichText::new("YOUR BROWSER").size(10.0).color(Color32::GRAY).strong());
        ui.add_space(8.0);
        ui.columns(3, |columns| {
            for (i, (title, detail)) in [
                ("Downloads", "Download manager foundation"),
                ("Recent activity", "Browsing history foundation"),
                ("Privacy center", "Protection controls roadmap"),
            ].iter().enumerate() {
                egui::Frame::new().fill(card).corner_radius(12).inner_margin(14).show(&mut columns[i], |ui| {
                    ui.label(RichText::new(*title).color(text).strong());
                    ui.label(RichText::new(*detail).size(11.0).color(Color32::GRAY));
                });
            }
        });
    }
}

impl eframe::App for LucidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (bg, panel, card, accent, text) = self.palette();
        let mut visuals = if self.theme == Theme::Calm { egui::Visuals::dark() } else { egui::Visuals::dark() };
        visuals.panel_fill = bg;
        visuals.window_fill = panel;
        visuals.extreme_bg_color = Color32::from_rgb(12, 11, 16);
        visuals.widgets.noninteractive.bg_fill = panel;
        visuals.widgets.inactive.bg_fill = card;
        visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.22);
        visuals.widgets.active.bg_fill = accent.gamma_multiply(0.38);
        visuals.selection.bg_fill = accent.gamma_multiply(0.45);
        visuals.override_text_color = Some(text);
        ctx.set_visuals(visuals);

        self.header(ctx, bg, panel, accent, text);

        if self.sidebar_open {
            egui::SidePanel::left("lucid_sidebar").default_width(58.0).resizable(false).frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin::symmetric(8, 12))).show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    for (icon, label) in [("⌂", "Home"), ("☆", "Bookmarks"), ("◷", "History"), ("↓", "Downloads"), ("▦", "Workspaces"), ("⚙", "Settings")] {
                        let response = ui.add_sized([38.0, 38.0], egui::Button::new(RichText::new(icon).size(19.0).color(accent)).fill(bg).corner_radius(10));
                        response.on_hover_text(label);
                        if response.clicked() {
                            match label { "History" => self.history_open = !self.history_open, "Downloads" => self.downloads_open = !self.downloads_open, "Settings" => self.settings_open = !self.settings_open, _ => self.page_message = format!("{label} panel is being implemented.") }
                        }
                    }
                    ui.add_space(12.0);
                    if ui.button("☰").on_hover_text("Hide sidebar").clicked() { self.sidebar_open = false; }
                });
            });
        } else {
            egui::Area::new("show_sidebar".into()).fixed_pos(egui::pos2(8.0, 112.0)).show(ctx, |ui| {
                if ui.button("☰").clicked() { self.sidebar_open = true; }
            });
        }

        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin::symmetric(26, 16))).show(ctx, |ui| {
            if self.page_message.is_empty() {
                egui::ScrollArea::vertical().show(ui, |ui| self.home(ui, accent, text, card));
            } else {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("PAGE").color(accent).strong());
                    if ui.button("Return to new tab").clicked() { self.page_message.clear(); self.address = "lucid://newtab".into(); }
                });
                ui.add_space(18.0);
                egui::Frame::new().fill(panel).corner_radius(14).inner_margin(22).show(ui, |ui| {
                    ui.label(RichText::new(&self.page_message).color(text).size(15.0));
                });
            }
        });

        if self.settings_open {
            egui::Window::new("Lucid Settings").open(&mut self.settings_open).default_size([430.0, 420.0]).show(ctx, |ui| {
                ui.heading("Appearance");
                ui.label("Choose your Lucid experience");
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.theme == Theme::Calm, "Lucid Calm").clicked() { self.theme = Theme::Calm; }
                    if ui.selectable_label(self.theme == Theme::Cyberpunk, "Lucid Cyberpunk").clicked() { self.theme = Theme::Cyberpunk; }
                });
                ui.add(egui::Slider::new(&mut self.accent_strength, 0.0..=1.0).text("Accent intensity"));
                ui.checkbox(&mut self.compact_mode, "Compact interface density");
                ui.checkbox(&mut self.animations, "Enable interface motion");
                ui.separator();
                ui.label("Engine status");
                ui.label("Independent renderer: experimental");
                ui.label("JavaScript: not implemented");
                ui.label("Arbitrary website rendering: not implemented");
            });
        }
        if self.downloads_open {
            egui::Window::new("Downloads").open(&mut self.downloads_open).show(ctx, |ui| {
                ui.label("No active downloads.");
                ui.label("The download service will be connected after the network stack is implemented.");
            });
        }
        if self.history_open {
            egui::Window::new("Recent activity").open(&mut self.history_open).show(ctx, |ui| {
                ui.label("No browsing history in this prototype.");
                ui.label("History storage and privacy controls are planned.");
            });
        }
        if self.command_open {
            egui::Window::new("Command palette").open(&mut self.command_open).default_size([460.0, 250.0]).show(ctx, |ui| {
                ui.label("Quick actions");
                if ui.button("New tab").clicked() { self.new_tab(); self.command_open = false; }
                if ui.button("Toggle theme").clicked() { self.theme = if self.theme == Theme::Calm { Theme::Cyberpunk } else { Theme::Calm }; }
                if ui.button("Open settings").clicked() { self.settings_open = true; self.command_open = false; }
                if ui.button("Toggle sidebar").clicked() { self.sidebar_open = !self.sidebar_open; self.command_open = false; }
            });
        }
        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) { self.command_open = true; }
        let _ = Vec2::ZERO;
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1360.0, 860.0]).with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native("Lucid Browser", options, Box::new(|_cc| Ok(Box::new(LucidApp::default()))))
}
