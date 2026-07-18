import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

const layoutDir = resolve(root, process.env.BROWSER_SWITCHER_MSIX_LAYOUT_DIR ?? join("store", "msix", "layout"));
const outputDir = resolve(root, process.env.BROWSER_SWITCHER_MSIX_OUTPUT_DIR ?? join("store", "msix", "out"));
const outputPath = join(outputDir, `ToshiSoftware.BrowserSwitcher_${packageJson.version}.0_x64.msix`);

function findMakeAppx() {
  const kitsRoot = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
  if (!existsSync(kitsRoot)) {
    return null;
  }

  const versions = readdirSync(kitsRoot)
    .filter((entry) => /^\d+\.\d+\.\d+\.\d+$/.test(entry))
    .sort()
    .reverse();

  for (const version of versions) {
    const candidate = join(kitsRoot, version, "x64", "makeappx.exe");
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

const makeAppx = process.env.MAKEAPPX_EXE ?? findMakeAppx();
if (!makeAppx) {
  throw new Error("makeappx.exe was not found. Install the Windows SDK or set MAKEAPPX_EXE.");
}

mkdirSync(outputDir, { recursive: true });
execFileSync(makeAppx, ["pack", "/d", layoutDir, "/p", outputPath, "/o"], {
  cwd: root,
  stdio: "inherit",
});

console.log(`Built unsigned MSIX at ${outputPath}`);
