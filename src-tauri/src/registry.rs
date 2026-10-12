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

pub fn current_default_browser_protocol_id() -> Option<String> {
    current_default_browser_protocol_id_impl()
}

#[cfg(not(windows))]
fn discover_browsers_impl() -> Vec<BrowserRegistration> {
    Vec::new()
}

#[cfg(not(windows))]
fn current_default_browser_protocol_id_impl() -> Option<String> {
    None
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
fn current_default_browser_protocol_id_impl() -> Option<String> {
    use windows::{
        core::w,
        Win32::{
            System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
            UI::Shell::{ApplicationAssociationRegistration, IApplicationAssociationRegistration},
        },
    };

    let _apartment = AssociationQueryApartment::initialize()?;
    let registration: IApplicationAssociationRegistration = unsafe {
        CoCreateInstance(
            &ApplicationAssociationRegistration,
            None,
            CLSCTX_INPROC_SERVER,
        )
        .ok()?
    };

    // Preserve HTTP priority if users configure different HTTP/HTTPS handlers.
    // Never fall back to UserChoice/UserChoiceLatest: either can be stale.
    query_default_protocol_id(&registration, w!("http"))
        .or_else(|| query_default_protocol_id(&registration, w!("https")))
}

#[cfg(windows)]
fn query_default_protocol_id(
    registration: &windows::Win32::UI::Shell::IApplicationAssociationRegistration,
    protocol: windows::core::PCWSTR,
) -> Option<String> {
    use windows::Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{AL_EFFECTIVE, AT_URLPROTOCOL},
    };

    // Query the handler Windows would actually use, rather than registry storage.
    let prog_id = unsafe {
        registration
            .QueryCurrentDefault(protocol, AT_URLPROTOCOL, AL_EFFECTIVE)
            .ok()?
    };
    let value = if prog_id.is_null() {
        None
    } else {
        unsafe { prog_id.to_string().ok() }
    };
    // The API allocates this string; release it even if decoding fails.
    unsafe { CoTaskMemFree(Some(prog_id.0.cast())) };

    value
        .map(|prog_id| prog_id.trim().to_string())
        .filter(|prog_id| !prog_id.is_empty())
}

#[cfg(windows)]
struct AssociationQueryApartment(bool);

#[cfg(windows)]
impl AssociationQueryApartment {
    fn initialize() -> Option<Self> {
        use windows::Win32::{
            Foundation::RPC_E_CHANGED_MODE,
            System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
        };

        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result.is_ok() {
            // S_OK and S_FALSE both require a matching CoUninitialize.
            Some(Self(true))
        } else if result == RPC_E_CHANGED_MODE {
            // The caller already owns a different apartment (e.g. MTA).
            Some(Self(false))
        } else {
            None
        }
    }
}

#[cfg(windows)]
impl Drop for AssociationQueryApartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
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

            let url_protocol_ids = read_web_protocol_ids(&root, &capabilities_path);
            if url_protocol_ids.is_empty() {
                return None;
            }

            let display_name = read_application_name(&root, &capabilities_path)
                .unwrap_or_else(|| value_name.clone());
            let executable_path = read_application_icon_path(&root, &capabilities_path);

            browser_if_valid(
                display_name,
                value_name,
                scope.clone(),
                executable_path,
                url_protocol_ids,
            )
            .map(apply_app_model_registration_if_available)
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

            let url_protocol_ids = read_web_protocol_ids(&root, &capabilities_path);
            if url_protocol_ids.is_empty() {
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

            browser_if_valid(
                display_name,
                client_id,
                scope.clone(),
                executable_path,
                url_protocol_ids,
            )
            .map(apply_app_model_registration_if_available)
        })
        .collect()
}

#[cfg(windows)]
fn apply_app_model_registration_if_available(
    mut browser: BrowserRegistration,
) -> BrowserRegistration {
    if let Some((aumid, protocol_ids, icon_path)) = find_app_model_registration_for_browser(&browser)
    {
        browser.registry_id = aumid;
        browser.scope = BrowserScope::AppModel;
        browser.icon_path = icon_path;
        merge_protocol_ids(&mut browser.url_protocol_ids, &protocol_ids);
    }

    browser
}

#[cfg(windows)]
fn find_app_model_registration_for_browser(
    browser: &BrowserRegistration,
) -> Option<(String, Vec<String>, Option<String>)> {
    let root = RegKey::predef(HKEY_CLASSES_ROOT);
    let packages_path =
        "Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\PackageRepository\\Packages";
    let packages = root.open_subkey(packages_path).ok()?;

    packages.enum_keys().filter_map(Result::ok).find_map(|package_id| {
        let package = packages.open_subkey(&package_id).ok()?;
        package
            .enum_keys()
            .filter_map(Result::ok)
            .filter(|aumid| aumid.contains('!'))
            .find_map(|aumid| {
                let app = package.open_subkey(&aumid).ok()?;
                let http = app.open_subkey("windows.protocol\\http").ok()?;
                let https = app.open_subkey("windows.protocol\\https").ok()?;

                if !app_model_protocol_matches_browser(&http, browser) {
                    return None;
                }

                let protocol_ids = read_app_model_protocol_ids(&root, &package_id);
                let mut protocol_ids = protocol_ids;
                merge_protocol_ids_from_key(&mut protocol_ids, &http);
                merge_protocol_ids_from_key(&mut protocol_ids, &https);
                let icon_path = read_app_model_icon_path(&http).or_else(|| read_app_model_icon_path(&https));

                Some((aumid, protocol_ids, icon_path))
            })
    })
}

#[cfg(windows)]
fn app_model_protocol_matches_browser(protocol: &RegKey, browser: &BrowserRegistration) -> bool {
    protocol
        .get_value::<String, _>("DisplayName")
        .ok()
        .is_some_and(|name| name.trim().eq_ignore_ascii_case(&browser.display_name))
}

#[cfg(windows)]
fn read_app_model_protocol_ids(root: &RegKey, package_id: &str) -> Vec<String> {
    ["http", "https"]
        .into_iter()
        .flat_map(|protocol| {
            let path = format!(
                "Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\PackageRepository\\Extensions\\windows.protocol\\{protocol}"
            );
            let Ok(key) = root.open_subkey(path) else {
                return Vec::new();
            };

            key.enum_keys()
                .filter_map(Result::ok)
                .filter(|protocol_id| {
                    key.open_subkey(protocol_id)
                        .ok()
                        .and_then(|protocol_key| protocol_key.get_value::<String, _>(package_id).ok())
                        .is_some()
                })
                .collect::<Vec<_>>()
        })
        .fold(Vec::new(), |mut protocol_ids, protocol_id| {
            merge_protocol_id(&mut protocol_ids, protocol_id);
            protocol_ids
        })
}

#[cfg(windows)]
fn read_app_model_icon_path(protocol: &RegKey) -> Option<String> {
    protocol
        .get_value::<String, _>("Logo")
        .ok()
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
}

#[cfg(windows)]
fn read_web_protocol_ids(root: &RegKey, capabilities_path: &str) -> Vec<String> {
    let Ok(url_associations) = root.open_subkey(format!("{capabilities_path}\\URLAssociations"))
    else {
        return Vec::new();
    };

    let http = url_associations.get_value::<String, _>("http").ok();
    let https = url_associations.get_value::<String, _>("https").ok();

    if http.is_none() || https.is_none() {
        return Vec::new();
    }

    [http, https]
        .into_iter()
        .flatten()
        .map(|protocol_id| protocol_id.trim().to_string())
        .filter(|protocol_id| !protocol_id.is_empty())
        .fold(Vec::new(), |mut protocol_ids, protocol_id| {
            if !protocol_ids
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(&protocol_id))
            {
                protocol_ids.push(protocol_id);
            }
            protocol_ids
        })
}

#[cfg(windows)]
fn merge_protocol_ids(existing: &mut Vec<String>, incoming: &[String]) {
    for protocol_id in incoming {
        merge_protocol_id(existing, protocol_id.clone());
    }
}

#[cfg(windows)]
fn merge_protocol_ids_from_key(existing: &mut Vec<String>, key: &RegKey) {
    if let Ok(protocol_id) = key.get_value::<String, _>("ACID") {
        merge_protocol_id(existing, protocol_id);
    }
}

#[cfg(windows)]
fn merge_protocol_id(existing: &mut Vec<String>, protocol_id: String) {
    let protocol_id = protocol_id.trim().to_string();
    if protocol_id.is_empty() {
        return;
    }

    if !existing
        .iter()
        .any(|existing_id| existing_id.eq_ignore_ascii_case(&protocol_id))
    {
        existing.push(protocol_id);
    }
}

#[cfg(windows)]
fn browser_if_valid(
    display_name: String,
    registry_id: String,
    scope: BrowserScope,
    executable_path: Option<String>,
    url_protocol_ids: Vec<String>,
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
        icon_path: executable_path.clone(),
        executable_path,
        url_protocol_ids,
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

    #[cfg(windows)]
    fn apartment_type() -> windows::core::Result<windows::Win32::System::Com::APTTYPE> {
        use windows::Win32::System::Com::{CoGetApartmentType, APTTYPE, APTTYPEQUALIFIER};

        let mut kind = APTTYPE::default();
        let mut qualifier = APTTYPEQUALIFIER::default();
        unsafe { CoGetApartmentType(&mut kind, &mut qualifier)? };
        Ok(kind)
    }

    #[cfg(windows)]
    #[test]
    fn default_query_balances_com_initialization_on_a_fresh_thread() {
        use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

        std::thread::spawn(|| {
            let _ = current_default_browser_protocol_id();
            // A leaked STA initialization would prevent switching to MTA.
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED).ok().unwrap();
                CoUninitialize();
            }
        })
        .join()
        .unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn default_query_preserves_the_callers_sta_and_mta_apartments() {
        use windows::Win32::System::Com::{
            CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
        };

        for mode in [COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED] {
            std::thread::spawn(move || {
                unsafe { CoInitializeEx(None, mode).ok().unwrap() };
                let before = apartment_type().unwrap();
                for _ in 0..3 {
                    let _ = current_default_browser_protocol_id();
                    assert_eq!(apartment_type().unwrap(), before);
                }
                unsafe { CoUninitialize() };
                // A leaked initialization would prevent changing apartments.
                let other_mode = if mode == COINIT_APARTMENTTHREADED {
                    COINIT_MULTITHREADED
                } else {
                    COINIT_APARTMENTTHREADED
                };
                unsafe {
                    CoInitializeEx(None, other_mode).ok().unwrap();
                    CoUninitialize();
                }
            })
            .join()
            .unwrap();
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "Reads live Windows settings; set BROWSER_SWITCHER_EXPECT_DEFAULT_PROG_ID for verification"]
    fn reports_current_default_browser() {
        let expected = std::env::var("BROWSER_SWITCHER_EXPECT_DEFAULT_PROG_ID")
            .expect("Set BROWSER_SWITCHER_EXPECT_DEFAULT_PROG_ID to the expected Windows ProgID");
        let protocol_id = current_default_browser_protocol_id().expect("No effective browser found");
        println!("Effective default browser ProgID: {protocol_id}");
        assert!(protocol_id.eq_ignore_ascii_case(&expected));

        let marked: Vec<_> = discover_browsers()
            .into_iter()
            .filter(|browser| {
                crate::browser::browser_matches_default_protocol(browser, Some(&protocol_id))
            })
            .map(|browser| crate::browser::browser_menu_label(&browser, Some(&protocol_id)))
            .collect();
        println!("Marked browser menu labels: {marked:?}");
        assert!(
            !marked.is_empty(),
            "Default browser is missing from the menu"
        );
    }

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
