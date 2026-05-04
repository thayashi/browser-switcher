import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const sourceHtml = readFileSync(join(root, "src-about", "index.html"), "utf8");
const distDir = join(root, "dist");

mkdirSync(distDir, { recursive: true });

writeFileSync(
  join(distDir, "index.html"),
  sourceHtml.replaceAll("__APP_VERSION__", packageJson.version),
);
copyFileSync(join(root, "src-tauri", "icons", "icon.png"), join(distDir, "icon.png"));

console.log(`Built About window assets for Browser Switcher ${packageJson.version}`);
