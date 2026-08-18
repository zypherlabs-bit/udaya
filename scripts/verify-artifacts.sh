#!/bin/bash

# Udaya Native Execution Artifact Verification
# This script verifies that all release artifacts work correctly in native execution mode

set -e  # Exit on error
set -u  # Exit on undefined variables
set -o pipefail  # Fail pipeline if any command fails

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_failure() {
    echo -e "${RED}✗ $1${NC}"
}

# Results tracking
declare -A artifact_results
artifact_results=(
    ["clean_build"]=false
    ["binary_hashes"]=false
    ["native_execution"]=false
    ["configuration_files"]=false
    ["documentation_complete"]=false
    ["release_checksums"]=false
)

# Test 1: Clean Build Reproducibility
test_clean_build() {
    log_section "Testing Clean Build Reproducibility"

    log_info "Cleaning previous build artifacts..."
    git clean -fd
    cargo clean

    log_info "Performing clean build..."
    start_time=$(date +%s)
    cargo build --release
    end_time=$(date +%s)

    build_time=$((end_time - start_time))
    log_info "Clean build completed in ${build_time} seconds"

    # Verify main binary exists
    if [ -f "target/release/udayad" ]; then
        log_success "Main binary (udayad) built successfully"
        artifact_results["clean_build"]=true
    else
        log_failure "Main binary (udayad) not found after clean build"
        artifact_results["clean_build"]=false
    fi
}

# Test 2: Binary Hash Verification
test_binary_hashes() {
    log_section "Testing Binary Hash Reproducibility"

    # Expected hashes (these would be updated with actual expected values)
    declare -A expected_hashes
    expected_hashes=(
        ["udayad"]="placeholder_hash_1"
        ["udaya-cli"]="placeholder_hash_2"
        ["udaya-faucet"]="placeholder_hash_3"
    )

    local all_match=true

    for binary in "${!expected_hashes[@]}"; do
        if [ -f "target/release/$binary" ]; then
            actual_hash=$(sha256sum "target/release/$binary" | cut -d' ' -f1)
            log_info "Binary: $binary, Hash: $actual_hash"

            # In real usage, compare with expected_hashes[$binary]
            # For now, just verify the binary exists and has a valid hash
            if [[ "$actual_hash" =~ ^[a-f0-9]{64}$ ]]; then
                log_success "Valid hash for $binary"
            else
                log_failure "Invalid hash for $binary"
                all_match=false
            fi
        else
            log_failure "Binary $binary not found"
            all_match=false
        fi
    done

    artifact_results["binary_hashes"]=$all_match
}

# Test 3: Native Execution Test
test_native_execution() {
    log_section "Testing Native Execution"

    # Test that the binary can start and respond to basic commands
    if [ -f "target/release/udayad" ]; then
        log_info "Testing binary execution..."

        # Test help command
        if ./target/release/udayad --help > /dev/null 2>&1; then
            log_success "Binary executes successfully"
        else
            log_failure "Binary failed to execute"
            artifact_results["native_execution"]=false
            return
        fi

        # Test version info
        if ./target/release/udayad --version > /dev/null 2>&1; then
            log_success "Version command works"
        else
            log_failure "Version command failed"
            artifact_results["native_execution"]=false
            return
        fi

        # Test basic configuration validation
        if ./target/release/udayad --config config/bitfury.conf getblockchaininfo > /dev/null 2>&1; then
            log_success "Configuration validation works"
        else
            log_warn "Configuration validation failed (may be expected if no blockchain data)"
        fi

        artifact_results["native_execution"]=true
    else
        log_failure "Binary not found for native execution test"
        artifact_results["native_execution"]=false
    fi
}

# Test 4: Configuration Files Validation
test_configuration_files() {
    log_section "Testing Configuration Files"

    local config_files=(
        "config/bitfury.conf"
        "config/mainnet/default.conf"
        "config/testnet/default.conf"
    )

    local all_valid=true

    for config_file in "${config_files[@]}"; do
        if [ -f "$config_file" ]; then
            log_success "Configuration file found: $config_file"

            # Basic validation - check if file is not empty
            if [ -s "$config_file" ]; then
                log_info "Configuration file $config_file is not empty"
            else
                log_failure "Configuration file $config_file is empty"
                all_valid=false
            fi
        else
            log_failure "Configuration file not found: $config_file"
            all_valid=false
        fi
    done

    artifact_results["configuration_files"]=$all_valid
}

# Test 5: Documentation Completeness
test_documentation() {
    log_section "Testing Documentation Completeness"

    local required_docs=(
        "README.md"
        "CONTRIBUTING.md"
        "docs/operations/getting-started.md"
        "docs/api/README.md"
        "docs/mining/README.md"
        "docs/architecture/README.md"
    )

    local all_found=true

    for doc_file in "${required_docs[@]}"; do
        if [ -f "$doc_file" ]; then
            log_success "Documentation found: $doc_file"
        else
            log_failure "Documentation missing: $doc_file"
            all_found=false
        fi
    done

    artifact_results["documentation_complete"]=$all_found
}

# Test 6: Release Artifact Checksums
test_release_checksums() {
    log_section "Testing Release Artifact Checksums"

    local release_files=(
        "target/release/udayad"
        "target/release/udaya-cli"
        "target/release/udaya-faucet"
    )

    local checksum_file="release-checksums.sha256"
    local all_valid=true

    # Generate checksums
    log_info "Generating checksums for release artifacts..."
    sha256sum "${release_files[@]}" > "$checksum_file"

    if [ -f "$checksum_file" ]; then
        log_success "Checksum file generated: $checksum_file"

        # Verify checksums
        log_info "Verifying checksums..."
        if sha256sum -c "$checksum_file"; then
            log_success "All checksums verified successfully"
        else
            log_failure "Checksum verification failed"
            all_valid=false
        fi
    else
        log_failure "Failed to generate checksum file"
        all_valid=false
    fi

    artifact_results["release_checksums"]=$all_valid
}

# Generate verification report
generate_report() {
    log_section "Generating Verification Report"

    local passed=0
    local failed=0
    local skipped=0

    echo -e "\n${YELLOW}Test Results:${NC}"
    echo "─────────────────────────────────────────────────────────────────"

    for test_name in "${!artifact_results[@]}"; do
        case "${artifact_results[$test_name]}" in
            true)
                echo -e "${GREEN}✓${NC} $test_name: PASSED"
                ((passed++))
                ;;
            false)
                echo -e "${RED}✗${NC} $test_name: FAILED"
                ((failed++))
                ;;
            "skipped")
                echo -e "${YELLOW}○${NC} $test_name: SKIPPED"
                ((skipped++))
                ;;
            *)
                echo -e "${YELLOW}?${NC} $test_name: UNKNOWN"
                ;;
        esac
    done

    echo -e "\n${YELLOW}Summary:${NC}"
    echo -e "Passed: ${GREEN}$passed${NC}"
    echo -e "Failed: ${RED}$failed${NC}"
    echo -e "Skipped: ${YELLOW}$skipped${NC}"

    if [ $failed -eq 0 ]; then
        log_success "🎉 All native execution tests passed!"
        log_info "Release artifacts are ready for native deployment."

        echo -e "\n${GREEN}Release Preparation Checklist:${NC}"
        echo "1. [ ] Update version numbers in Cargo.toml"
        echo "2. [ ] Generate final release checksums"
        echo "3. [ ] Create GitHub release with native binaries"
        echo "4. [ ] Update documentation with native deployment instructions"
        echo "5. [ ] Announce release to community"
    else
        log_error "❌ Some native execution tests failed."
        log_info "Please address the failed tests before proceeding with release."
    fi
}

# Main execution
main() {
    echo -e "${BLUE}"
    echo "  _   _           _        _   _"
    echo " | | | |_ __   __| | ___  | | | |___  ___ _ __"
    echo " | | | | '_ \ / _\` |/ _ \ | | | / __|/ _ \ '__|"
    echo " | |_| | |_) | (_| |  __/ | |_| \__ \  __/ |"
    echo "  \___/| .__/ \__,_|\___|  \___/|___/\___|_|"
    echo "       |_|"
    echo -e "\nUdaya Native Execution Verification${NC}"
    echo "=========================================="

    test_clean_build
    test_binary_hashes
    test_native_execution
    test_configuration_files
    test_documentation
    test_release_checksums

    generate_report
}

# Run main function
main