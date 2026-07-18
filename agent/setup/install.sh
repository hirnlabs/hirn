#!/usr/bin/env bash
set -e

# Repository settings
REPO="hirnlabs/hirn"
VERSION=""

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -v|--version) VERSION="$2"; shift ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)
        OS_NAME="linux"
        if [ "${ARCH}" = "x86_64" ]; then
            TARGET="x86_64-unknown-linux-gnu"
            EXT="tar.gz"
        elif [ "${ARCH}" = "aarch64" ] || [ "${ARCH}" = "arm64" ]; then
            TARGET="aarch64-unknown-linux-gnu"
            EXT="tar.gz"
        else
            echo "Unsupported architecture: ${ARCH}"
            exit 1
        fi
        ;;
    Darwin)
        OS_NAME="macos"
        if [ "${ARCH}" = "x86_64" ]; then
            TARGET="x86_64-apple-darwin"
            EXT="tar.gz"
        elif [ "${ARCH}" = "arm64" ] || [ "${ARCH}" = "aarch64" ]; then
            TARGET="aarch64-apple-darwin"
            EXT="tar.gz"
        else
            echo "Unsupported architecture: ${ARCH}"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported OS: ${OS}"
        exit 1
        ;;
esac

# Create directory structure
HIRN_DIR="${HOME}/.hirn"
BIN_DIR="${HIRN_DIR}/bin"
CONFIG_DIR="${HIRN_DIR}/config"

mkdir -p "${BIN_DIR}"
mkdir -p "${CONFIG_DIR}"

# Fetch release info
if [ -z "${VERSION}" ]; then
    echo "Fetching latest release information..."
    # Get latest published release tag
    TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | head -n 1 | cut -d'"' -f4)
    if [ -z "${TAG}" ]; then
        echo "Failed to fetch latest release tag. Is there a published release?"
        exit 1
    fi
else
    TAG="${VERSION}"
fi

echo "Installing Hirn Agent ${TAG} for ${TARGET}..."

# Construct download URL
# Format: https://github.com/hirnlabs/hirn/releases/download/v0.1.0/hirn-x86_64-unknown-linux-gnu.tar.gz
ASSET_NAME="hirn-${TAG}-${TARGET}.${EXT}"
DOWNLOAD_URL="https://github.com/hirnlabs/hirn/releases/download/${TAG}/${ASSET_NAME}"

TEMP_DIR=$(mktemp -d)
CLEANUP() {
    rm -rf "${TEMP_DIR}"
}
trap CLEANUP EXIT

echo "Downloading from ${DOWNLOAD_URL}..."
if ! curl -L -o "${TEMP_DIR}/${ASSET_NAME}" "${DOWNLOAD_URL}"; then
    echo "Download failed. Please check your internet connection or tag/version '${TAG}'."
    exit 1
fi

echo "Extracting binary..."
tar -xzf "${TEMP_DIR}/${ASSET_NAME}" -C "${TEMP_DIR}"

if [ ! -f "${TEMP_DIR}/hirn" ]; then
    echo "Binary 'hirn' not found in extracted archive."
    exit 1
fi

mv "${TEMP_DIR}/hirn" "${BIN_DIR}/hirn"
chmod +x "${BIN_DIR}/hirn"

echo ""
echo "==============================================="
echo " Hirn Agent was successfully installed!"
echo "==============================================="
echo "Binary path: ${BIN_DIR}/hirn"
echo "Config path: ${CONFIG_DIR}"
echo ""
echo "To add it to your PATH, add the following line to your shell profile"
echo "(e.g., ~/.bashrc, ~/.zshrc, or ~/.bash_profile):"
echo ""
echo "  export PATH=\"\${PATH}:${BIN_DIR}\""
echo ""
echo "Then, restart your terminal or run:"
echo "  source <your-profile-file>"
echo "==============================================="
