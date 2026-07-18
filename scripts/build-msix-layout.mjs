import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

const packageName = "ToshiSoftware.BrowserSwitcher";
const publisher = "CN=2EF452DB-B9F8-45A1-8F23-32AC995C605E";
const publisherDisplayName = "Toshi Software";
const displayName = "Browser Switcher";
const architecture = "x64";
const packageVersion = `${packageJson.version}.0`;

const sourceExe = resolve(
  root,
  process.env.BROWSER_SWITCHER_MSIX_SOURCE_EXE ??
    join("src-tauri", "target", "release", "browser-switcher.exe"),
);
const layoutDir = resolve(
  root,
  process.env.BROWSER_SWITCHER_MSIX_LAYOUT_DIR ?? join("store", "msix", "layout"),
);
const assetsDir = join(layoutDir, "Assets");
const assetSourceDir = join(root, "store", "msix", "assets");

if (!existsSync(sourceExe)) {
  throw new Error(`Store executable was not found: ${sourceExe}`);
}
if (!existsSync(assetSourceDir)) {
  throw new Error(`MSIX asset source directory was not found: ${assetSourceDir}`);
}

rmSync(layoutDir, { recursive: true, force: true });
mkdirSync(assetsDir, { recursive: true });

copyFileSync(sourceExe, join(layoutDir, "browser-switcher.exe"));
for (const entry of readdirSync(assetSourceDir, { withFileTypes: true })) {
  if (entry.isFile() && entry.name.endsWith(".png")) {
    copyFileSync(join(assetSourceDir, entry.name), join(assetsDir, entry.name));
  }
}

writeFileSync(
  join(layoutDir, "AppxManifest.xml"),
  `<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap desktop rescap">
  <Identity
    Name="${packageName}"
    Publisher="${publisher}"
    Version="${packageVersion}"
    ProcessorArchitecture="${architecture}" />
  <Properties>
    <DisplayName>${displayName}</DisplayName>
    <PublisherDisplayName>${publisherDisplayName}</PublisherDisplayName>
    <Description>${displayName}</Description>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
  <Applications>
    <Application Id="BrowserSwitcher" Executable="browser-switcher.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="${displayName}"
        Description="${displayName}"
        BackgroundColor="transparent"
        Square44x44Logo="Assets\\Square44x44Logo.png"
        Square150x150Logo="Assets\\Square150x150Logo.png">
        <uap:DefaultTile
          Square310x310Logo="Assets\\Square310x310Logo.png"
          Wide310x150Logo="Assets\\Wide310x150Logo.png" />
      </uap:VisualElements>
      <Extensions>
        <desktop:Extension Category="windows.startupTask" Executable="browser-switcher.exe" EntryPoint="Windows.FullTrustApplication">
          <desktop:StartupTask TaskId="BrowserSwitcherStartup" Enabled="false" DisplayName="${displayName}" />
        </desktop:Extension>
      </Extensions>
    </Application>
  </Applications>
</Package>
`,
);

console.log(`Built MSIX layout at ${layoutDir}`);
