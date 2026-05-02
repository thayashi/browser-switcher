# Default Browser Tray Utility Design

## Summary

Build a small Windows 11 utility named **Default Browser Tray**. The app runs in the task tray, lists installed browsers, and opens the Windows Default Apps settings page for the selected browser. The user still changes defaults in Windows Settings; the app only removes the navigation friction.

## Goals

- Run as a lightweight tray-resident Windows app.
- Detect installed browsers from Windows registration data.
- Show detected browsers in the tray menu.
- Open the selected browser's Default Apps settings page directly.
- Provide menu actions to refresh the browser list and quit the app.
- Keep the implementation suitable for packaging as a Windows executable.

## Non-Goals

- Do not change the default browser silently or by writing protected registry associations.
- Do not support Windows 10 in the first version.
- Do not build a full settings UI unless tray behavior alone proves insufficient.
- Do not sync settings or persist user profiles.

## Platform And Stack

- Target OS: Windows 11.
- App framework: Tauri 2 with Rust.
- Frontend: minimal static Tauri surface only if required by the framework; the primary UI is the tray menu.
- Windows integration:
  - Read browser registrations from the registry.
  - Launch Windows Settings through `ms-settings:` URIs.

Development can happen from WSL because the project lives under `C:\Users\thaya\LocalDocs\dev\default-browser-tray`, but Windows-native tooling is required for tray runtime checks, `ms-settings:` behavior, and final Windows builds.

## Browser Discovery

The app will discover browsers from Windows registration locations rather than hardcoding a fixed list. The first implementation will check:

- `HKCU\Software\RegisteredApplications`
- `HKLM\Software\RegisteredApplications`
- `HKLM\Software\WOW6432Node\RegisteredApplications`
- `HKCU\Software\Clients\StartMenuInternet`
- `HKLM\Software\Clients\StartMenuInternet`
- `HKLM\Software\WOW6432Node\Clients\StartMenuInternet`

Each browser entry will be normalized into:

- Display name, such as `Google Chrome`.
- Registry identifier, such as a `RegisteredApplications` value name or StartMenuInternet client id.
- Scope, either user or machine.
- Optional executable path if available.

Duplicate entries will be merged by stable identifier and display name. If a browser cannot be mapped to a Default Apps deep link, it can still appear disabled or be omitted from the first release depending on observed Windows behavior.

## Opening Default Apps

When a tray item is selected, the app will launch Windows Settings with the most specific URI available for that browser. Candidate URI forms are:

- `ms-settings:defaultapps?registeredAppUser=<registered-app-name>`
- `ms-settings:defaultapps?registeredAppMachine=<registered-app-name>`

The exact mapping depends on where Windows registered the browser. If a direct browser page cannot be opened for a detected browser, the fallback is `ms-settings:defaultapps`.

The app will not simulate clicks inside Settings. It will only open the intended Settings page and let the user complete the change through the supported Windows UI.

## Tray Menu

The tray menu will contain:

- One menu item per detected browser.
- `Refresh browsers`.
- `Quit`.

The menu is rebuilt after refresh. If no browsers are detected, it shows a disabled `No browsers found` item plus `Refresh browsers` and `Quit`.

## Error Handling

- Registry read failures should not crash the app; inaccessible hives are skipped.
- If launching a deep link fails, the app should fall back to opening `ms-settings:defaultapps`.
- If both deep link and fallback fail, the app should log the error and keep running.
- Invalid or incomplete registry entries should be ignored.

## Testing

Unit tests should cover:

- Registry parsing helpers using sample data.
- Browser deduplication.
- Settings URI generation for user and machine registrations.

Manual Windows checks should cover:

- App starts and appears in the task tray.
- Tray menu lists installed browsers.
- Refresh updates the menu.
- Selecting Chrome, Edge, Firefox, or another installed browser opens that browser's Default Apps page when supported.
- Quit exits the tray process.

## Open Constraints

- Windows Settings URI behavior can vary by Windows 11 build. The app must keep a safe fallback to the general Default Apps page.
- Final packaging requires Windows-native Rust, Node, MSVC build tools, and WebView2 runtime support.
