#![cfg(target_os = "windows")]

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

use cef::*;
use super::BrowserEvent;

#[derive(Clone)]
pub struct CefCallbacks {
    pub tab_id: u64,
    pub sender: Sender<BrowserEvent>,
    pub context: egui::Context,
}

wrap_display_handler! {
    pub struct LucidDisplayHandler {
        callbacks: CefCallbacks,
    }

    impl DisplayHandler {
        fn on_address_change(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            url: Option<&CefString>,
        ) {
            if !frame.is_some_and(|frame| frame.is_main() != 0) {
                return;
            }
            if let Some(url) = url {
                let _ = self.callbacks.sender.send(BrowserEvent::CefAddressChanged {
                    tab_id: self.callbacks.tab_id,
                    url: url.to_string(),
                });
                self.callbacks.context.request_repaint();
            }
        }

        fn on_title_change(
            &self,
            _browser: Option<&mut Browser>,
            title: Option<&CefString>,
        ) {
            if let Some(title) = title {
                let _ = self.callbacks.sender.send(BrowserEvent::CefTitleChanged {
                    tab_id: self.callbacks.tab_id,
                    title: title.to_string(),
                });
                self.callbacks.context.request_repaint();
            }
        }
    }
}

wrap_client! {
    pub struct LucidClient {
        callbacks: CefCallbacks,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(LucidDisplayHandler::new(self.callbacks.clone()))
        }
    }
}

wrap_browser_process_handler! {
    pub struct LucidBrowserProcessHandler {
        context: egui::Context,
    }

    impl BrowserProcessHandler {
        fn on_schedule_message_pump_work(&self, delay_ms: i64) {
            let delay = Duration::from_millis(delay_ms.max(0) as u64);
            self.context.request_repaint_after(delay);
        }
    }
}

wrap_app! {
    pub struct LucidCefApp {
        context: egui::Context,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(LucidBrowserProcessHandler::new(self.context.clone()))
        }
    }
}

pub struct HostedBrowser {
    pub browser: Browser,
    pub client: Client,
    pub child_hwnd: isize,
}

pub fn initialize(
    context: egui::Context,
    main_args: &MainArgs,
    sandbox_info: *mut u8,
) -> Result<LucidCefApp, String> {
    let app_data = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "Windows did not provide an APPDATA directory for the Chromium profile.".to_owned())?;
    let root = app_data.join("Lucid").join("Chromium");
    let profile = root.join("Default");
    std::fs::create_dir_all(&profile)
        .map_err(|error| format!("Could not create Lucid's Chromium profile: {error}"))?;

    let mut settings = Settings::default();
    settings.external_message_pump = 1;
    settings.root_cache_path = CefString::from(root.to_string_lossy().as_ref());
    settings.cache_path = CefString::from(profile.to_string_lossy().as_ref());
    settings.persist_session_cookies = 1;
    settings.persist_user_preferences = 1;

    let mut app = LucidCefApp::new(context);
    let initialized = cef::initialize(
        Some(main_args),
        Some(&settings),
        Some(&mut app),
        sandbox_info,
    );
    if initialized != 1 {
        return Err("Chromium Embedded Framework could not initialize. Check the bundled CEF runtime files.".into());
    }
    Ok(app)
}

pub fn create_browser(
    parent_hwnd: isize,
    callbacks: CefCallbacks,
    url: &str,
) -> Result<HostedBrowser, String> {
    let bounds = Rect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
    };
    let window_info = WindowInfo::default().set_as_child(parent_hwnd as _, &bounds);
    let browser_settings = BrowserSettings::default();
    let mut client = LucidClient::new(callbacks);
    let cef_url = CefString::from(url);
    let browser = cef::browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client),
        Some(&cef_url),
        Some(&browser_settings),
        None,
        None,
    )
    .ok_or_else(|| "Chromium could not create a browser child window.".to_owned())?;

    let host = browser
        .host()
        .ok_or_else(|| "Chromium created a page without a browser host.".to_owned())?;
    let child_hwnd = host.window_handle().0 as isize;
    if child_hwnd == 0 {
        return Err("Chromium returned an invalid browser window handle.".into());
    }

    Ok(HostedBrowser {
        browser,
        client,
        child_hwnd,
    })
}

pub fn initialize_process(
    main_args: &MainArgs,
    sandbox_info: *mut u8,
) -> Result<(), String> {
    let code = cef::execute_process(
        Some(main_args),
        None::<&mut App>,
        sandbox_info,
    );
    if code >= 0 {
        return Err(format!("Chromium subprocess exited with code {code}."));
    }
    Ok(())
}
