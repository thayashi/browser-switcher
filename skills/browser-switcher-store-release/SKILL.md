---
name: browser-switcher-store-release
description: Build, verify, and submit Browser Switcher Microsoft Store releases. Use when preparing a versioned Windows MSIX update, updating Partner Center package/listings, confirming Store certification status, or investigating a release-blocking UI regression in this Tauri tray app.
---

# Browser Switcher Store Release

## Guardrails

- Treat every tray-menu callback as latency-sensitive UI work. Never synchronously wait for Windows async APIs, network I/O, process execution, or blocking registry work on Tauri's main/UI thread.
- For Windows Runtime APIs that return async operations, execute the wait on a dedicated worker thread with the appropriate COM apartment, then return visual/menu updates with `app.run_on_main_thread`.
- Keep menu callbacks short: collect the menu ID, start background work, and return. Make failure paths non-blocking and recoverable (for example, opening Windows Settings).
- Re-test the exact user interaction that triggered a fix in a release build before submission. Verify the application still accepts mouse input and the tray menu can be reopened after toggling the option.
- Before changing tray menu structure, preserve the required order: `Launch`, `About`, `Quit`; keep optional settings above this group.

## Prepare the release

1. Work on a feature branch; do not commit directly to `main`.
2. Inspect the worktree before editing and preserve unrelated changes.
3. Bump and align the version in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
4. Run `npm run build:about` after every version change.
5. Update `docs/development-summary.md` for behavior, release, test, or Windows/Tauri assumptions that future maintainers need.

## Verify and package

Run the smallest relevant checks; for a Store update run:

```powershell
npm run test:about
npm run test:rust
cargo check --manifest-path src-tauri\Cargo.toml
npm run build
npm run build:msix
npm run test:msix
```

Before installer or manual interaction testing, fully quit any existing `browser-switcher.exe`. Confirm the generated MSIX version and package identity match Partner Center. Inspect the archive when packaging logic changes; it must contain the app payload and no captured WebView/user-data directories.

## Publish code and create the Store submission

1. Commit intentionally, then merge/push the approved release commit to `main`; the GitHub release workflow runs from `main` and only creates assets for a new version tag.
2. In Partner Center, start an update and upload `store\msix\out\ToshiSoftware.BrowserSwitcher_<version>.0_x64.msix`.
3. Confirm the package is **Validated**. If Partner Center schedules an older package for removal because the new package serves the same customers, review the versions and save only when the new package is the intended replacement.
4. Update Store listings for both English (United States) and Japanese. State user-visible fixes precisely and avoid claims that have not been tested.
5. Confirm **Packages: Updated/Validated**, **Store listings: Updated**, and **Age ratings: Complete**.

Use the Computer Use skill for Partner Center and the signed-in Edge session. Re-observe the page before each UI input. Ask the user for confirmation immediately before the irreversible **Submit for certification** action, unless they explicitly direct that action in the current turn.

## Hand-off

After submission, report the exact certification stage. Do not say the update is live until Partner Center shows publishing is complete. If automatic publishing is configured, state that it will publish after certification passes.
