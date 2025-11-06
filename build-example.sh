#!/usr/bin/env bash
# Build tos-alloc example contracts

set -e

EXAMPLE="${1:-basic}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOS_NETWORK_DIR="$(dirname "$SCRIPT_DIR")"

# Determine architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    HOST_TRIPLE=aarch64-apple-darwin
else
    HOST_TRIPLE=x86_64-apple-darwin
fi

# TOS Rust compiler paths
RUSTC="${TOS_NETWORK_DIR}/rust/build/${HOST_TRIPLE}/stage1/bin/rustc"
LLVM_BIN="${TOS_NETWORK_DIR}/rust/build/${HOST_TRIPLE}/llvm/build/bin"

# Verify toolchain
if [ ! -f "$RUSTC" ]; then
    echo "Error: TOS Rust compiler not found at:"
    echo "  $RUSTC"
    echo ""
    echo "Please build the toolchain first:"
    echo "  cd ${TOS_NETWORK_DIR}/platform-tools"
    echo "  ./build.sh"
    exit 1
fi

echo "========================================"
echo "Building tos-alloc example: $EXAMPLE"
echo "========================================"
echo ""
echo "✓ Found TOS Rust compiler:"
"$RUSTC" --version
echo ""

# Set up environment
export RUSTC="$RUSTC"
export PATH="${LLVM_BIN}:${PATH}"

# 就地构建 (不使用tako的脚本 - 示例不在tako/examples/)
cd "$SCRIPT_DIR/examples/$EXAMPLE"

echo "Running: cargo +nightly build --release --target tbpf-tos-tos ..."
echo ""

if cargo +nightly build \
    --release \
    --target tbpf-tos-tos \
    -Zbuild-std=core,alloc \
    -Zbuild-std-features=panic_immediate_abort; then

    echo ""
    echo "========================================"
    echo "✓ Build Successful!"
    echo "========================================"
    echo ""

    # 查找输出文件
    OUTPUT_FILE=$(find target/tbpf-tos-tos/release -name "*.so" -type f | head -1)

    if [[ -n "$OUTPUT_FILE" ]]; then
        echo "Output: $OUTPUT_FILE"
        echo "Size: $(ls -lh "$OUTPUT_FILE" | awk '{print $5}')"
        echo ""
    fi
else
    echo ""
    echo "========================================"
    echo "✗ Build Failed"
    echo "========================================"
    exit 1
fi
