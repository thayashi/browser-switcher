#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use browser_switcher::{
    browser::BrowserRegistration,
    icons::browser_icon,
    registry::discover_browsers,
    settings::{default_apps_uri_for, DEFAULT_APPS_URI},
};
use tauri::{
    menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, WindowEvent,
};

const APP_NAME: &str = "Browser Switcher";
const TRAY_ID: &str = "browser-switcher";
const ABOUT_ID: &str = "about";
const QUIT_ID: &str = "quit";
const BROWSER_PREFIX: &str = "browser:";

struct AppState {
    browsers: Mutex<Vec<BrowserRegistration>>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let browsers = discover_browsers();
            app.manage(AppState {
                browsers: Mutex::new(browsers.clone()),
            });

            let menu = build_menu(app.handle(), &browsers)?;
            let builder = TrayIconBuilder::with_id(TRAY_ID)
                .tooltip(APP_NAME)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(handle_menu_event);

            let builder = if let Some(icon) = app.default_window_icon() {
                builder.icon(icon.clone())
            } else {
                builder
            };

            builder.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Browser Switcher");
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id.as_ref();

    if id == QUIT_ID {
        app.exit(0);
        return;
    }

    if id == ABOUT_ID {
        show_about(app);
        return;
    }

    if let Some(index) = id
        .strip_prefix(BROWSER_PREFIX)
        .and_then(|raw| raw.parse::<usize>().ok())
    {
        open_browser_settings(app, index);
    }
}

fn build_menu(app: &AppHandle, browsers: &[BrowserRegistration]) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;

    if browsers.is_empty() {
        let empty = MenuItem::with_id(
            app,
            "no-browsers",
            "No browsers found",
            false,
            None::<&str>,
        )?;
        menu.append(&empty)?;
    } else {
        for (index, browser) in browsers.iter().enumerate() {
            let id = format!("{BROWSER_PREFIX}{index}");
            if let Some(icon) = browser_icon(browser) {
                let item = IconMenuItem::with_id(
                    app,
                    id,
                    &browser.display_name,
                    true,
                    Some(icon),
                    None::<&str>,
                )?;
                menu.append(&item)?;
            } else {
                let item = MenuItem::with_id(
                    app,
                    id,
                    &browser.display_name,
                    true,
                    None::<&str>,
                )?;
                menu.append(&item)?;
            }
        }
    }

    let separator = PredefinedMenuItem::separator(app)?;
    let about = MenuItem::with_id(app, ABOUT_ID, "About Browser Switcher", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;

    menu.append(&separator)?;
    menu.append(&about)?;
    menu.append(&quit)?;

    Ok(menu)
}

fn open_browser_settings(app: &AppHandle, index: usize) {
    let browser = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .browsers
                .lock()
                .ok()
                .and_then(|browsers| browsers.get(index).cloned())
        });

    let Some(browser) = browser else {
        return;
    };

    let browser_uri = default_apps_uri_for(&browser);
    if open_settings_uri(&browser_uri).is_err() {
        let _ = open_settings_uri(DEFAULT_APPS_URI);
    }
}

fn show_about(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn open_settings_uri(uri: &str) -> std::io::Result<()> {
    open_settings_uri_impl(uri)
}

#[cfg(windows)]
fn open_settings_uri_impl(uri: &str) -> std::io::Result<()> {
    use windows::{
        core::PCWSTR,
        Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    };

    let operation = widestring("open");
    let target = widestring(uri);
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 as isize <= 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("ShellExecuteW failed with code {}", result.0 as isize),
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn widestring(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(not(windows))]
fn open_settings_uri_impl(uri: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(uri).spawn().map(|_| ())
}
