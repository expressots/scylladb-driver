# Publish preview packages to npm (manual workflow).
#
# Prerequisites:
#   - npm login (or NPM_TOKEN in env)
#   - All three platform .node files under npm/*/ (see PUBLISHING.md)
#   - index.js / index.d.ts generated (yarn build)
#
# Usage:
#   yarn publish:preview              # publish with --tag preview
#   yarn publish:preview -- --latest  # publish stable (no tag)

set -euo pipefail

TAG="preview"
if [ "${1:-}" = "--latest" ]; then
  TAG=""
fi

if ! npm whoami >/dev/null 2>&1; then
  echo "Not logged in to npm. Run: npm login"
  exit 1
fi

for node_file in \
  npm/linux-x64-gnu/scylladb-driver.linux-x64-gnu.node \
  npm/darwin-x64/scylladb-driver.darwin-x64.node \
  npm/win32-x64-msvc/scylladb-driver.win32-x64-msvc.node
do
  if [ ! -f "$node_file" ]; then
    echo "Missing platform binary: $node_file"
    echo "Build all platforms or download CI artifacts and run: yarn artifacts"
    exit 1
  fi
done

if [ ! -f index.js ] || [ ! -f index.d.ts ]; then
  echo "Missing index.js / index.d.ts. Run: yarn build"
  exit 1
fi

echo "Running napi prepublish..."
napi prepublish -t npm

publish_pkg() {
  local dir="$1"
  echo "Publishing $dir ..."
  if [ -n "$TAG" ]; then
    (cd "$dir" && npm publish --access public --tag "$TAG")
  else
    (cd "$dir" && npm publish --access public)
  fi
}

publish_pkg npm/linux-x64-gnu
publish_pkg npm/darwin-x64
publish_pkg npm/win32-x64-msvc

echo "Publishing root scylladb-driver ..."
if [ -n "$TAG" ]; then
  npm publish --access public --tag "$TAG"
else
  npm publish --access public
fi

echo ""
echo "Published successfully."
if [ -n "$TAG" ]; then
  echo "Install: npm install scylladb-driver@$TAG"
else
  echo "Install: npm install scylladb-driver"
fi
echo ""
echo "Note: napi prepublish may have added optionalDependencies to package.json."
echo "Revert before committing: git checkout package.json"
