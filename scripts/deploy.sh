#!/bin/bash
# Deploy Shrimp web app to tedbauer.github.io/shrimp/
# Usage: ./scripts/deploy.sh

set -euo pipefail

EMULATOR_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOY_DIR="$EMULATOR_DIR/../deploying/tedbauer.github.io/shrimp"

echo "🦐 Deploying Shrimp..."

# 1. Build compiler WASM
echo "  Building compiler WASM..."
cd "$EMULATOR_DIR"
wasm-pack build compiler --target web --out-dir ../web/compiler_pkg -- --features wasm

# 2. Build emulator WASM
echo "  Building emulator WASM..."
wasm-pack build --target web --out-dir web/pkg

# 3. Copy web files
echo "  Copying web files to $DEPLOY_DIR ..."
rm -rf "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR"
cp web/index.html "$DEPLOY_DIR/"
cp web/index.js   "$DEPLOY_DIR/"

# Copy WASM packages (without .gitignore files that wasm-pack generates)
mkdir -p "$DEPLOY_DIR/compiler_pkg"
cp web/compiler_pkg/compiler.js         "$DEPLOY_DIR/compiler_pkg/"
cp web/compiler_pkg/compiler_bg.wasm    "$DEPLOY_DIR/compiler_pkg/"
cp web/compiler_pkg/compiler.d.ts       "$DEPLOY_DIR/compiler_pkg/" 2>/dev/null || true
cp web/compiler_pkg/compiler_bg.wasm.d.ts "$DEPLOY_DIR/compiler_pkg/" 2>/dev/null || true

mkdir -p "$DEPLOY_DIR/pkg"
cp web/pkg/emulator.js       "$DEPLOY_DIR/pkg/"
cp web/pkg/emulator_bg.wasm  "$DEPLOY_DIR/pkg/"
cp web/pkg/emulator.d.ts     "$DEPLOY_DIR/pkg/" 2>/dev/null || true
cp web/pkg/emulator_bg.wasm.d.ts "$DEPLOY_DIR/pkg/" 2>/dev/null || true

# Copy BIOS if present
if [ -f web/bios/bios.rom ]; then
    mkdir -p "$DEPLOY_DIR/bios"
    cp web/bios/bios.rom "$DEPLOY_DIR/bios/"
fi

# 4. Generate documentation with Pushpin
echo "  Generating docs..."
DOCS_DIR="$EMULATOR_DIR/docs-site"
if command -v pushpin &>/dev/null; then
    PUSHPIN_BIN=pushpin
elif [ -f "$EMULATOR_DIR/../pushpin/target/release/pushpin" ]; then
    PUSHPIN_BIN="$EMULATOR_DIR/../pushpin/target/release/pushpin"
else
    echo "  ⚠ pushpin not found, skipping docs generation"
    PUSHPIN_BIN=""
fi

if [ -n "$PUSHPIN_BIN" ]; then
    cd "$DOCS_DIR"
    rm -rf docs/
    "$PUSHPIN_BIN" generate
    mkdir -p "$DEPLOY_DIR/docs"
    cp -r docs/* "$DEPLOY_DIR/docs/"
    cd "$EMULATOR_DIR"
fi

echo ""
echo "✅ Done! Files copied to:"
echo "   $DEPLOY_DIR"
echo ""
echo "Next steps:"
echo "   cd $EMULATOR_DIR/../deploying/tedbauer.github.io"
echo "   git add shrimp/"
echo "   git commit -m 'Update Shrimp IDE'"
echo "   git push"
