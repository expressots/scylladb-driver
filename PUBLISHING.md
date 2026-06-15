# Publishing Guide

This project uses [napi-rs](https://napi.rs/) to ship a native addon split across multiple npm packages.

## Package layout

| Package | npm name | Contents |
|---------|----------|----------|
| Root loader | `scylladb-driver` | `index.js`, `index.d.ts`, TypeScript helpers |
| Linux x64 | `scylladb-driver-linux-x64-gnu` | `.node` binary |
| macOS x64 | `scylladb-driver-darwin-x64` | `.node` binary |
| Windows x64 | `scylladb-driver-win32-x64-msvc` | `.node` binary |

Users install `scylladb-driver`. npm pulls the correct platform binary via `optionalDependencies`.

## Scripts

| Command | Purpose |
|---------|---------|
| `yarn build:debug` | Dev build for your machine |
| `yarn build` | Release build for your machine |
| `yarn pack:local` | Build + prepublish + pack + verify (local machine) |
| `yarn pack:ci` | Prepublish + pack + verify (CI, after artifacts) |
| `yarn clean` | Remove build artifacts |
| `yarn test:all` | Unit + integration tests |

### What `prepublishOnly` does

On `npm publish`, npm runs:

```json
"prepublishOnly": "napi prepublish -t npm"
```

That copies `*.node` files into `npm/<platform>/` and adds `optionalDependencies` to the root `package.json`. You do not need to run it manually before publish.

**Do not commit `optionalDependencies` to git.** They are injected by `napi prepublish` during publish. If they appear after running `yarn pack:ci` locally, revert them before committing.

## Local pack check

```bash
yarn pack:local
```

Creates `scylladb-driver-<version>.tgz` and verifies it contains only publishable files (no Rust source, tests, or docker config).

## Manual publishing (local machine)

Use this when you publish yourself instead of letting CI publish on push.

### What changes vs CI auto-publish

| Topic | CI auto-publish | Manual from your machine |
|-------|----------------|------------------------|
| Trigger | Commit message like `1.0.0-preview.1` | You run publish when ready |
| npm login | `NPM_TOKEN` in GitHub secrets | `npm login` on your machine |
| Platform binaries | CI builds all 3 OS targets | **You** must provide all 3 `.node` files |
| Dist-tag | Inferred from commit message | You pass `--tag preview` explicitly |
| `optionalDependencies` | Added by `prepublishOnly` | Added by `napi prepublish` before publish |
| Git | Commit version, push, empty release commit | Commit version bump yourself; no release commit needed |

CI **build/test/pack-check** still runs on push to `main` (useful validation). The **publish** job only runs on release commits; if you never push a semver commit message, CI will not publish for you.

### One-time setup

```bash
npm login                    # or export NPM_TOKEN=...
npm whoami
```

Ensure these package names are yours on npm (first publish claims them):

- `scylladb-driver`
- `scylladb-driver-linux-x64-gnu`
- `scylladb-driver-darwin-x64`
- `scylladb-driver-win32-x64-msvc`

### Getting all three platform binaries (Linux dev machine)

Your machine can only natively build **Linux**. For macOS and Windows `.node` files, use one of:

**A. Download from CI (recommended hybrid)**

1. Push code to `main` and wait for CI **build** job to finish
2. Download artifacts from GitHub Actions (bindings for each target)
3. Place them so `yarn artifacts` can copy into `npm/*/`:

```bash
mkdir -p artifacts
# copy downloaded .node files into artifacts/ (layout from CI upload)
yarn artifacts
```

**B. Build Linux locally, skip other platforms for a test publish**

Only viable if you accept that macOS/Windows users cannot install yet. Not recommended for real preview.

### First preview publish: `1.0.0-preview.1`

Version is already set to `1.0.0-preview.1` in `package.json`, `npm/*/package.json`, and `Cargo.toml`.

```bash
# 1. Test
yarn docker:up
yarn test:all

# 2. Generate JS loader + Linux binary
yarn build

# 3. Ensure all platform binaries are in npm/*/
#    (see "Getting all three platform binaries" above)
yarn artifacts    # if you downloaded CI artifacts

# 4. Dry-run pack (optional sanity check)
yarn pack:ci

# 5. Publish all 4 packages with preview tag
yarn publish:preview

# 6. Revert generated package.json fields before git commit
git checkout package.json

# 7. Smoke test
mkdir /tmp/scylla-smoke && cd /tmp/scylla-smoke
npm init -y
npm install scylladb-driver@preview
node -e "const { Cluster } = require('scylladb-driver'); console.log(typeof Cluster)"
```

### Subsequent preview updates (`1.0.0-preview.2`, etc.)

```bash
# 1. Bump version everywhere
npm version 1.0.0-preview.2 --no-git-tag-version
napi version

# 2. Commit and push code (CI builds new artifacts)
git add package.json npm/*/package.json Cargo.toml Cargo.lock
git commit -m "chore: bump to 1.0.0-preview.2"
git push origin main
# Wait for CI build job, then download artifacts

# 3. Build JS bindings + merge platform binaries
yarn build
yarn artifacts

# 4. Verify and publish
yarn pack:ci
yarn publish:preview
git checkout package.json
```

### Manual publish command reference

```bash
yarn publish:preview          # preview dist-tag (npm install scylladb-driver@preview)
yarn publish:preview -- --latest   # stable dist-tag (when ready for 1.0.0)
```

Under the hood this runs `napi prepublish`, publishes each `npm/*/` package, then the root package. Order matters: platform packages first, then root.

## CI auto-publish (optional)

If you prefer CI to publish later, push a commit whose message is exactly the version:

```bash
git commit --allow-empty -m "1.0.0-preview.1"
git push origin main
```

## Release tags and npm dist-tags

CI reads the **latest commit message** to decide whether to publish:

| Commit message example | npm dist-tag | When to use |
|------------------------|--------------|-------------|
| `1.0.0` | `latest` | Stable release |
| `1.0.0-preview.1` | `preview` | Preview releases |
| `1.0.0-beta.1` | `next` | Other pre-releases |

Install preview builds with:

```bash
npm install scylladb-driver@preview
```

## Launch checklist: `1.0.0-preview.1`

### Before you publish

- [ ] All tests pass locally: `yarn test:all` (requires `yarn docker:up`)
- [ ] Pack check passes: `yarn pack:local`
- [ ] Version is `1.0.0-preview.1` in:
  - `package.json`
  - `npm/*/package.json`
  - `Cargo.toml`
- [ ] `NPM_TOKEN` secret is set in GitHub repo settings (Automation or Publish token)
- [ ] npm org/user `@expressots` or package name `scylladb-driver` is available on npm (or you own it)
- [ ] README install instructions mention `@preview` tag

### Option A: Publish via CI (recommended)

1. Commit and push all release-prep changes to `main`
2. Create an empty commit with the version as the message:

```bash
git commit --allow-empty -m "1.0.0-preview.1"
git push origin main
```

3. CI runs build, test, pack-check, then publish with `--tag preview`
4. Verify on npm:

```bash
npm view scylladb-driver versions
npm view scylladb-driver dist-tags
```

### Option B: Publish manually from your machine

Requires built binaries for all three platforms (usually only feasible after CI artifacts download).

```bash
# 1. Set version (syncs platform packages via napi)
npm version 1.0.0-preview.1 --no-git-tag-version

# 2. Build for your platform
yarn build

# 3. Place all platform .node files under npm/*/ (from CI artifacts if needed)
yarn artifacts   # if artifacts/ directory exists from CI download

# 4. Prepublish and verify
yarn pack:ci

# 5. Publish platform packages first, then root
for dir in npm/*; do
  (cd "$dir" && npm publish --access public --tag preview)
done
npm publish --access public --tag preview
```

### After publish

- [ ] Smoke test install:

```bash
mkdir /tmp/scylla-driver-smoke && cd /tmp/scylla-driver-smoke
npm init -y
npm install scylladb-driver@preview
node -e "const { Cluster } = require('scylladb-driver'); console.log(typeof Cluster)"
```

- [ ] Create GitHub release tagged `v1.0.0-preview.1` with changelog notes
- [ ] Announce preview: breaking changes may still occur before `1.0.0`

### Bumping preview versions

For `1.0.0-preview.2`, `1.0.0-preview.3`, etc.:

```bash
# Update version in package.json, npm/*, Cargo.toml
npm version 1.0.0-preview.2 --no-git-tag-version
napi version   # sync platform package versions

git add -A
git commit -m "1.0.0-preview.2"
git push origin main
# CI publishes with --tag preview
```

### Graduating to stable `1.0.0`

When ready for stable:

1. Set version to `1.0.0`
2. Push commit with message exactly `1.0.0`
3. CI publishes with dist-tag `latest`
4. Users can then `npm install scylladb-driver` without `@preview`

## Troubleshooting

**`prepublishOnly` warns that npm/*/ `.node` files don't exist**

Build artifacts are missing. Run `yarn build` locally or download CI artifacts and run `yarn artifacts`.

**Installed package can't find native binding**

The platform optional dependency failed to install. Check:

```bash
npm ls scylladb-driver-linux-x64-gnu
```

**Publish fails with 403**

Token lacks publish permission or package name is taken by another user.

**CI skips publish**

Latest commit message must match a semver pattern. Use exactly `1.0.0-preview.1` as the commit message.
