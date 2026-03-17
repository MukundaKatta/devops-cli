#!/usr/bin/env bash
#
# DevTool installer
# Usage: curl -fsSL https://raw.githubusercontent.com/youruser/devtool/main/install.sh | bash
#

set -euo pipefail

REPO="youruser/devtool"
BINARY_NAME="devtool"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${CYAN}>>>${NC} $*"; }
success() { echo -e "${GREEN}done:${NC} $*"; }
warn() { echo -e "${YELLOW}warn:${NC} $*"; }
error() { echo -e "${RED}error:${NC} $*" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)   os="linux" ;;
        Darwin*)  os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)        error "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7*)        arch="armv7" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get the latest version tag from GitHub
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        error "Could not determine latest version"
    fi

    echo "$version"
}

# Download and install the binary
install() {
    local platform version download_url tmp_dir archive_name

    info "Detecting platform..."
    platform=$(detect_platform)
    info "Platform: ${platform}"

    info "Fetching latest version..."
    version=$(get_latest_version)
    info "Version: ${version}"

    # Construct download URL
    if [[ "$platform" == *"windows"* ]]; then
        archive_name="${BINARY_NAME}-${version}-${platform}.zip"
    else
        archive_name="${BINARY_NAME}-${version}-${platform}.tar.gz"
    fi

    download_url="https://github.com/${REPO}/releases/download/${version}/${archive_name}"

    # Create temp directory
    tmp_dir=$(mktemp -d)
    trap "rm -rf ${tmp_dir}" EXIT

    info "Downloading ${archive_name}..."
    if ! curl -fsSL "${download_url}" -o "${tmp_dir}/${archive_name}"; then
        error "Download failed. Check if release exists at: ${download_url}"
    fi

    info "Extracting..."
    cd "${tmp_dir}"
    if [[ "$archive_name" == *.zip ]]; then
        unzip -q "${archive_name}"
    else
        tar xzf "${archive_name}"
    fi

    # Find the binary
    local binary_path
    binary_path=$(find "${tmp_dir}" -name "${BINARY_NAME}" -type f | head -1)

    if [ -z "$binary_path" ]; then
        error "Binary not found in archive"
    fi

    # Install
    info "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
    chmod +x "${binary_path}"

    if [ -w "${INSTALL_DIR}" ]; then
        mv "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        warn "Need sudo to install to ${INSTALL_DIR}"
        sudo mv "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    # Verify
    if command -v "${BINARY_NAME}" &>/dev/null; then
        success "DevTool ${version} installed successfully!"
        echo ""
        info "Run 'devtool --help' to get started"
        echo ""

        # Suggest shell completions
        echo -e "${CYAN}Tip:${NC} Generate shell completions:"
        echo "  devtool completions bash >> ~/.bashrc"
        echo "  devtool completions zsh >> ~/.zshrc"
        echo "  devtool completions fish > ~/.config/fish/completions/devtool.fish"
    else
        warn "Installed but '${BINARY_NAME}' not found in PATH"
        warn "Make sure ${INSTALL_DIR} is in your PATH"
    fi
}

# Build from source as fallback
build_from_source() {
    info "Building from source..."

    if ! command -v cargo &>/dev/null; then
        error "Rust/Cargo not found. Install from https://rustup.rs"
    fi

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf ${tmp_dir}" EXIT

    info "Cloning repository..."
    git clone "https://github.com/${REPO}.git" "${tmp_dir}/devtool"
    cd "${tmp_dir}/devtool"

    info "Building release binary..."
    cargo build --release

    local binary_path="target/release/${BINARY_NAME}"
    chmod +x "${binary_path}"

    if [ -w "${INSTALL_DIR}" ]; then
        mv "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        sudo mv "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    success "DevTool built and installed successfully!"
}

# Main
main() {
    echo ""
    echo -e "${CYAN}DevTool Installer${NC}"
    echo "=================================="
    echo ""

    # Check for required tools
    if ! command -v curl &>/dev/null; then
        error "'curl' is required but not installed"
    fi

    # Try binary install first, fall back to source
    if ! install 2>/dev/null; then
        warn "Pre-built binary not available for your platform"
        info "Attempting to build from source..."
        build_from_source
    fi
}

main "$@"
