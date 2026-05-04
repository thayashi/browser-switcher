use crate::browser::{BrowserRegistration, BrowserScope};

pub const DEFAULT_APPS_URI: &str = "ms-settings:defaultapps";

pub fn default_apps_uri_for(browser: &BrowserRegistration) -> String {
    let parameter = match browser.scope {
        BrowserScope::User => "registeredAppUser",
        BrowserScope::Machine => "registeredAppMachine",
        BrowserScope::AppModel => "registeredAUMID",
    };

    format!(
        "{DEFAULT_APPS_URI}?{parameter}={}",
        percent_encode_query_value(&browser.registry_id)
    )
}

fn percent_encode_query_value(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser(scope: BrowserScope, registry_id: &str) -> BrowserRegistration {
        BrowserRegistration {
            display_name: "Browser".to_string(),
            registry_id: registry_id.to_string(),
            scope,
            executable_path: None,
            icon_path: None,
            url_protocol_ids: vec![registry_id.to_string()],
        }
    }

    #[test]
    fn builds_user_registered_app_uri() {
        let uri = default_apps_uri_for(&browser(BrowserScope::User, "FirefoxURL"));

        assert_eq!(
            uri,
            "ms-settings:defaultapps?registeredAppUser=FirefoxURL"
        );
    }

    #[test]
    fn builds_machine_registered_app_uri() {
        let uri = default_apps_uri_for(&browser(BrowserScope::Machine, "ChromeHTML"));

        assert_eq!(
            uri,
            "ms-settings:defaultapps?registeredAppMachine=ChromeHTML"
        );
    }

    #[test]
    fn percent_encodes_registry_ids_for_query_strings() {
        let uri = default_apps_uri_for(&browser(BrowserScope::User, "Contoso Browser"));

        assert_eq!(
            uri,
            "ms-settings:defaultapps?registeredAppUser=Contoso%20Browser"
        );
    }

    #[test]
    fn builds_app_model_registered_app_uri() {
        let uri = default_apps_uri_for(&browser(
            BrowserScope::AppModel,
            "TheBrowserCompany.Arc_ttt1ap7aakyb4!Arc",
        ));

        assert_eq!(
            uri,
            "ms-settings:defaultapps?registeredAUMID=TheBrowserCompany.Arc_ttt1ap7aakyb4%21Arc"
        );
    }
}
