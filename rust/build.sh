#!/usr/bin/env bash
#
# Build the inferi-ohos Rust library for OpenHarmony targets.
#
# Prerequisites:
#   1. Rust targets installed:
#        rustup target add aarch64-unknown-linux-ohos x86_64-unknown-linux-ohos
#   2. OHOS_NDK_HOME set to the OpenHarmony SDK path, e.g.:
#        export OHOS_NDK_HOME=/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony
#
# Usage:
#   ./build.sh          # builds for both targets using ohrs
#   ./build.sh manual   # builds using cargo + linker wrapper scripts
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBS_DIR="$PROJECT_ROOT/entry/libs"

if [ -z "${OHOS_NDK_HOME:-}" ]; then
    echo "Error: OHOS_NDK_HOME is not set."
    echo "Set it to your OpenHarmony SDK path, e.g.:"
    echo "  export OHOS_NDK_HOME=/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony"
    exit 1
fi

CLANG="$OHOS_NDK_HOME/native/llvm/bin/clang"
SYSROOT="$OHOS_NDK_HOME/native/sysroot"

if [ ! -f "$CLANG" ]; then
    echo "Error: Clang not found at $CLANG"
    echo "Verify that OHOS_NDK_HOME is set correctly."
    exit 1
fi

# --- Option A: build using ohrs (recommended) ---
if [ "${1:-}" != "manual" ] && command -v ohrs &>/dev/null; then
    echo "==> Building with ohrs..."
    cd "$SCRIPT_DIR"
    ohrs build

    echo "==> Copying .so files to entry/libs/..."
    mkdir -p "$LIBS_DIR/arm64-v8a" "$LIBS_DIR/x86_64"
    # ohrs places outputs in dist/<arch>/
    [ -f dist/aarch64/libinferi_ohos.so ] && cp dist/aarch64/libinferi_ohos.so "$LIBS_DIR/arm64-v8a/libinferi_ohos.so"
    [ -f dist/x86_64/libinferi_ohos.so ]  && cp dist/x86_64/libinferi_ohos.so  "$LIBS_DIR/x86_64/libinferi_ohos.so"

    echo "==> Done (ohrs)."
    exit 0
fi

# --- Option B: manual build with cargo + linker wrapper ---
echo "==> Building manually with cargo..."

# Create temporary linker wrapper scripts
TMPDIR="$(mktemp -d)"
trap "rm -rf $TMPDIR" EXIT

cat > "$TMPDIR/aarch64-unknown-linux-ohos-clang.sh" <<EOF
#!/bin/sh
exec "$CLANG" --target=aarch64-linux-ohos --sysroot="$SYSROOT" -D__MUSL__ "\$@"
EOF

cat > "$TMPDIR/x86_64-unknown-linux-ohos-clang.sh" <<EOF
#!/bin/sh
exec "$CLANG" --target=x86_64-linux-ohos --sysroot="$SYSROOT" -D__MUSL__ "\$@"
EOF

chmod +x "$TMPDIR"/*.sh
export PATH="$TMPDIR:$PATH"

cd "$SCRIPT_DIR"

echo "  -> aarch64-unknown-linux-ohos"
cargo build --release --target aarch64-unknown-linux-ohos

echo "  -> x86_64-unknown-linux-ohos"
cargo build --release --target x86_64-unknown-linux-ohos

echo "==> Copying .so files to entry/libs/..."
mkdir -p "$LIBS_DIR/arm64-v8a" "$LIBS_DIR/x86_64"
cp target/aarch64-unknown-linux-ohos/release/libinferi_ohos.so "$LIBS_DIR/arm64-v8a/libinferi_ohos.so"
cp target/x86_64-unknown-linux-ohos/release/libinferi_ohos.so  "$LIBS_DIR/x86_64/libinferi_ohos.so"

echo "==> Done (manual)."
