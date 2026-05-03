import { readFileSync } from "node:fs";
import assert from "node:assert/strict";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");

assert.match(html, /getCurrentWindow\(\)\.hide\(\)/);
assert.doesNotMatch(html, /window\.close\(\)/);

assert.match(html, /<img[^>]+src="\.\/icon\.png"/);
assert.match(html, /Version __APP_VERSION__/);
assert.doesNotMatch(html, /Version 0\.1\.0/);

console.log("about HTML contract passed");
