use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserScope {
    User,
    Machine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserRegistration {
    pub display_name: String,
    pub registry_id: String,
    pub scope: BrowserScope,
    pub executable_path: Option<String>,
}

pub fn dedupe_browsers(browsers: Vec<BrowserRegistration>) -> Vec<BrowserRegistration> {
    let mut by_key = BTreeMap::new();

    for browser in browsers {
        let key = browser_key(&browser);
        by_key
            .entry(key)
            .and_modify(|existing: &mut BrowserRegistration| {
                if existing.executable_path.is_none() {
                    existing.executable_path = browser.executable_path.clone();
                }
            })
            .or_insert(browser);
    }

    by_key.into_values().collect()
}

fn browser_key(browser: &BrowserRegistration) -> String {
    browser.registry_id.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser(
        display_name: &str,
        registry_id: &str,
        scope: BrowserScope,
        executable_path: Option<&str>,
    ) -> BrowserRegistration {
        BrowserRegistration {
            display_name: display_name.to_string(),
            registry_id: registry_id.to_string(),
            scope,
            executable_path: executable_path.map(str::to_string),
        }
    }

    #[test]
    fn dedupe_merges_matching_identifier_and_display_name_case_insensitively() {
        let browsers = dedupe_browsers(vec![
            browser("Google Chrome", "ChromeHTML", BrowserScope::User, None),
            browser(
                "google chrome",
                "chromehtml",
                BrowserScope::Machine,
                Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            ),
        ]);

        assert_eq!(browsers.len(), 1);
        assert_eq!(browsers[0].display_name, "Google Chrome");
        assert_eq!(
            browsers[0].executable_path.as_deref(),
            Some(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
        );
    }

    #[test]
    fn dedupe_keeps_distinct_browsers_with_the_same_display_name() {
        let browsers = dedupe_browsers(vec![
            browser("Firefox", "FirefoxURL", BrowserScope::Machine, None),
            browser("Firefox", "FirefoxURL-308046B0AF4A39CB", BrowserScope::User, None),
        ]);

        assert_eq!(browsers.len(), 2);
    }

    #[test]
    fn dedupe_merges_matching_identifier_with_different_display_names() {
        let browsers = dedupe_browsers(vec![
            browser("Firefox", "Firefox-308046B0AF4A39CB", BrowserScope::Machine, None),
            browser(
                "Mozilla Firefox",
                "Firefox-308046B0AF4A39CB",
                BrowserScope::Machine,
                Some(r"C:\Program Files\Mozilla Firefox\firefox.exe"),
            ),
        ]);

        assert_eq!(browsers.len(), 1);
        assert_eq!(browsers[0].display_name, "Firefox");
        assert_eq!(
            browsers[0].executable_path.as_deref(),
            Some(r"C:\Program Files\Mozilla Firefox\firefox.exe")
        );
    }
}
