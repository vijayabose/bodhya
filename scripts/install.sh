#!/usr/bin/env bash

# Bodhya Installer for Linux/macOS
# Version: 1.0
# License: MIT

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/vijayabose/bodhya"
INSTALL_DIR="${HOME}/.local/bin"
BODHYA_HOME="${HOME}/.bodhya"

# Print colored message
print_msg() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Print section header
print_header() {
    echo ""
    print_msg "$BLUE" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_msg "$BLUE" "$1"
    print_msg "$BLUE" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        *)
            print_msg "$RED" "Error: Unsupported operating system: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_msg "$RED" "Error: Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    print_msg "$GREEN" "Detected platform: $OS ($ARCH)"
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check for Rust
    if ! command_exists rustc; then
        print_msg "$YELLOW" "Rust is not installed."
        print_msg "$YELLOW" "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        local rust_version=$(rustc --version | cut -d' ' -f2)
        print_msg "$GREEN" "✓ Rust installed: $rust_version"
    fi

    # Check Cargo
    if ! command_exists cargo; then
        print_msg "$RED" "Error: Cargo not found. Please install Rust."
        exit 1
    fi

    # Check Git
    if ! command_exists git; then
        print_msg "$RED" "Error: Git is required but not installed."
        print_msg "$YELLOW" "Please install Git:"
        if [ "$OS" = "linux" ]; then
            print_msg "$YELLOW" "  sudo apt-get install git  # Debian/Ubuntu"
            print_msg "$YELLOW" "  sudo yum install git      # RHEL/CentOS"
        else
            print_msg "$YELLOW" "  brew install git"
        fi
        exit 1
    fi

    # Check SQLite (recommended but not required)
    if ! command_exists sqlite3; then
        print_msg "$YELLOW" "⚠ SQLite3 not found (optional for history feature)"
    else
        print_msg "$GREEN" "✓ SQLite3 installed"
    fi

    print_msg "$GREEN" "✓ All prerequisites satisfied"
}

# Clone or update repository
setup_repository() {
    print_header "Setting Up Repository"

    local tmp_dir=$(mktemp -d)
    local build_dir="$tmp_dir/bodhya"

    print_msg "$BLUE" "Cloning repository to temporary directory..."
    git clone --depth 1 "$REPO_URL" "$build_dir"

    echo "$build_dir"
}

# Build Bodhya
build_bodhya() {
    local build_dir=$1

    print_header "Building Bodhya"

    cd "$build_dir"

    print_msg "$BLUE" "Building in release mode (this may take a few minutes)..."
    cargo build --release -p bodhya-cli

    if [ $? -ne 0 ]; then
        print_msg "$RED" "Error: Build failed"
        exit 1
    fi

    print_msg "$GREEN" "✓ Build successful"
}

# Install binary
install_binary() {
    local build_dir=$1

    print_header "Installing Bodhya"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Copy binary
    print_msg "$BLUE" "Installing binary to $INSTALL_DIR..."
    cp "$build_dir/target/release/bodhya" "$INSTALL_DIR/bodhya"
    chmod +x "$INSTALL_DIR/bodhya"

    print_msg "$GREEN" "✓ Binary installed to $INSTALL_DIR/bodhya"
}

# Setup PATH
setup_path() {
    print_header "Configuring PATH"

    # Check if INSTALL_DIR is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_msg "$YELLOW" "⚠ $INSTALL_DIR is not in your PATH"

        local shell_rc=""
        if [ -n "$BASH_VERSION" ]; then
            shell_rc="$HOME/.bashrc"
        elif [ -n "$ZSH_VERSION" ]; then
            shell_rc="$HOME/.zshrc"
        else
            shell_rc="$HOME/.profile"
        fi

        print_msg "$BLUE" "Adding to $shell_rc..."
        echo "" >> "$shell_rc"
        echo "# Added by Bodhya installer" >> "$shell_rc"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_rc"

        print_msg "$GREEN" "✓ PATH updated in $shell_rc"
        print_msg "$YELLOW" "  Please restart your shell or run: source $shell_rc"
    else
        print_msg "$GREEN" "✓ $INSTALL_DIR already in PATH"
    fi
}

# Initialize Bodhya
initialize_bodhya() {
    print_header "Initializing Bodhya"

    if [ -d "$BODHYA_HOME" ]; then
        print_msg "$YELLOW" "⚠ Bodhya is already initialized at $BODHYA_HOME"
        read -p "Do you want to reinitialize? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_msg "$BLUE" "Skipping initialization"
            return
        fi
    fi

    # Run init command
    print_msg "$BLUE" "Running: bodhya init --profile full"

    # Export PATH for this session
    export PATH="$PATH:$INSTALL_DIR"

    if "$INSTALL_DIR/bodhya" init --profile full; then
        print_msg "$GREEN" "✓ Bodhya initialized successfully"
    else
        print_msg "$YELLOW" "⚠ Initialization skipped (you can run 'bodhya init' later)"
    fi
}

# Cleanup
cleanup() {
    local tmp_dir=$1
    if [ -n "$tmp_dir" ] && [ -d "$tmp_dir" ]; then
        print_msg "$BLUE" "Cleaning up temporary files..."
        rm -rf "$tmp_dir"
    fi
}

# Print success message
print_success() {
    print_header "Installation Complete!"

    print_msg "$GREEN" "✅ Bodhya has been successfully installed!"
    echo ""
    print_msg "$BLUE" "Installation directory: $INSTALL_DIR"
    print_msg "$BLUE" "Configuration directory: $BODHYA_HOME"
    echo ""
    print_msg "$YELLOW" "Next steps:"
    echo "  1. Restart your shell or run: source ~/.bashrc (or ~/.zshrc)"
    echo "  2. Verify installation: bodhya --version"
    echo "  3. View help: bodhya --help"
    echo "  4. Run a task: bodhya run --domain code --task \"Create hello world\""
    echo ""
    print_msg "$BLUE" "Documentation:"
    echo "  • User Guide: https://github.com/vijayabose/bodhya/blob/main/USER_GUIDE.md"
    echo "  • Developer Guide: https://github.com/vijayabose/bodhya/blob/main/DEVELOPER_GUIDE.md"
    echo ""
    print_msg "$GREEN" "Happy coding with Bodhya! 🚀"
}

# Main installation flow
main() {
    print_header "Bodhya Installer v1.0"

    print_msg "$BLUE" "This script will install Bodhya on your system."
    echo ""

    # Detect platform
    detect_platform

    # Check prerequisites
    check_prerequisites

    # Setup repository
    local build_dir=$(setup_repository)
    local tmp_dir=$(dirname "$build_dir")

    # Ensure cleanup on exit
    trap "cleanup '$tmp_dir'" EXIT

    # Build
    build_bodhya "$build_dir"

    # Install
    install_binary "$build_dir"

    # Setup PATH
    setup_path

    # Initialize
    initialize_bodhya

    # Success
    print_success
}

# Run main function
main "$@"
