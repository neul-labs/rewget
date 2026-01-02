#!/bin/bash
# Install script for rwget
# Usage: curl -sSL https://rwget.dev/install.sh | bash

set -e

VERSION="${RWGET_VERSION:-1.0.0}"
INSTALL_DIR="${RWGET_INSTALL_DIR:-$HOME/.local/bin}"
GITHUB_REPO="dipankardas011/rwget"

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="unknown-linux-gnu" ;;
        Darwin*) os="apple-darwin" ;;
        MINGW*|CYGWIN*|MSYS*) os="pc-windows-gnu" ;;
        *)
            echo "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *)
            echo "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

PLATFORM=$(detect_platform)
TARBALL="rwget-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${TARBALL}"

echo "rwget installer v${VERSION}"
echo ""
echo "Platform: ${PLATFORM}"
echo "Installing to: ${INSTALL_DIR}"
echo ""

# Create install directory
mkdir -p "${INSTALL_DIR}"

# Download and extract
TEMP_DIR=$(mktemp -d)
trap "rm -rf ${TEMP_DIR}" EXIT

echo "Downloading ${DOWNLOAD_URL}..."

if command -v curl &> /dev/null; then
    curl -sSL "${DOWNLOAD_URL}" -o "${TEMP_DIR}/${TARBALL}"
elif command -v wget &> /dev/null; then
    wget -q "${DOWNLOAD_URL}" -O "${TEMP_DIR}/${TARBALL}"
else
    echo "Error: curl or wget is required"
    exit 1
fi

echo "Extracting..."
tar -xzf "${TEMP_DIR}/${TARBALL}" -C "${TEMP_DIR}"

# Find and install binaries
EXTRACT_DIR="${TEMP_DIR}/${PLATFORM}"
if [[ ! -d "${EXTRACT_DIR}" ]]; then
    # Try without the leading directory
    EXTRACT_DIR="${TEMP_DIR}"
fi

echo "Installing binaries..."
install -m 755 "${EXTRACT_DIR}/rwget" "${INSTALL_DIR}/rwget"
install -m 755 "${EXTRACT_DIR}/rwgetd" "${INSTALL_DIR}/rwgetd"

# Install man page if available
if [[ -f "${EXTRACT_DIR}/rwget.1" ]] && [[ -d "${HOME}/.local/share/man/man1" ]]; then
    mkdir -p "${HOME}/.local/share/man/man1"
    install -m 644 "${EXTRACT_DIR}/rwget.1" "${HOME}/.local/share/man/man1/"
fi

echo ""
echo "Installation complete!"
echo ""

# Check if install directory is in PATH
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    echo "Note: ${INSTALL_DIR} is not in your PATH."
    echo "Add it by running:"
    echo ""
    echo "  export PATH=\"\${PATH}:${INSTALL_DIR}\""
    echo ""
    echo "Or add this line to your shell configuration file (.bashrc, .zshrc, etc.)"
    echo ""
fi

echo "Verify installation:"
echo "  rwget --rwget-version"
echo ""
echo "Get started:"
echo "  rwget --rwget-help"
