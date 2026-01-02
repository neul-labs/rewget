#!/bin/bash
# Release script for rwget
# Builds release binaries and creates release tarballs

set -e

VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
echo "Building rwget v${VERSION}..."

# Build directory
RELEASE_DIR="target/release-dist"
mkdir -p "$RELEASE_DIR"

# Function to build for a target
build_target() {
    local target="$1"
    local name="$2"

    echo "Building for $target..."

    if cargo build --release --target "$target" 2>/dev/null; then
        local out_dir="$RELEASE_DIR/$name"
        mkdir -p "$out_dir"

        # Copy binaries
        if [[ "$target" == *"windows"* ]]; then
            cp "target/$target/release/rwget.exe" "$out_dir/"
            cp "target/$target/release/rwgetd.exe" "$out_dir/"
        else
            cp "target/$target/release/rwget" "$out_dir/"
            cp "target/$target/release/rwgetd" "$out_dir/"
        fi

        # Copy man page
        if [[ -f "target/$target/release/build/rwget-"*/out/man/rwget.1 ]]; then
            cp "target/$target/release/build/rwget-"*/out/man/rwget.1 "$out_dir/"
        fi

        # Copy README and LICENSE
        cp README.md "$out_dir/"
        cp LICENSE "$out_dir/" 2>/dev/null || true

        # Create tarball
        local tarball="$RELEASE_DIR/rwget-$name.tar.gz"
        (cd "$RELEASE_DIR" && tar -czf "rwget-$name.tar.gz" "$name")

        # Calculate SHA256
        local sha256=$(sha256sum "$tarball" | cut -d' ' -f1)
        echo "$name: $sha256"
        echo "$sha256" > "$tarball.sha256"

        # Cleanup
        rm -rf "$out_dir"
    else
        echo "  Skipped: $target (cross-compilation not available)"
    fi
}

echo ""
echo "=== Building Release Binaries ==="
echo ""

# Native build (always works)
echo "Building native release..."
cargo build --release

# Copy native build
NATIVE_DIR="$RELEASE_DIR/native"
mkdir -p "$NATIVE_DIR"
cp target/release/rwget "$NATIVE_DIR/"
cp target/release/rwgetd "$NATIVE_DIR/"
if [[ -f target/release/build/rwget-*/out/man/rwget.1 ]]; then
    cp target/release/build/rwget-*/out/man/rwget.1 "$NATIVE_DIR/"
fi
cp README.md "$NATIVE_DIR/"

echo ""
echo "Native binaries built in: $NATIVE_DIR"
echo ""

# Cross-compile if targets are installed
# Install targets with: rustup target add <target>

TARGETS=(
    "x86_64-unknown-linux-gnu:x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu:aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin:x86_64-apple-darwin"
    "aarch64-apple-darwin:aarch64-apple-darwin"
    "x86_64-pc-windows-gnu:x86_64-pc-windows-gnu"
)

for entry in "${TARGETS[@]}"; do
    target="${entry%%:*}"
    name="${entry##*:}"
    build_target "$target" "$name"
done

echo ""
echo "=== Release Build Complete ==="
echo "Release artifacts in: $RELEASE_DIR"
echo ""
echo "To update Homebrew formula, replace PLACEHOLDER_SHA256_* with actual hashes."
