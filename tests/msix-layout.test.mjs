import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { inflateSync } from "node:zlib";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const tempDir = mkdtempSync(join(tmpdir(), "browser-switcher-msix-"));
const packageVersion = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version;

function readPngRgbaPixel(filePath, x, y) {
  const { rows } = readPng(filePath);
  const pixelOffset = x * 4;
  const row = rows[y];
  return Array.from(row.subarray(pixelOffset, pixelOffset + 4));
}

function readPng(filePath) {
  const png = readFileSync(filePath);
  assert.equal(png.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");

  let offset = 8;
  let width = 0;
  let height = 0;
  const idatChunks = [];
  while (offset < png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.subarray(offset + 4, offset + 8).toString("ascii");
    const data = png.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      assert.equal(data[8], 8);
      assert.equal(data[9], 6);
    } else if (type === "IDAT") {
      idatChunks.push(data);
    } else if (type === "IEND") {
      break;
    }
    offset += length + 12;
  }

  const inflated = inflateSync(Buffer.concat(idatChunks));
  const bytesPerPixel = 4;
  const stride = width * bytesPerPixel;
  const rows = [];
  let inputOffset = 0;
  let previous = Buffer.alloc(stride);

  for (let rowIndex = 0; rowIndex < height; rowIndex += 1) {
    const filter = inflated[inputOffset];
    inputOffset += 1;
    const scanline = Buffer.from(inflated.subarray(inputOffset, inputOffset + stride));
    inputOffset += stride;
    const row = Buffer.alloc(stride);

    for (let i = 0; i < stride; i += 1) {
      const left = i >= bytesPerPixel ? row[i - bytesPerPixel] : 0;
      const up = previous[i];
      const upLeft = i >= bytesPerPixel ? previous[i - bytesPerPixel] : 0;
      const raw = scanline[i];
      if (filter === 0) row[i] = raw;
      else if (filter === 1) row[i] = (raw + left) & 0xff;
      else if (filter === 2) row[i] = (raw + up) & 0xff;
      else if (filter === 3) row[i] = (raw + Math.floor((left + up) / 2)) & 0xff;
      else if (filter === 4) {
        const p = left + up - upLeft;
        const pa = Math.abs(p - left);
        const pb = Math.abs(p - up);
        const pc = Math.abs(p - upLeft);
        const predictor = pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft;
        row[i] = (raw + predictor) & 0xff;
      } else {
        throw new Error(`Unsupported PNG filter: ${filter}`);
      }
    }
    rows.push(row);
    previous = row;
  }

  return { width, height, rows };
}

try {
  const fakeExe = join(tempDir, "browser-switcher.exe");
  const outputDir = join(tempDir, "layout");
  writeFileSync(fakeExe, "fake exe for layout contract test");

  execFileSync(process.execPath, [join(root, "scripts", "build-msix-layout.mjs")], {
    cwd: root,
    env: {
      ...process.env,
      BROWSER_SWITCHER_MSIX_SOURCE_EXE: fakeExe,
      BROWSER_SWITCHER_MSIX_LAYOUT_DIR: outputDir,
    },
    stdio: "pipe",
  });

  const manifest = readFileSync(join(outputDir, "AppxManifest.xml"), "utf8");
  assert.match(manifest, /Name="ToshiSoftware\.BrowserSwitcher"/);
  assert.match(
    manifest,
    /Publisher="CN=2EF452DB-B9F8-45A1-8F23-32AC995C605E"/,
  );
  assert.ok(manifest.includes(`Version="${packageVersion}.0"`));
  assert.match(manifest, /Executable="browser-switcher\.exe"/);
  assert.match(manifest, /Category="windows\.startupTask"/);
  assert.match(manifest, /TaskId="BrowserSwitcherStartup" Enabled="false"/);
  assert.match(manifest, /BackgroundColor="transparent"/);
  assert.match(manifest, /<Logo>Assets\\StoreLogo\.png<\/Logo>/);
  assert.doesNotMatch(manifest, /VFS\\/);

  assert.equal(existsSync(join(outputDir, "browser-switcher.exe")), true);
  assert.equal(existsSync(join(outputDir, "Assets", "StoreLogo.png")), true);
  assert.equal(existsSync(join(outputDir, "Assets", "StoreLogo.scale-200.png")), true);
  assert.equal(existsSync(join(outputDir, "Assets", "Square150x150Logo.scale-200.png")), true);
  assert.equal(
    existsSync(join(outputDir, "Assets", "StoreLogo.targetsize-256_altform-unplated.png")),
    true,
  );
  assert.equal(
    existsSync(join(outputDir, "Assets", "Square44x44Logo.targetsize-256_altform-unplated.png")),
    true,
  );
  const storeLogo = readPng(join(outputDir, "Assets", "StoreLogo.png"));
  assert.deepEqual([storeLogo.width, storeLogo.height], [256, 256]);
  assert.equal(readPngRgbaPixel(join(outputDir, "Assets", "StoreLogo.png"), 0, 0)[3], 255);
  assert.deepEqual(
    readPngRgbaPixel(
      join(outputDir, "Assets", "Square44x44Logo.targetsize-256_altform-unplated.png"),
      0,
      0,
    ),
    [
    0,
    0,
    0,
    0,
    ],
  );

  const topLevelEntries = readdirSync(outputDir, { withFileTypes: true }).map((entry) => entry.name);
  assert.deepEqual(
    topLevelEntries.sort(),
    ["AppxManifest.xml", "Assets", "browser-switcher.exe"].sort(),
  );
  assert.equal(topLevelEntries.includes("VFS"), false);
  assert.equal(topLevelEntries.includes("EBWebView"), false);

  console.log("msix layout contract passed");
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
