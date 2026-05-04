#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use browser_switcher::{
    browser::{browser_menu_label, BrowserRegistration},
    icons::browser_icon,
    registry::{current_default_browser_protocol_id, discover_browsers},
    settings::{default_apps_uri_for, DEFAULT_APPS_URI},
};
use tauri::{
    menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
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
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            refresh_tray_icon(app);
            show_about(app);
        }))
        .setup(|app| {
            let browsers = discover_browsers();
            app.manage(AppState {
                browsers: Mutex::new(browsers.clone()),
            });

            build_tray_icon(app.handle(), &browsers)?;
            if should_show_about_on_start() {
                show_about(app.handle());
            }
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

fn handle_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    else {
        return;
    };

    if button_state != MouseButtonState::Up {
        return;
    }

    if matches!(button, MouseButton::Left | MouseButton::Right) {
        refresh_and_show_tray_menu(tray);
    }
}

fn build_menu(
    app: &AppHandle,
    browsers: &[BrowserRegistration],
    default_protocol_id: Option<&str>,
) -> tauri::Result<Menu<tauri::Wry>> {
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
            let label = browser_menu_label(browser, default_protocol_id);
            if let Some(icon) = browser_icon(browser) {
                let item = IconMenuItem::with_id(
                    app,
                    id,
                    &label,
                    true,
                    Some(icon),
                    None::<&str>,
                )?;
                menu.append(&item)?;
            } else {
                let item = MenuItem::with_id(
                    app,
                    id,
                    &label,
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

fn build_tray_icon(app: &AppHandle, browsers: &[BrowserRegistration]) -> tauri::Result<()> {
    let _ = app.remove_tray_by_id(TRAY_ID);

    let default_protocol_id = current_default_browser_protocol_id();
    let menu = build_menu(app, browsers, default_protocol_id.as_deref())?;
    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(APP_NAME)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_icon_event);

    let builder = if let Some(icon) = app.default_window_icon() {
        builder.icon(icon.clone())
    } else {
        builder
    };

    let tray = builder.build(app)?;
    let _ = tray.with_inner_tray_icon(|tray| {
        tray.set_show_menu_on_right_click(false);
    });
    Ok(())
}

fn refresh_tray_icon(app: &AppHandle) {
    let browsers = app
        .try_state::<AppState>()
        .and_then(|state| state.browsers.lock().ok().map(|browsers| browsers.clone()))
        .unwrap_or_else(discover_browsers);

    let _ = build_tray_icon(app, &browsers);
}

fn refresh_and_show_tray_menu(tray: &TrayIcon) {
    let app = tray.app_handle();
    let browsers = app
        .try_state::<AppState>()
        .and_then(|state| state.browsers.lock().ok().map(|browsers| browsers.clone()))
        .unwrap_or_else(discover_browsers);
    let default_protocol_id = current_default_browser_protocol_id();

    if let Ok(menu) = build_menu(app, &browsers, default_protocol_id.as_deref()) {
        let _ = tray.set_menu(Some(menu));
    }

    let _ = tray.with_inner_tray_icon(|tray| {
        tray.show_menu();
    });
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

fn should_show_about_on_start() -> bool {
    cfg!(debug_assertions) && std::env::var_os("BROWSER_SWITCHER_SHOW_ABOUT_ON_START").is_some()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_window_is_hidden_by_default_on_startup() {
        std::env::remove_var("BROWSER_SWITCHER_SHOW_ABOUT_ON_START");

        assert!(!should_show_about_on_start());
    }

    #[test]
    fn about_window_can_be_shown_on_startup_for_development_layout_checks() {
        std::env::set_var("BROWSER_SWITCHER_SHOW_ABOUT_ON_START", "1");

        assert!(should_show_about_on_start());

        std::env::remove_var("BROWSER_SWITCHER_SHOW_ABOUT_ON_START");
    }
}
