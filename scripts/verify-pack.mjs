import { execSync } from "node:child_process";
import { existsSync } from "node:fs";

const REQUIRED = [
  "package/package.json",
  "package/index.js",
  "package/index.d.ts",
  "package/README.md",
  "package/LICENSE",
  "package/ts/index.ts",
];

const FORBIDDEN = [
  "package/src/",
  "package/__test__/",
  "package/docker/",
  "package/driver_api/",
  "package/Cargo.toml",
];

const tgz = execSync("ls -1 scylladb-driver-*.tgz | head -1", {
  encoding: "utf8",
}).trim();

if (!tgz || !existsSync(tgz)) {
  console.error("No scylladb-driver-*.tgz found. Run yarn pack:local or yarn pack:ci first.");
  process.exit(1);
}

const listing = execSync(`tar -tzf ${tgz}`, { encoding: "utf8" })
  .split("\n")
  .filter(Boolean);

console.log(`Verifying ${tgz} (${listing.length} entries)`);

for (const path of REQUIRED) {
  if (!listing.includes(path)) {
    console.error(`Missing required file in tarball: ${path}`);
    process.exit(1);
  }
}

for (const prefix of FORBIDDEN) {
  if (listing.some((entry) => entry.startsWith(prefix))) {
    console.error(`Forbidden path in tarball: ${prefix}`);
    process.exit(1);
  }
}

console.log("Pack verification passed.");
console.log(`  tarball: ${tgz}`);
console.log(`  files: ${REQUIRED.length} required entries present`);
console.log(`  no dev-only paths leaked`);
