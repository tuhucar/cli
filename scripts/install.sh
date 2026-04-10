#!/bin/bash
set -euo pipefail

# TuhuCar CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/tuhucar/cli/main/scripts/install.sh | sh

REPO="tuhucar/cli"
INSTALL_DIR="${HOME}/.tuhucar/bin"
VERSION="${TUHUCAR_VERSION:-latest}"

info() { printf "\033[0;34m%s\033[0m\n" "$1"; }
error() { printf "\033[0;31mError: %s\033[0m\n" "$1" >&2; exit 1; }
success() { printf "\033[0;32m%s\033[0m\n" "$1"; }

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux)   os="linux" ;;
        Darwin)  os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="win32" ;;
        *)       error "Unsupported OS: $(uname -s). Supported: Linux, macOS, Windows." ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             error "Unsupported architecture: $(uname -m). Supported: x64, arm64." ;;
    esac

    # Detect musl on Linux
    local variant=""
    if [ "$os" = "linux" ]; then
        if [ -f /lib/ld-musl-*.so.1 ] 2>/dev/null || (ldd --version 2>&1 | grep -qi musl); then
            variant="-musl"
        fi
    fi

    local suffix=""
    if [ "$os" = "win32" ]; then
        suffix=".exe"
    fi

    echo "tuhucar-${os}-${arch}${variant}${suffix}"
}

get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    local version
    version=$(curl -fsSL "$url" 2>/dev/null | grep '"tag_name"' | head -1 | sed 's/.*"v\([^"]*\)".*/\1/')
    if [ -z "$version" ]; then
        error "Failed to determine latest version. Set TUHUCAR_VERSION=x.y.z to specify."
    fi
    echo "$version"
}

main() {
    info "TuhuCar CLI Installer"
    echo ""

    # Detect platform
    local artifact
    artifact=$(detect_platform)
    info "Detected platform: ${artifact}"

    # Determine version
    if [ "$VERSION" = "latest" ]; then
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi
    info "Version: ${VERSION}"

    # Download
    local download_url="https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}"
    local checksum_url="${download_url}.sha256"
    local tmp_dir
    tmp_dir=$(mktemp -d)
    local tmp_bin="${tmp_dir}/${artifact}"

    info "Downloading from ${download_url}..."
    curl -fsSL -o "$tmp_bin" "$download_url" || error "Download failed"

    # Verify checksum
    info "Verifying checksum..."
    if curl -fsSL -o "${tmp_dir}/checksum" "$checksum_url" 2>/dev/null; then
        local expected actual
        expected=$(cat "${tmp_dir}/checksum" | awk '{print $1}')
        actual=$(sha256sum "$tmp_bin" 2>/dev/null || shasum -a 256 "$tmp_bin" 2>/dev/null | awk '{print $1}')
        actual=$(echo "$actual" | awk '{print $1}')
        if [ "$expected" != "$actual" ]; then
            rm -rf "$tmp_dir"
            error "Checksum verification failed!"
        fi
        success "Checksum verified."
    else
        echo "Warning: Could not download checksum file, skipping verification."
    fi

    # Install
    mkdir -p "$INSTALL_DIR"
    local binary_name="tuhucar"
    if echo "$artifact" | grep -q ".exe"; then
        binary_name="tuhucar.exe"
    fi

    mv "$tmp_bin" "${INSTALL_DIR}/${binary_name}"
    chmod +x "${INSTALL_DIR}/${binary_name}"
    rm -rf "$tmp_dir"

    success "Installed tuhucar to ${INSTALL_DIR}/${binary_name}"

    # Add to PATH if needed
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        info "Add tuhucar to your PATH by adding this to your shell profile:"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        # Try to detect shell and suggest the right file
        local shell_profile=""
        case "${SHELL:-}" in
            */zsh)  shell_profile="~/.zshrc" ;;
            */bash) shell_profile="~/.bashrc" ;;
            */fish) shell_profile="~/.config/fish/config.fish" ;;
        esac
        if [ -n "$shell_profile" ]; then
            info "For your shell, run:"
            echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ${shell_profile}"
            echo "  source ${shell_profile}"
        fi
    fi

    echo ""

    # Best-effort skill install
    export PATH="${INSTALL_DIR}:${PATH}"
    if command -v tuhucar >/dev/null 2>&1; then
        info "Installing skills..."
        tuhucar skill install || echo "Note: Skill installation skipped. Run 'tuhucar skill install' later."
    fi

    echo ""
    success "Installation complete! Run 'tuhucar --help' to get started."
}

main "$@"
