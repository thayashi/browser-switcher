# Default Browser Tray Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows 11 Tauri tray utility that lists registered browsers and opens the selected browser's Default Apps settings page.

**Architecture:** Keep registry parsing and settings URI generation in small Rust modules with unit tests. Keep Tauri-specific tray/menu behavior in `main.rs`, with browser discovery behind a Windows-only registry module and a non-Windows empty fallback for development.

**Tech Stack:** Tauri 2, Rust 2021, `windows-registry`, minimal static HTML/CSS frontend.

---

### Task 1: Project Skeleton

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`

- [x] Add minimal Tauri package metadata.
- [x] Add a static hidden-window fallback surface.
- [x] Add Tauri Rust package metadata and Windows registry dependency.

### Task 2: Core Browser Model

**Files:**
- Create: `src-tauri/src/browser.rs`
- Create: `src-tauri/src/settings.rs`

- [x] Write unit tests for browser deduplication.
- [x] Write unit tests for user and machine settings URI generation.
- [x] Implement minimal model and URI helpers.

### Task 3: Windows Registry Discovery

**Files:**
- Create: `src-tauri/src/registry.rs`

- [x] Read `RegisteredApplications` from HKCU/HKLM/WOW6432Node.
- [x] Read `Clients\StartMenuInternet` from HKCU/HKLM/WOW6432Node.
- [x] Skip inaccessible or incomplete registry entries.
- [x] Return deduplicated browser entries.

### Task 4: Tray Runtime

**Files:**
- Create: `src-tauri/src/main.rs`

- [x] Build a tray menu with detected browsers, refresh, and quit.
- [x] Rebuild the menu after refresh.
- [x] Open browser-specific `ms-settings:` URI and fall back to general Default Apps.

### Task 5: Verification

**Files:**
- Read: all created Rust and config files.

- [ ] Run `cargo test` when Rust is installed.
- [ ] Run `npm install` and `npm run tauri dev` on Windows-native tooling for tray/manual checks.
