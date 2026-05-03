# Browser Switcher Development Summary

Date: 2026-05-01

## Context

This project started as **Default Browser Tray**, a small Windows tray utility for opening the Windows Default Apps settings page for installed browsers. During development, the product name was changed to **Browser Switcher**.

The workspace folder was later renamed to:

```text
C:\Users\thaya\LocalDocs\dev\browser-switcher
```

The original design note is:

```text
docs/superpowers/specs/2026-05-01-default-browser-tray-utility-design.md
```

The original implementation plan is:

```text
docs/superpowers/plans/2026-05-01-default-browser-tray-implementation.md
```

## Product Direction

Browser Switcher is a Windows task tray app. It lists installed browsers and opens the Windows Default Apps settings flow for the selected browser.

The app intentionally does **not** change the default browser directly. It only opens the supported Windows Settings page so the user can complete the change through Windows UI.

## Current Stack

- Tauri 2
- Rust
- Minimal static HTML frontend for the About window
- Native Windows tray/menu UI
- Windows registry discovery for installed browsers
- Windows `ms-settings:` URI launch for Default Apps settings

## Implemented Behavior

- Runs as a tray app.
- Shows browser entries in the tray menu.
- Uses native Windows menu rendering.
- Browser menu entries can show icons.
- Browser icons are extracted from the browser executable when a normal executable path is available.
- If a browser-specific icon cannot be extracted, a separate fallback browser icon is used.
- App icon and fallback browser icon are separate files:
  - `src-tauri/icons/icon.ico`
  - `src-tauri/icons/browser-default.ico`
- Selecting a browser opens Windows Settings for default app selection when possible.
- If a browser-specific settings URI does not work, the app falls back to the general Default Apps settings page.
- `Refresh browsers` was removed from the tray menu.
- `About Browser Switcher` was added to the tray menu.
- The About window is hidden by default and can be opened from the tray menu.
- Closing the About window hides it instead of quitting the tray app.
- `Quit` exits the tray app.

## Browser Discovery Notes

Initial browser discovery listed too many registered Windows applications. The filtering was tightened so an entry must have both `http` and `https` URL association capabilities to be treated as a browser.

Firefox appeared twice because Windows registered it through both:

- `RegisteredApplications`
- `Clients\StartMenuInternet`

The duplicate entries shared the same registry identifier but had different display names, such as `Firefox` and `Mozilla Firefox`. Deduplication was changed so entries with the same `registry_id` are considered the same browser.

Arc required special consideration. Its normal registration did not expose a standard executable path suitable for icon extraction. A deeper MSIX/AppModel lookup was briefly explored, but it was removed because it was too specific and brittle. The current behavior is to use the default browser fallback icon when a standard icon source is unavailable.

## Environment Setup Done

The Windows development environment was completed with:

- Rust toolchain
- Cargo
- rustup
- Visual Studio Build Tools 2022 with MSVC and Windows SDK
- WebView2 runtime
- Node and npm

`npx tauri info` eventually reported the required Windows build environment as available.

## Verification Completed

At the end of the previous development session, these checks were passing:

```text
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

The latest reported Rust test count was 14 passing tests after removing the special Arc fallback and keeping the default browser icon fallback.

Manual testing confirmed:

- The tray app starts.
- The tray menu is usable.
- Browser list is filtered to browsers rather than all registered apps.
- Chrome selection opens the expected Windows Settings flow.
- Firefox duplicate entry was fixed.
- Browser icons show for browsers with extractable executable icons.
- A fallback icon appears when a specific browser icon is unavailable.

## Current Outputs

Build outputs have been generated under:

```text
src-tauri/target/release/
src-tauri/target/release/bundle/msi/
src-tauri/target/release/bundle/nsis/
```

Known generated artifacts include:

```text
browser-switcher.exe
Browser Switcher_0.1.0_x64_en-US.msi
Browser Switcher_0.1.0_x64-setup.exe
```

Older `Default Browser Tray` build artifacts may still exist in `target` from before the rename.

## Important Repository State Note

After the folder rename/copy, this workspace currently does not appear to contain a `.git` directory. `git status` reports that this folder is not a Git repository.

Before making release-quality changes, decide whether to:

- reinitialize Git in this folder,
- copy the `.git` directory from the old folder if it still exists,
- or continue without Git until the project shape is finalized.

## Near-Term Next Tasks

- Replace the temporary app icon with a production Browser Switcher icon.
- Replace the temporary default browser fallback icon if needed.
- Review About window copy for distribution.
- Add a README with install, run, and manual test instructions.
- Decide on signing and distribution:
  - self-distributed installer first,
  - Microsoft Store later if desired.
- Clean old `Default Browser Tray` build artifacts from `target` when no longer needed.
- Add Git back before broader development or distribution work.

## Icon Guidance Captured

For Tauri/Windows, provide a high-resolution master icon and generate Tauri icon assets from it. A 1024x1024 source alone may not produce good 16x16 output, so small sizes should be visually checked and possibly simplified manually.

Recommended icon sizes:

```text
16x16
20x20
24x24
32x32
40x40
48x48
64x64
128x128
256x256
512x512
1024x1024 master
```

The `.ico` should include at least:

```text
16x16
32x32
48x48
256x256
```

## How To Run Locally

From PowerShell:

```powershell
cd C:\Users\thaya\LocalDocs\dev\browser-switcher
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
npm run dev
```

## How To Verify

From PowerShell:

```powershell
cd C:\Users\thaya\LocalDocs\dev\browser-switcher
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
npm run test:rust
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

## 2026-05-03 Follow-Up Session

This section captures the continuation after the chat history/workspace context became long.

### Git Repository Recreated

The renamed workspace did not have a `.git` directory, and the old `default-browser-tray` folder also did not contain one. A new Git repository was initialized in:

```text
C:\Users\thaya\LocalDocs\dev\browser-switcher
```

Git author identity was configured globally:

```text
Toshi Hayashi <thayashing@gmail.com>
```

Initial repository commits:

```text
aacbe60 Initial Browser Switcher app
034283a Hide console window in release builds
f8a72ce Apply Browser Switcher icon set
73d2a42 Fix Windows executable icon resource
b2f4801 Bump version for icon update
3ad95b3 Enforce single app instance
2c7d5ea Refresh tray icon on duplicate launch
```

`.gitignore` excludes:

```text
node_modules/
src-tauri/target/
src-tauri/gen/
```

### Release Console Window Fix

The first MSI-installed app opened a terminal-like console window. Closing that window also killed the app.

Root cause:

- The Windows release binary was built with the default console subsystem.

Fix:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

This was added at the top of `src-tauri/src/main.rs`, so release builds use the Windows GUI subsystem while debug/dev builds can still expose logs if needed.

### App Icon Work

New icon source files were added under:

```text
icon-designs/
```

The selected app icon source was:

```text
icon-designs/ver2/icons-light/
```

Applied generated assets under:

```text
src-tauri/icons/
```

Important note:

- The first generated `icon.ico` used PNG-compressed ICO entries.
- The ICO file itself looked valid, but Windows resource embedding produced a bad/default icon in `browser-switcher.exe`.
- The fix was to regenerate `src-tauri/icons/icon.ico` as a Windows-compatible uncompressed DIB/BMP multi-size ICO.

Confirmed executable icon resource contains:

```text
16x16
20x20
24x24
32x32
40x40
48x48
64x64
128x128
256x256
```

Verification used Windows icon extraction from the built executable and confirmed the new icon could be extracted from:

```text
src-tauri/target/release/browser-switcher.exe
```

### Windows UI Icon Cache Notes

Different Windows UI surfaces read icons from different places:

- Tray menu runtime icon: Tauri runtime/default window icon.
- Start menu entry: installer shortcut / Windows Installer icon table / app executable.
- Startup Apps and taskbar notification settings: Windows notification area cache and installed executable metadata.

The Start menu icon was fixed after the executable icon resource and installer version were updated.

The taskbar notification settings icon may retain stale cached entries until:

- the app process is fully quit,
- the app is updated with a higher version,
- Explorer or Windows is restarted,
- or Windows refreshes notification area cache.

### Version Bumps

The app version was bumped during this work to force Windows Installer to replace installed files and refresh shortcut/icon metadata:

```text
0.1.0 -> 0.1.1
0.1.1 -> 0.1.2
0.1.2 -> 0.1.3
```

Current version at the end of this section:

```text
0.1.3
```

Latest generated installers:

```text
src-tauri/target/release/bundle/msi/Browser Switcher_0.1.3_x64_en-US.msi
src-tauri/target/release/bundle/nsis/Browser Switcher_0.1.3_x64-setup.exe
```

### Singleton Behavior

Multiple instances could initially be launched, producing multiple tray processes/icons.

Fix:

- Added `tauri-plugin-single-instance = "2"`.
- Registered the plugin in `src-tauri/src/main.rs`.
- Duplicate launches no longer leave a new process running.

Current duplicate-launch behavior:

- Existing instance remains running.
- New process exits.
- Existing instance receives the duplicate-launch callback.
- Existing instance refreshes/re-registers the tray icon.
- About window is shown/focused.

This matters because after singleton support was added, a second launch no longer rebuilt the tray icon by creating a new process. If Windows lost or hid the tray icon, duplicate launch needed to explicitly refresh the existing tray icon.

### Tray Re-Registration Fix

After adding singleton support, Browser Switcher could show as enabled in Windows notification settings but not appear in the taskbar tray.

Hypothesis:

- Before singleton, launching again created a new process and therefore a new tray icon.
- After singleton, launching again only notified the existing process, so the tray icon was not recreated.

Fix:

- Extracted tray creation into `build_tray_icon`.
- Added `refresh_tray_icon`.
- On duplicate launch, the app removes any existing tray icon by id and rebuilds it.

Relevant behavior:

```text
duplicate launch -> refresh_tray_icon(app) -> show_about(app)
```

### Verification Completed On 2026-05-03

Repeated verification commands passed during the session:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
npm run test:rust
npm run build
```

Latest Rust test count:

```text
14 passed
```

Singleton was manually measured by starting the release executable twice and confirming the process count stayed at one:

```text
BeforeCount: 0
AfterTwoLaunchesCount: 1
```

### Current Manual Test Guidance

Before installing a new build:

1. Quit Browser Switcher from the tray menu if visible.
2. If not visible, use Task Manager or PowerShell to ensure all `browser-switcher.exe` processes are stopped.
3. Install the latest MSI or NSIS setup:

```text
Browser Switcher_0.1.3_x64_en-US.msi
Browser Switcher_0.1.3_x64-setup.exe
```

After installing:

1. Launch Browser Switcher once.
2. Confirm there is only one `browser-switcher.exe` process.
3. Confirm the tray icon appears.
4. Launch Browser Switcher again from Start menu.
5. Confirm a second process does not remain.
6. Confirm the About window appears and the tray icon remains available.

If Windows notification settings still show stale icon behavior, restart Explorer or Windows before treating it as an app bug.
