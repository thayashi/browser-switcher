# Browser Switcher

Browser Switcher is a small Windows tray utility for opening the Windows Default Apps settings page for installed browsers.

It does not change default apps directly. It only opens the supported Windows Settings flow so the user can complete the change through Windows UI.

## Features

- Runs as a lightweight Windows tray app.
- Detects installed browsers from Windows registration data.
- Shows detected browsers in a native tray menu.
- Opens browser-specific Default Apps settings when Windows supports the deep link.
- Falls back to the general Default Apps settings page.
- Prevents multiple running instances.
- Includes an About window with app version and icon.

## Requirements

- Windows 11
- WebView2 Runtime

For development:

- Node.js and npm
- Rust toolchain with Cargo
- Visual Studio Build Tools 2022 with MSVC and Windows SDK

## Development

From PowerShell:

```powershell
cd C:\Users\thaya\LocalDocs\dev\browser-switcher
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
npm install
npm run dev
```

To open the About window immediately for layout checks:

```powershell
npm run dev:about
```

## Testing

```powershell
npm run test:about
npm run test:rust
cargo check --manifest-path src-tauri\Cargo.toml
```

## Build

For a release/distribution build, bump the version first, then run:

```powershell
npm run build
```

Build outputs are written under:

```text
src-tauri/target/release/
src-tauri/target/release/bundle/msi/
src-tauri/target/release/bundle/nsis/
```

## Documentation

- Development context and historical notes: [docs/development-summary.md](docs/development-summary.md)
- Agent/developer workflow rules: [AGENTS.md](AGENTS.md)
- Historical planning artifacts are kept under `docs/superpowers/`.
