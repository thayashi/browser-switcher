const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE_NAME: &str = "Browser Switcher";
const STARTUP_TASK_ID: &str = "BrowserSwitcherStartup";

pub fn is_enabled() -> std::io::Result<bool> {
    is_enabled_impl()
}

pub fn toggle() -> std::io::Result<bool> {
    let enabled = !is_enabled()?;
    set_enabled_impl(enabled)?;
    Ok(enabled)
}

/// Toggles startup from a dedicated MTA thread so a packaged app never blocks
/// Tauri's menu/UI thread while Windows completes `RequestEnableAsync`.
#[cfg(windows)]
pub fn toggle_on_worker_thread() -> std::io::Result<bool> {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(windows_error)?;
    }

    let result = toggle();

    unsafe {
        CoUninitialize();
    }

    result
}

#[cfg(not(windows))]
pub fn toggle_on_worker_thread() -> std::io::Result<bool> {
    toggle()
}

#[cfg(windows)]
fn is_enabled_impl() -> std::io::Result<bool> {
    if is_packaged() {
        let task = startup_task()?;
        return Ok(task.State().map_err(windows_error)?
            == windows::ApplicationModel::StartupTaskState::Enabled);
    }

    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = current_user.open_subkey(RUN_KEY) else {
        return Ok(false);
    };

    Ok(key.get_value::<String, _>(RUN_VALUE_NAME).is_ok())
}

#[cfg(windows)]
fn set_enabled_impl(enabled: bool) -> std::io::Result<()> {
    if is_packaged() {
        let task = startup_task()?;
        if enabled {
            let state = task
                .RequestEnableAsync()
                .map_err(windows_error)?
                .get()
                .map_err(windows_error)?;
            if state != windows::ApplicationModel::StartupTaskState::Enabled {
                return Err(std::io::Error::other(
                    "Windows has disabled this startup task; re-enable it in Startup Apps",
                ));
            }
        } else {
            task.Disable().map_err(windows_error)?;
        }
        return Ok(());
    }

    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    if enabled {
        let executable = std::env::current_exe()?;
        let (key, _) = current_user.create_subkey(RUN_KEY)?;
        key.set_value(RUN_VALUE_NAME, &quoted_executable_path(&executable))?;
    } else if let Ok(key) = current_user.open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_WRITE) {
        let _ = key.delete_value(RUN_VALUE_NAME);
    }

    Ok(())
}

#[cfg(windows)]
fn is_packaged() -> bool {
    windows::ApplicationModel::Package::Current().is_ok()
}

#[cfg(windows)]
fn startup_task() -> std::io::Result<windows::ApplicationModel::StartupTask> {
    use windows::{core::HSTRING, ApplicationModel::StartupTask};

    StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))
        .map_err(windows_error)?
        .get()
        .map_err(windows_error)
}

#[cfg(windows)]
fn windows_error(error: windows::core::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

#[cfg(windows)]
fn quoted_executable_path(path: &std::path::Path) -> String {
    format!("\"{}\"", path.display())
}

#[cfg(not(windows))]
fn is_enabled_impl() -> std::io::Result<bool> {
    Ok(false)
}

#[cfg(not(windows))]
fn set_enabled_impl(_enabled: bool) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::*;

    #[cfg(windows)]
    #[test]
    fn quotes_run_registry_executable_path() {
        assert_eq!(
            quoted_executable_path(std::path::Path::new(
                "C:\\Program Files\\Browser Switcher\\browser-switcher.exe"
            )),
            "\"C:\\Program Files\\Browser Switcher\\browser-switcher.exe\""
        );
    }
}
