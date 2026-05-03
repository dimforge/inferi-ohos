#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_DIR="$SCRIPT_DIR/rust"
ENTRY_LIBS="$SCRIPT_DIR/entry/libs"
NDK_HOME="/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony"
DEVECO_SDK="/Applications/DevEco-Studio.app/Contents/sdk"
HVIGOR="/Applications/DevEco-Studio.app/Contents/tools/node/bin/node /Applications/DevEco-Studio.app/Contents/tools/hvigor/bin/hvigorw.js"

echo "==> Building Rust..."
cd "$RUST_DIR"
OHOS_NDK_HOME="$NDK_HOME" ./build.sh manual

echo "==> Copying libc++_shared.so..."
cp "$NDK_HOME/native/llvm/lib/aarch64-linux-ohos/libc++_shared.so" "$ENTRY_LIBS/arm64-v8a/"
cp "$NDK_HOME/native/llvm/lib/x86_64-linux-ohos/libc++_shared.so" "$ENTRY_LIBS/x86_64/"

echo "==> Building HAP..."
cd "$SCRIPT_DIR"
DEVECO_SDK_HOME="$DEVECO_SDK" $HVIGOR --mode module -p module=entry@default -p product=default assembleHap

echo "==> Done. Hit Run in DevEco Studio."
