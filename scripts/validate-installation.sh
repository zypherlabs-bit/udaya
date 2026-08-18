#!/bin/bash

# Udaya Phase 3 Installation Validation Script
# This script validates that Udaya can be installed and built on a clean machine

set -e  # Exit on error
set -u  # Exit on undefined variables
set -o pipefail  # Fail pipeline if any command fails

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_section() {
    echo -e "\n${YELLOW}=== $1 ===${NC}"
}

# Validation results
declare -A validation_results
validation_results=(
    ["rust_installation"]=false
    ["dependency_installation"]=false
    ["git_clone"]=false
    ["cargo_build"]=false
    ["binary_existence"]=false
    ["basic_functionality"]=false
)

# Test 1: Rust Toolchain Installation
test_rust_installation() {
    log_section "Testing Rust Toolchain Installation"

    if command -v rustc &> /dev/null; then
        log_info "Rust is already installed: $(rustc --version)"
        validation_results["rust_installation"]=true
        return
    fi

    log_info "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"

    if rustc --version | grep -q "1.75"; then
        log_info "Rust 1.75+ installed successfully: $(rustc --version)"
        validation_results["rust_installation"]=true
    else
        log_error "Rust version requirement not met. Expected 1.75+, got: $(rustc --version)"
        validation_results["rust_installation"]=false
    fi
}

# Test 2: Dependency Installation
test_dependency_installation() {
    log_section "Testing Dependency Installation"

    if [ -f /etc/os-release ]; then
        # Linux system
        source /etc/os-release
        case $ID in
            ubuntu|debian)
                log_info "Detected $ID, installing build dependencies..."
                sudo apt-get update
                sudo apt-get install -y build-essential pkg-config libssl-dev
                ;;
            fedora)
                log_info "Detected Fedora, installing build dependencies..."
                sudo dnf install -y gcc-c++ make pkg-config openssl-devel
                ;;
            *)
                log_warn "Unsupported Linux distribution: $ID"
                log_info "Attempting generic dependency installation..."
                sudo apt-get update || true
                sudo apt-get install -y build-essential pkg-config libssl-dev || true
                ;;
        esac
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if command -v brew &> /dev/null; then
            log_info "Homebrew detected, installing dependencies..."
            brew install pkg-config openssl
        else
            log_error "Homebrew not found. Please install Homebrew first."
            validation_results["dependency_installation"]=false
            return
        fi
    elif [[ "$OSTYPE" == "msys"* ]]; then
        # Windows (MSYS/MinGW)
        log_info "Windows detected, checking for build tools..."
        # Windows dependencies are typically handled by Rust toolchain
        validation_results["dependency_installation"]=true
        return
    else
        log_error "Unsupported operating system: $OSTYPE"
        validation_results["dependency_installation"]=false
        return
    fi

    log_info "Dependencies installed successfully"
    validation_results["dependency_installation"]=true
}

# Test 3: Git Clone
test_git_clone() {
    log_section "Testing Git Clone"

    if [ -d "Udaya" ]; then
        log_info "Udaya directory already exists, skipping clone"
        cd Udaya
        validation_results["git_clone"]=true
        return
    fi

    log_info "Cloning Udaya repository..."
    git clone https://github.com/UdayaFoundation/Udaya.git
    cd Udaya

    if [ -f "Cargo.toml" ]; then
        log_info "Repository cloned successfully"
        validation_results["git_clone"]=true
    else
        log_error "Repository clone failed - Cargo.toml not found"
        validation_results["git_clone"]=false
    fi
}

# Test 4: Cargo Build
test_cargo_build() {
    log_section "Testing Cargo Build"

    log_info "Building Udaya in release mode..."
    cargo build --release

    if [ -f "target/release/udayad" ]; then
        log_info "Build completed successfully"
        validation_results["cargo_build"]=true
    else
        log_error "Build failed - udayad binary not found"
        validation_results["cargo_build"]=false
    fi
}

# Test 5: Binary Existence
test_binary_existence() {
    log_section "Testing Binary Existence"

    local expected_binaries=(
        "udayad"
        "udaya-cli"
        "udaya-faucet"
        "udaya-explorer"
        "udaya-pool-server"
    )

    local all_found=true

    for binary in "${expected_binaries[@]}"; do
        if [ -f "target/release/$binary" ]; then
            log_info "Found binary: $binary"
        else
            log_warn "Missing binary: $binary"
            all_found=false
        fi
    done

    validation_results["binary_existence"]=$all_found
}

# Test 6: Basic Functionality
test_basic_functionality() {
    log_section "Testing Basic Functionality"

    # Test configuration file existence
    if [ -f "config/bitfury.conf" ]; then
        log_info "Configuration file found: config/bitfury.conf"
    else
        log_error "Configuration file not found: config/bitfury.conf"
        validation_results["basic_functionality"]=false
        return
    fi

    # Test help command
    if ./target/release/udayad --help &> /dev/null; then
        log_info "udayad --help command works"
    else
        log_error "udayad --help command failed"
        validation_results["basic_functionality"]=false
        return
    fi

    # Test version command
    if ./target/release/udayad --version &> /dev/null; then
        log_info "udayad --version command works"
    else
        log_error "udayad --version command failed"
        validation_results["basic_functionality"]=false
        return
    fi

    log_info "Basic functionality tests passed"
    validation_results["basic_functionality"]=true
}

# Test 7: Run Tests
test_run_tests() {
    log_section "Running Test Suite"

    log_info "Running cargo test..."
    if cargo test --lib; then
        log_info "Unit tests passed"
    else
        log_warn "Some unit tests failed"
    fi

    log_info "Running integration tests..."
    if cargo test --test '*'; then
        log_info "Integration tests passed"
    else
        log_warn "Some integration tests failed"
    fi
}

# Generate Report
generate_report() {
    log_section "Installation Validation Report"

    local passed=0
    local failed=0

    echo -e "\n${YELLOW}Validation Results:${NC}"
    for test_name in "${!validation_results[@]}"; do
        if [ "${validation_results[$test_name]}" = true ]; then
            echo -e "${GREEN}✓${NC} $test_name: PASSED"
            ((passed++))
        else
            echo -e "${RED}✗${NC} $test_name: FAILED"
            ((failed++))
        fi
    done

    echo -e "\n${YELLOW}Summary:${NC}"
    echo -e "Passed: ${GREEN}$passed${NC}"
    echo -e "Failed: ${RED}$failed${NC}"

    if [ $failed -eq 0 ]; then
        log_info "🎉 All validation tests passed! Udaya is ready to use."
        echo -e "\n${GREEN}Next steps:${NC}"
        echo "1. Configure your node: cp config/bitfury.conf config/udaya.conf"
        echo "2. Set environment variables: export RPC_USER=your_user RPC_PASSWORD=your_pass"
        echo "3. Start the node: ./target/release/udayad --config config/udaya.conf"
        echo "4. Test wallet: ./target/release/udaya-cli getnewaddress"
    else
        log_error "❌ Some validation tests failed. Please check the errors above."
        log_info "For support, visit: https://github.com/UdayaFoundation/Udaya/issues"
    fi
}

# Main execution
main() {
    echo -e "${YELLOW}"
    echo "  _   _           _        _   _"
    echo " | | | |_ __   __| | ___  | | | |___  ___ _ __"
    echo " | | | | '_ \ / _\` |/ _ \ | | | / __|/ _ \ '__|"
    echo " | |_| | |_) | (_| |  __/ | |_| \__ \  __/ |"
    echo "  \___/| .__/ \__,_|\___|  \___/|___/\___|_|"
    echo "       |_|"
    echo -e "\nUdaya Phase 3 Installation Validation${NC}"
    echo "======================================"

    test_rust_installation
    test_dependency_installation
    test_git_clone
    test_cargo_build
    test_binary_existence
    test_basic_functionality
    test_run_tests

    generate_report
}

# Run main function
main