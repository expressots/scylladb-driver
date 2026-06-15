/**
 * Prepare root package.json for npm pack without publishing platform packages.
 * CI and local pack checks use this instead of `napi prepublish -t npm`, which
 * attempts npm publish and GitHub release creation.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const rootPkgPath = join(rootDir, "package.json");
const rootPkg = JSON.parse(readFileSync(rootPkgPath, "utf8"));
const npmDir = join(rootDir, "npm");

const optionalDependencies = {};

for (const dir of ["linux-x64-gnu", "darwin-x64", "win32-x64-msvc"]) {
  const pkgPath = join(npmDir, dir, "package.json");
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  optionalDependencies[pkg.name] = pkg.version;
}

rootPkg.optionalDependencies = optionalDependencies;
writeFileSync(rootPkgPath, `${JSON.stringify(rootPkg, null, 2)}\n`);
console.log("Updated optionalDependencies for pack:", optionalDependencies);
