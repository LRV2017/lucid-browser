#![cfg(target_os = "windows")]

use std::path::PathBuf;
use eframe::egui;
use std::sync::mpsc::Sender;
use std::time::Duration;

use cef::{*, ImplBrowser, ImplRequest};
use super::adblock::AdBlockService;
use super::BrowserEvent;

#[derive(Clone)]
pub struct CefCallbacks {
    pub tab_id: u64,
    pub sender: Sender<BrowserEvent>,
    pub context: egui::Context,
    pub adblocker: AdBlockService,
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

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(LucidRequestHandler::new(self.callbacks.adblocker.clone()))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(LucidKeyboardHandler::new(self.callbacks.clone()))
        }
    }
}

wrap_keyboard_handler! {
    pub struct LucidKeyboardHandler {
        callbacks: CefCallbacks,
    }

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut windows_sys::Win32::UI::WindowsAndMessaging::MSG>,
            _is_keyboard_shortcut: Option<&mut i32>,
        ) -> i32 {
            let Some(event) = event else { return 0; };
            let control_pressed = event.modifiers & 4 != 0;
            let address_shortcut = event.type_ == KeyEventType::RAWKEYDOWN
                && event.windows_key_code == 0x4C
                && control_pressed;

            if address_shortcut {
                let _ = self.callbacks.sender.send(BrowserEvent::FocusAddressBar);
                self.callbacks.context.request_repaint();
                return 1;
            }

            0
        }
    }
}

wrap_request_handler! {
    pub struct LucidRequestHandler {
        adblocker: AdBlockService,
    }

    impl RequestHandler {
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            _is_navigation: i32,
            _is_download: i32,
            request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut i32>,
        ) -> Option<ResourceRequestHandler> {
            Some(LucidResourceRequestHandler::new(
                self.adblocker.clone(),
                request_initiator.map(|value| value.to_string()).unwrap_or_default(),
            ))
        }
    }
}

wrap_resource_request_handler! {
    pub struct LucidResourceRequestHandler {
        adblocker: AdBlockService,
        source_url: String,
    }

    impl ResourceRequestHandler {
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else {
                return ReturnValue::CONTINUE;
            };

            let cef_url = request.url();
            let utf16_url = CefStringUtf16::from(&cef_url);
            let url = CefStringUtf8::from(&utf16_url).to_string();

            let cef_method = request.method();
            let utf16_method = CefStringUtf16::from(&cef_method);
            let method = CefStringUtf8::from(&utf16_method).to_string();
            let cef_type = format!("{:?}", request.resource_type());
            let request_type = match cef_type.as_str() {
                "RT_MAIN_FRAME" | "RT_NAVIGATION_PRELOAD_MAIN_FRAME" => "main_frame",
                "RT_SUB_FRAME" | "RT_NAVIGATION_PRELOAD_SUB_FRAME" => "sub_frame",
                "RT_STYLESHEET" => "stylesheet",
                "RT_SCRIPT" | "RT_WORKER" | "RT_SHARED_WORKER" | "RT_SERVICE_WORKER" => "script",
                "RT_IMAGE" | "RT_FAVICON" => "image",
                "RT_FONT_RESOURCE" => "font",
                "RT_MEDIA" => "media",
                "RT_OBJECT" | "RT_PLUGIN_RESOURCE" => "object",
                "RT_XHR" => "xhr",
                "RT_PING" => "ping",
                "RT_CSP_REPORT" => "csp_report",
                _ => "other",
            };

            if self.adblocker.should_block(&url, &self.source_url, request_type, &method) {
                ReturnValue::CANCEL
            } else {
                ReturnValue::CONTINUE
            }
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
) -> Result<App, String> {
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

    let mut app: App = LucidCefApp::new(context);
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
    let window_info = WindowInfo::default().set_as_child(cef::sys::HWND(parent_hwnd as _), &bounds);
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

pub fn execute_subprocess(main_args: &MainArgs, sandbox_info: *mut u8) -> Option<i32> {
    let code = cef::execute_process(
        Some(main_args),
        None::<&mut App>,
        sandbox_info,
    );
    (code >= 0).then_some(code)
}
