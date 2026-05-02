use crate::browser::BrowserRegistration;

#[cfg(windows)]
use crate::browser::dedupe_browsers;
#[cfg(windows)]
use crate::browser::BrowserScope;
#[cfg(windows)]
use winreg::{enums::*, RegKey};

pub fn discover_browsers() -> Vec<BrowserRegistration> {
    discover_browsers_impl()
}

#[cfg(not(windows))]
fn discover_browsers_impl() -> Vec<BrowserRegistration> {
    Vec::new()
}

#[cfg(windows)]
fn discover_browsers_impl() -> Vec<BrowserRegistration> {
    let mut browsers = Vec::new();

    browsers.extend(read_registered_applications(
        RegKey::predef(HKEY_CURRENT_USER),
        "Software\\RegisteredApplications",
        BrowserScope::User,
    ));
    browsers.extend(read_registered_applications(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "Software\\RegisteredApplications",
        BrowserScope::Machine,
    ));
    browsers.extend(read_registered_applications(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "Software\\WOW6432Node\\RegisteredApplications",
        BrowserScope::Machine,
    ));

    browsers.extend(read_start_menu_clients(
        RegKey::predef(HKEY_CURRENT_USER),
        "Software\\Clients\\StartMenuInternet",
        BrowserScope::User,
    ));
    browsers.extend(read_start_menu_clients(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "Software\\Clients\\StartMenuInternet",
        BrowserScope::Machine,
    ));
    browsers.extend(read_start_menu_clients(
        RegKey::predef(HKEY_LOCAL_MACHINE),
        "Software\\WOW6432Node\\Clients\\StartMenuInternet",
        BrowserScope::Machine,
    ));

    dedupe_browsers(browsers)
}

#[cfg(windows)]
fn read_registered_applications(
    root: RegKey,
    path: &str,
    scope: BrowserScope,
) -> Vec<BrowserRegistration> {
    let Ok(key) = root.open_subkey(path) else {
        return Vec::new();
    };

    key.enum_values()
        .filter_map(Result::ok)
        .filter_map(|(value_name, _)| {
            let capabilities_path = key.get_value::<String, _>(&value_name).ok()?;

            if !capabilities_include_web_protocols(&root, &capabilities_path) {
                return None;
            }

            let display_name = read_application_name(&root, &capabilities_path)
                .unwrap_or_else(|| value_name.clone());
            let executable_path = read_application_icon_path(&root, &capabilities_path);

            browser_if_valid(display_name, value_name, scope.clone(), executable_path)
        })
        .collect()
}

#[cfg(windows)]
fn read_application_name(root: &RegKey, capabilities_path: &str) -> Option<String> {
    let key = root.open_subkey(capabilities_path).ok()?;
    key.get_value::<String, _>("ApplicationName")
        .ok()
        .filter(|name| !name.trim().is_empty())
}

#[cfg(windows)]
fn read_application_icon_path(root: &RegKey, capabilities_path: &str) -> Option<String> {
    let key = root.open_subkey(capabilities_path).ok()?;
    key.get_value::<String, _>("ApplicationIcon")
        .ok()
        .and_then(|icon| extract_icon_file_path(&icon))
}

#[cfg(windows)]
fn read_start_menu_clients(
    root: RegKey,
    path: &str,
    scope: BrowserScope,
) -> Vec<BrowserRegistration> {
    let Ok(key) = root.open_subkey(path) else {
        return Vec::new();
    };

    key.enum_keys()
        .filter_map(Result::ok)
        .filter_map(|client_id| {
            let client_key = key.open_subkey(&client_id).ok()?;
            let capabilities_path = format!("{path}\\{client_id}\\Capabilities");

            if !capabilities_include_web_protocols(&root, &capabilities_path) {
                return None;
            }

            let display_name = client_key
                .get_value::<String, _>("")
                .ok()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| client_id.clone());
            let executable_path = read_application_icon_path(&root, &capabilities_path).or_else(|| {
                client_key
                    .open_subkey("shell\\open\\command")
                    .ok()
                    .and_then(|command_key| command_key.get_value::<String, _>("").ok())
                    .and_then(|command| extract_executable_path(&command))
            });

            browser_if_valid(display_name, client_id, scope.clone(), executable_path)
        })
        .collect()
}

#[cfg(windows)]
fn capabilities_include_web_protocols(root: &RegKey, capabilities_path: &str) -> bool {
    let Ok(url_associations) = root.open_subkey(format!("{capabilities_path}\\URLAssociations"))
    else {
        return false;
    };

    url_associations.get_value::<String, _>("http").is_ok()
        && url_associations.get_value::<String, _>("https").is_ok()
}

#[cfg(windows)]
fn browser_if_valid(
    display_name: String,
    registry_id: String,
    scope: BrowserScope,
    executable_path: Option<String>,
) -> Option<BrowserRegistration> {
    let display_name = display_name.trim().to_string();
    let registry_id = registry_id.trim().to_string();

    if display_name.is_empty() || registry_id.is_empty() {
        return None;
    }

    Some(BrowserRegistration {
        display_name,
        registry_id,
        scope,
        executable_path,
    })
}

fn extract_executable_path(command: &str) -> Option<String> {
    let command = command.trim();

    if command.is_empty() {
        return None;
    }

    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }

    command
        .split_whitespace()
        .next()
        .map(str::to_string)
        .filter(|path| !path.is_empty())
}

fn extract_icon_file_path(icon_location: &str) -> Option<String> {
    let icon_location = icon_location.trim();

    if icon_location.is_empty() || icon_location.starts_with('@') {
        return None;
    }

    let path = if let Some(rest) = icon_location.strip_prefix('"') {
        let end = rest.find('"')?;
        &rest[..end]
    } else if let Some((path, _index)) = icon_location.rsplit_once(',') {
        path.trim()
    } else {
        icon_location
    };

    let path = path.trim().trim_matches('"');
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

#[cfg(test)]
fn includes_required_web_protocols(protocols: &[&str]) -> bool {
    let has_http = protocols
        .iter()
        .any(|protocol| protocol.eq_ignore_ascii_case("http"));
    let has_https = protocols
        .iter()
        .any(|protocol| protocol.eq_ignore_ascii_case("https"));

    has_http && has_https
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_quoted_executable_path_from_shell_command() {
        let path = extract_executable_path(
            r#""C:\Program Files\Mozilla Firefox\firefox.exe" -osint -url "%1""#,
        );

        assert_eq!(
            path.as_deref(),
            Some(r"C:\Program Files\Mozilla Firefox\firefox.exe")
        );
    }

    #[test]
    fn extracts_unquoted_executable_path_from_shell_command() {
        let path = extract_executable_path(r"C:\Browser\browser.exe --flag");

        assert_eq!(path.as_deref(), Some(r"C:\Browser\browser.exe"));
    }

    #[test]
    fn ignores_blank_shell_commands() {
        assert_eq!(extract_executable_path("  "), None);
    }

    #[test]
    fn extracts_path_from_application_icon_location() {
        let path = extract_icon_file_path(r"C:\Program Files\Arc\Arc.exe,0");

        assert_eq!(path.as_deref(), Some(r"C:\Program Files\Arc\Arc.exe"));
    }

    #[test]
    fn extracts_quoted_path_from_application_icon_location() {
        let path = extract_icon_file_path(r#""C:\Program Files\Arc\Arc.exe",0"#);

        assert_eq!(path.as_deref(), Some(r"C:\Program Files\Arc\Arc.exe"));
    }

    #[test]
    fn ignores_indirect_resource_icon_locations() {
        let path = extract_icon_file_path(
            r"@{Microsoft.Paint_11.2601.441.0_x64__8wekyb3d8bbwe?ms-resource://Microsoft.Paint/Resources/AppDisplayName}",
        );

        assert_eq!(path, None);
    }

    #[test]
    fn treats_capabilities_with_http_and_https_as_browser_capabilities() {
        assert!(includes_required_web_protocols(&["http", "https", "mailto"]));
    }

    #[test]
    fn rejects_capabilities_without_full_web_protocol_pair() {
        assert!(!includes_required_web_protocols(&["mailto", "ms-appinstaller"]));
        assert!(!includes_required_web_protocols(&["http"]));
        assert!(!includes_required_web_protocols(&["https"]));
    }
}
