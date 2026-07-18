# MSIX Store Publishing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish Browser Switcher to the Microsoft Store without paying for a third-party code signing certificate by producing an MSIX package for Store submission.

**Architecture:** Keep the existing Tauri app and GitHub Release MSI/EXE workflow intact, then add a separate Store packaging path that wraps the Tauri release executable directly into MSIX with `makeappx.exe`. Tauri 2 currently generates MSI/EXE installers, so MSIX packaging must be handled by Windows tooling outside the normal Tauri bundler.

**Tech Stack:** Tauri 2, Rust, Node.js, GitHub Actions on Windows, Microsoft Store / Partner Center, MSIX Packaging Tool or Visual Studio Windows Application Packaging Project.

---

## Decision

Use the Microsoft Store MSIX route, not the Store MSI/EXE route.

Reasons:
- Microsoft Store MSIX publishing handles signing after certification, avoiding a paid CA code signing certificate.
- Microsoft's MSI/EXE Store route still requires the installer and PE files to be signed before submission.
- Tauri's current Store guidance says Tauri only generates EXE/MSI installers, so native MSIX output should not be assumed.

References:
- Microsoft Store MSIX signing: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options
- First Store app publishing flow: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/publish-first-app
- Tauri Microsoft Store limitation: https://v2.tauri.app/distribute/microsoft-store/
- MSIX desktop packaging: https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-packaging-dot-net

## File Structure

- Modify: `package.json`
  - Add Store/MSIX helper scripts once the chosen packaging tool is known.
- Modify: `src-tauri/tauri.conf.json`
  - Replace the local placeholder Tauri identifier with a stable reverse-DNS identity.
  - Add a publisher value that does not conflict with the product name.
- Create: `src-tauri/tauri.store.conf.json`
  - Store-specific Tauri config used only for the release executable build.
- Create: `store/msix/`
  - Store packaging documentation. Generated layout and output under this directory are ignored.
- Modify: `.github/workflows/release.yml`
  - Keep existing MSI/EXE release behavior.
  - Add a separate manual Store package job or workflow after local MSIX packaging is proven.
- Modify: `docs/development-summary.md`
  - Record the Store/MSIX publishing decision, signing assumptions, and release caveats.
- Test: manual install and Windows App Certification Kit validation.

## Task 1: Confirm Store Account And App Identity

- [x] **Step 1: Create or confirm the Partner Center developer account**

Use https://storedeveloper.microsoft.com and choose the free account path that matches the publisher identity.

Expected:
- A Partner Center account exists.
- The publisher display name is known.
- The app name `Browser Switcher` is reserved or an alternate Store name is chosen.

- [x] **Step 2: Record identity values**

Create a private note outside the repo with:

```text
Store app name: Browser Switcher
Publisher display name: Toshi Software
Package identity name: ToshiSoftware.BrowserSwitcher
Publisher ID / CN: CN=2EF452DB-B9F8-45A1-8F23-32AC995C605E
Package Family Name: ToshiSoftware.BrowserSwitcher_ngewhyn65gmh0
Store ID: 9P0TQCXR13D9
```

Do not commit private Partner Center IDs if Microsoft marks them sensitive.

- [x] **Step 3: Decide the Store package name**

Use Partner Center's package identity as the source of truth. Do not invent a package identity in code if Partner Center provides one.

Expected:
- `src-tauri/tauri.conf.json` can use a stable Tauri identifier.
- The later MSIX manifest can use the exact Partner Center identity values.

## Task 2: Add Store-Specific Tauri Configuration

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/tauri.store.conf.json`
- Modify: `package.json`

- [x] **Step 1: Update the Tauri identifier and publisher**

After Task 1 has produced Partner Center identity values, modify `src-tauri/tauri.conf.json` with a stable Tauri-safe identifier and publisher:

```json
{
  "identifier": "com.toshisoftware.browser-switcher",
  "bundle": {
    "publisher": "Toshi Software"
  }
}
```

Keep the exact Partner Center `Package/Identity/Name` for the MSIX manifest, not the Tauri identifier. Keep the existing `productName`, `version`, icons, and window settings unchanged.

- [x] **Step 2: Add a Store-specific Tauri config**

Create `src-tauri/tauri.store.conf.json`:

```json
{
  "bundle": {
    "active": false
  }
}
```

Purpose:
- Store packaging starts from the compiled Tauri executable.
- Tauri's MSI/EXE bundler is not the MSIX packaging source of truth.

- [x] **Step 3: Add a no-bundle Store build script**

Modify `package.json` scripts:

```json
{
  "build:store-exe": "npm run build:about && tauri build --no-bundle --config src-tauri/tauri.store.conf.json"
}
```

- [x] **Step 4: Verify the Store executable build**

Run:

```powershell
npm run build:store-exe
```

Expected:
- The command exits successfully.
- `src-tauri\target\release\browser-switcher.exe` exists.
- Existing `npm run build` still creates MSI/EXE for GitHub Releases.

## Task 3: Prove Direct makeappx MSIX Packaging

**Files:**
- Create: `store/msix/README.md`
- Create: `scripts/build-msix-layout.mjs`
- Create: `scripts/build-msix-package.mjs`
- Create: `tests/msix-layout.test.mjs`
- Modify: `package.json`
- Modify: `.gitignore`

- [x] **Step 1: Add a layout contract test**

Create `tests/msix-layout.test.mjs` to verify:
- `AppxManifest.xml` contains Partner Center identity values.
- The executable path is `browser-switcher.exe`, not a `VFS` path.
- Top-level layout contains only `AppxManifest.xml`, `Assets`, and `browser-switcher.exe`.
- Runtime data directories such as `VFS` and `EBWebView` are absent.

Expected:
- The test fails before `scripts/build-msix-layout.mjs` exists.

- [x] **Step 2: Build a minimal MSIX layout**

Create `scripts/build-msix-layout.mjs` to:
- Copy `src-tauri\target\release\browser-switcher.exe` into `store/msix/layout/browser-switcher.exe`.
- Write `store/msix/layout/AppxManifest.xml`.
- Copy app icon assets into `store/msix/layout/Assets`.
- Remove any previous generated layout before writing the new one.

Expected:
- No installer capture is used.
- No per-user `Local AppData`, `EBWebView`, or cache files are included.

- [x] **Step 3: Add makeappx packaging**

Create `scripts/build-msix-package.mjs` to locate Windows SDK `makeappx.exe` and pack `store/msix/layout` into:

```text
store/msix/out/ToshiSoftware.BrowserSwitcher_<version>.0_x64.msix
```

Expected:
- Package creation succeeds without a signing certificate.
- Store signing remains handled by Microsoft after Partner Center certification.

- [x] **Step 4: Document the packaging command**

Create `store/msix/README.md`:

````markdown
# Store MSIX Packaging

Browser Switcher uses a direct `makeappx.exe` packaging path for Microsoft Store MSIX builds.

Do not use MSIX Packaging Tool installer capture for Store packages. Capturing the Tauri MSI/EXE installer can include per-user runtime data such as `EBWebView` cache files under `VFS/Local AppData`, especially if the app launches during capture.

Build the unsigned Store MSIX:

```powershell
npm run build:msix
```

Inspect the package contents:

```powershell
tar -tf store\msix\out\ToshiSoftware.BrowserSwitcher_<version>.0_x64.msix
```

The package must not contain:

```text
VFS/Local AppData/
EBWebView/
```
````

- [x] **Step 5: Prove generated package contents**

Run:

```powershell
npm run build:msix
tar -tf store\msix\out\ToshiSoftware.BrowserSwitcher_0.1.7.0_x64.msix
```

Expected:
- The generated package contains only app assets, `browser-switcher.exe`, manifest, block map, and content types.
- The generated package does not contain `VFS/Local AppData` or `EBWebView`.

## Task 4: Add Store Listing Assets

**Files:**
- Create: `store/listing/`
- Create: `store/listing/description.md`
- Create: `store/listing/screenshots/`
- Create: `store/listing/assets/`

- [ ] **Step 1: Draft the Store description**

Create `store/listing/description.md`:

```markdown
# Browser Switcher

Browser Switcher is a small Windows tray utility for quickly switching the system default browser.

## Short Description

Switch your default browser from the Windows tray.

## Description

Browser Switcher helps Windows users change their default browser from a simple tray menu. It is designed for people who regularly switch between browsers for work, testing, privacy, or personal browsing.

## Keywords

browser, default browser, tray, Windows utility

## Category

Utilities & tools
```

- [ ] **Step 2: Capture screenshots**

Run:

```powershell
npm run dev:about
```

Capture Store-ready screenshots showing:
- Tray menu with available browsers.
- About window.
- Default browser indicator.

Expected:
- Screenshots are saved under `store/listing/screenshots/`.
- Images meet Partner Center dimensions shown during submission.

- [ ] **Step 3: Prepare Store logo assets**

Use the existing app icon as the source, then produce the Store sizes required by Partner Center.

Expected:
- Store logo files are saved under `store/listing/assets/`.
- The icon is legible at small sizes.

## Task 5: Validate The MSIX Locally

**Files:**
- Modify: `docs/development-summary.md`

- [ ] **Step 1: Install the MSIX locally**

Run the generated package through local install.

Expected:
- Browser Switcher launches.
- Tray icon appears.
- About window opens and hides correctly.
- The OK button still uses Tauri window hide behavior.

- [ ] **Step 2: Run existing checks**

Run:

```powershell
npm run test:about
npm run test:rust
cargo check --manifest-path src-tauri\Cargo.toml
npm run build
```

Expected:
- All commands pass.
- Existing GitHub Release MSI/EXE behavior is not broken.

- [ ] **Step 3: Run Windows App Certification Kit**

Run the Windows App Certification Kit against the generated MSIX package.

Expected:
- Certification checks pass.
- Any failure is documented with exact remediation in this plan before continuing.

- [ ] **Step 4: Update development summary**

Add a dated entry to `docs/development-summary.md`:

```markdown
## 2026-06-02 - Microsoft Store MSIX publishing path

- Store distribution uses MSIX to avoid third-party code signing certificate cost; Microsoft signs the package after certification.
- Tauri 2 does not currently produce MSIX directly, so Store packaging wraps the Tauri release executable with Windows MSIX tooling.
- Existing MSI/EXE GitHub Release workflow remains for non-Store distribution unless explicitly retired.
- Before Store submission, run About tests, Rust tests, cargo check, normal build, MSIX local install, and Windows App Certification Kit.
```

## Task 6: Add Optional CI For Store Packages

**Files:**
- Create: `.github/workflows/store-msix.yml`
- Or modify: `.github/workflows/release.yml`

- [ ] **Step 1: Keep Store packaging manual at first**

Create `.github/workflows/store-msix.yml` with `workflow_dispatch` only.

Expected:
- Store packages are not produced on every `main` merge until the packaging path is stable.
- GitHub Release publishing remains unchanged.

- [ ] **Step 2: Build the Store executable in CI**

Add steps equivalent to:

```yaml
- name: Set up Node.js
  uses: actions/setup-node@v4
  with:
    node-version: 22
    cache: npm

- name: Set up Rust
  uses: dtolnay/rust-toolchain@stable

- name: Install dependencies
  run: npm ci

- name: Test About window assets
  run: npm run test:about

- name: Run Rust tests
  run: npm run test:rust

- name: Build Store executable
  run: npm run build:store-exe
```

- [ ] **Step 3: Add packaging only after local packaging is repeatable**

Add the exact packaging command from Task 3.

Expected:
- CI emits `.msixupload` or `.msixbundle` as a workflow artifact.
- No signing secret is required for Store submission.

## Task 7: Submit To Partner Center

- [ ] **Step 1: Create the first submission**

In Partner Center:
- Select the reserved app.
- Upload the MSIX package.
- Add listing text from `store/listing/description.md`.
- Upload screenshots and logo assets.
- Complete age rating.
- Set pricing to free unless a paid app model is intentionally chosen.

- [ ] **Step 2: Submit for certification**

Expected:
- Partner Center accepts the MSIX.
- Certification starts.

- [ ] **Step 3: Record certification feedback**

If Microsoft rejects the submission, add the exact failure reason to `docs/development-summary.md` and create a focused fix branch.

## Self-Review

- Spec coverage: The plan covers the zero-cost requirement by choosing Store MSIX, not MSI/EXE.
- Known risk: Tauri does not natively generate MSIX, so the packaging proof is the highest-risk task.
- Placeholder scan: Store identity values must come from Partner Center; commands and file paths are concrete enough to start implementation and must be tightened to exact emitted package names during Task 3.
- Type consistency: File names and commands match the current repository shape.
