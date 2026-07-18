# Store MSIX Packaging

Browser Switcher uses a direct `makeappx.exe` packaging path for Microsoft Store MSIX builds.

Do not use MSIX Packaging Tool installer capture for Store packages. Capturing the Tauri MSI/EXE installer can include per-user runtime data such as `EBWebView` cache files under `VFS/Local AppData`, especially if the app launches during capture.

Build the unsigned Store MSIX:

```powershell
npm run build:msix
```

The package is written to:

```text
store/msix/out/ToshiSoftware.BrowserSwitcher_<version>.0_x64.msix
```

Before submitting to Partner Center, inspect the package contents:

```powershell
tar -tf store\msix\out\ToshiSoftware.BrowserSwitcher_<version>.0_x64.msix
```

Expected top-level payload:

```text
Assets/
browser-switcher.exe
AppxManifest.xml
AppxBlockMap.xml
[Content_Types].xml
```

The package must not contain:

```text
VFS/Local AppData/
EBWebView/
```

MSIX visual assets are stored in:

```text
store/msix/assets/
```

The Store package uses separate assets from the normal Tauri executable icon. `StoreLogo.png`
and scale-qualified tile logos include a square background. `Square44x44Logo.targetsize-*_altform-unplated.png`
files use a transparent background for Windows surfaces that request unplated target-size icons.

Do not bake rounded corners into these PNGs. Microsoft Store and Windows shell surfaces apply
their own masks and rounding where needed.
