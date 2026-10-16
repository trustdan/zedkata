#!/usr/bin/env bash
# Packages and stages zedkata native binaries and external lesson assets for release.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

OUTPUT_DIR=""
TARGET=""
SKIP_BUILD=false
CLEAN=false
ARCHIVE=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --target)
      TARGET="$2"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD=true
      shift 1
      ;;
    --clean)
      CLEAN=true
      shift 1
      ;;
    --archive)
      ARCHIVE=true
      shift 1
      ;;
    -h|--help)
      echo "Usage: ./scripts/package.sh [--output-dir DIR] [--target TRIPLE] [--skip-build] [--clean] [--archive]"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Read version from Cargo.toml
VERSION=$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/')
if [[ -z "$VERSION" ]]; then
  VERSION="0.1.0"
fi

if [[ -z "$OUTPUT_DIR" ]]; then
  OUTPUT_DIR="$REPO_ROOT/dist/zedkata-v$VERSION"
fi

echo "========================================"
echo " Zedkata Release Packaging (v$VERSION)  "
echo "========================================"

if [[ "$CLEAN" == "true" && -d "$OUTPUT_DIR" ]]; then
  echo "Cleaning existing output directory: $OUTPUT_DIR"
  rm -rf "$OUTPUT_DIR"
fi

TARGET_RELEASE_DIR="$REPO_ROOT/target/release"
BUILD_ARGS=(build --workspace --release)
if [[ -n "$TARGET" ]]; then
  TARGET_RELEASE_DIR="$REPO_ROOT/target/$TARGET/release"
  BUILD_ARGS+=(--target "$TARGET")
fi

if [[ "$SKIP_BUILD" != "true" ]]; then
  echo ""
  echo "[1/5] Building release binaries with Cargo..."
  (cd "$REPO_ROOT" && cargo "${BUILD_ARGS[@]}")
else
  echo ""
  echo "[1/5] Skipping cargo build (--skip-build specified)..."
fi

BINARY_NAMES=("zedkata-tui" "zedkata-lsp" "zedkata-agent")
echo ""
echo "[2/5] Verifying binary artifacts in $TARGET_RELEASE_DIR..."
for bin in "${BINARY_NAMES[@]}"; do
  bin_path="$TARGET_RELEASE_DIR/$bin"
  if [[ ! -f "$bin_path" ]]; then
    # Try with .exe extension in case cross-compiling or MSYS
    if [[ -f "$bin_path.exe" ]]; then
      bin="$bin.exe"
    else
      echo "Error: Missing required release binary: $bin_path" >&2
      exit 1
    fi
  fi
  echo "  [OK] Found $bin"
done

echo ""
echo "[3/5] Assembling staging tree at $OUTPUT_DIR..."
mkdir -p "$OUTPUT_DIR/bin"
mkdir -p "$OUTPUT_DIR/lessons"
mkdir -p "$OUTPUT_DIR/extension"

for bin in "${BINARY_NAMES[@]}"; do
  src="$TARGET_RELEASE_DIR/$bin"
  dst="$OUTPUT_DIR/bin/$bin"
  cp -f "$src" "$dst"
  chmod +x "$dst"
done

cp -rf "$REPO_ROOT/lessons/"* "$OUTPUT_DIR/lessons/"
cp -f "$REPO_ROOT/extension/extension.toml" "$OUTPUT_DIR/extension/"
cp -f "$REPO_ROOT/extension/README.md" "$OUTPUT_DIR/extension/"
if [[ -d "$REPO_ROOT/extension/languages" ]]; then
  cp -rf "$REPO_ROOT/extension/languages" "$OUTPUT_DIR/extension/"
  echo "  [OK] Extension languages/ definitions packaged"
fi
if [[ -f "$REPO_ROOT/extension/target/wasm32-wasip2/release/zedkata_extension.wasm" ]]; then
  cp -f "$REPO_ROOT/extension/target/wasm32-wasip2/release/zedkata_extension.wasm" "$OUTPUT_DIR/extension/extension.wasm"
  echo "  [OK] Precompiled extension.wasm packaged"
fi
cp -f "$REPO_ROOT/README.md" "$OUTPUT_DIR/"
for doc in LICENSE LICENSE-MIT LICENSE-APACHE RELEASE_NOTES.md; do
  if [[ -f "$REPO_ROOT/$doc" ]]; then
    cp -f "$REPO_ROOT/$doc" "$OUTPUT_DIR/"
    echo "  [OK] $doc packaged"
  fi
done

echo ""
echo "[4/5] Generating release manifest..."
PACKAGED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
TARGET_NAME="${TARGET:-host}"
LESSON_COUNT=$(find "$OUTPUT_DIR/lessons" -maxdepth 1 -name "*.toml" | wc -l | tr -d ' ')

# Compute binary hashes and sizes
BIN_JSON_ITEMS=""
for bin in "${BINARY_NAMES[@]}"; do
  dst="$OUTPUT_DIR/bin/$bin"
  file_size=$(wc -c < "$dst" | tr -d ' ')
  hash=""
  if command -v sha256sum >/dev/null 2>&1; then
    hash=$(sha256sum "$dst" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    hash=$(shasum -a 256 "$dst" | awk '{print $1}')
  fi
  item="    { \"name\": \"$bin\", \"path\": \"bin/$bin\", \"sha256\": \"$hash\", \"size_bytes\": $file_size }"
  if [[ -z "$BIN_JSON_ITEMS" ]]; then
    BIN_JSON_ITEMS="$item"
  else
    BIN_JSON_ITEMS="$BIN_JSON_ITEMS,
$item"
  fi
done

cat <<EOF > "$OUTPUT_DIR/MANIFEST.json"
{
  "name": "zedkata",
  "version": "$VERSION",
  "packaged_at": "$PACKAGED_AT",
  "target": "$TARGET_NAME",
  "binaries": [
$BIN_JSON_ITEMS
  ],
  "curriculum": {
    "lesson_count": $LESSON_COUNT,
    "fixtures_present": true
  },
  "layout": {
    "bin_dir": "bin",
    "lessons_dir": "lessons",
    "extension_dir": "extension"
  }
}
EOF
echo "  [OK] MANIFEST.json written"

echo ""
echo "[5/5] Validating staged artifacts..."
for bin in "${BINARY_NAMES[@]}"; do
  staged_bin="$OUTPUT_DIR/bin/$bin"
  out=$("$staged_bin" --version)
  echo "  [OK] $bin --version reports: $out"
done

echo "  [OK] $LESSON_COUNT external lesson TOML files verified"

echo ""
echo "========================================"
echo " Release staged successfully!          "
echo " Staged Location: $OUTPUT_DIR"
echo "========================================"

if [[ "$ARCHIVE" == "true" ]]; then
  echo ""
  echo "Creating release archive..."
  ARCHIVE_TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
  ARCHIVE_NAME="zedkata-v$VERSION-$ARCHIVE_TARGET.tar.gz"
  PARENT_DIR=$(dirname "$OUTPUT_DIR")
  BASE_DIR=$(basename "$OUTPUT_DIR")
  mkdir -p "$PARENT_DIR"
  ARCHIVE_PATH="$PARENT_DIR/$ARCHIVE_NAME"
  rm -f "$ARCHIVE_PATH"
  (cd "$PARENT_DIR" && tar -czf "$ARCHIVE_NAME" "$BASE_DIR")
  echo "  [OK] Release archive created at: $ARCHIVE_PATH"
fi
