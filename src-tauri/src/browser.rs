use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserScope {
    User,
    Machine,
    AppModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserRegistration {
    pub display_name: String,
    pub registry_id: String,
    pub scope: BrowserScope,
    pub executable_path: Option<String>,
    pub icon_path: Option<String>,
    pub url_protocol_ids: Vec<String>,
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
                if existing.icon_path.is_none() {
                    existing.icon_path = browser.icon_path.clone();
                }
                merge_protocol_ids(&mut existing.url_protocol_ids, &browser.url_protocol_ids);
            })
            .or_insert(browser);
    }

    let mut browsers: Vec<_> = by_key.into_values().collect();
    browsers.sort_by_key(|browser| browser.display_name.to_ascii_lowercase());
    browsers
}

pub fn browser_menu_label(
    browser: &BrowserRegistration,
    default_protocol_id: Option<&str>,
) -> String {
    if browser_matches_default_protocol(browser, default_protocol_id) {
        format!("{} ✓", browser.display_name)
    } else {
        browser.display_name.clone()
    }
}

pub fn browser_matches_default_protocol(
    browser: &BrowserRegistration,
    default_protocol_id: Option<&str>,
) -> bool {
    default_protocol_id.is_some_and(|default_protocol_id| {
        let default_protocol_id = default_protocol_id.trim();

        browser
            .registry_id
            .trim()
            .eq_ignore_ascii_case(default_protocol_id)
            || browser.url_protocol_ids.iter().any(|protocol_id| {
                protocol_id
                    .trim()
                    .eq_ignore_ascii_case(default_protocol_id)
            })
    })
}

fn merge_protocol_ids(existing: &mut Vec<String>, incoming: &[String]) {
    for protocol_id in incoming {
        if !existing
            .iter()
            .any(|existing_id| existing_id.eq_ignore_ascii_case(protocol_id))
        {
            existing.push(protocol_id.clone());
        }
    }
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
            icon_path: None,
            url_protocol_ids: vec![registry_id.to_string()],
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

    #[test]
    fn dedupe_sorts_browsers_by_display_name() {
        let browsers = dedupe_browsers(vec![
            browser("Google Chrome", "ChromeHTML", BrowserScope::Machine, None),
            browser("Arc", "TheBrowserCompany.Arc_ttt1ap7aakyb4!Arc", BrowserScope::AppModel, None),
            browser("Firefox", "FirefoxURL", BrowserScope::Machine, None),
        ]);

        let names: Vec<_> = browsers
            .iter()
            .map(|browser| browser.display_name.as_str())
            .collect();

        assert_eq!(names, vec!["Arc", "Firefox", "Google Chrome"]);
    }

    #[test]
    fn menu_label_marks_default_browser() {
        let browser = browser("Google Chrome", "ChromeHTML", BrowserScope::Machine, None);

        assert_eq!(
            browser_menu_label(&browser, Some("ChromeHTML")),
            "Google Chrome ✓"
        );
    }

    #[test]
    fn menu_label_does_not_mark_non_default_browser() {
        let browser = browser("Firefox", "FirefoxURL", BrowserScope::Machine, None);

        assert_eq!(browser_menu_label(&browser, Some("ChromeHTML")), "Firefox");
    }

    #[test]
    fn default_browser_match_is_case_insensitive() {
        let browser = browser("Google Chrome", "ChromeHTML", BrowserScope::Machine, None);

        assert!(browser_matches_default_protocol(
            &browser,
            Some("chromehtml")
        ));
    }

    #[test]
    fn default_browser_matches_url_protocol_association_id() {
        let mut browser = browser("Google Chrome", "Google Chrome", BrowserScope::Machine, None);
        browser.url_protocol_ids = vec!["ChromeHTML".to_string()];

        assert_eq!(
            browser_menu_label(&browser, Some("ChromeHTML")),
            "Google Chrome ✓"
        );
    }
}
