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
