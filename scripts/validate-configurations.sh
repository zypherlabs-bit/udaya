#!/bin/bash

# Udaya Blockchain - Phase 5 Configuration Validation Script
# Comprehensive testing and validation of all mainnet configurations

set -euo pipefail

# Configuration Validation Script
# This script validates all Phase 5 configurations for Udaya mainnet deployment

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test counters
total_tests=0
passed_tests=0
failed_tests=0

# Configuration file paths
SEED_NODES_CONFIG="config/mainnet/seed-nodes.conf"
PUBLIC_RPC_CONFIG="config/mainnet/public-rpc.conf"
EXPLORER_CONFIG="config/mainnet/explorer.conf"
FAUCET_CONFIG="config/mainnet/faucet.conf"
MONITORING_CONFIG="deployments/monitoring/prometheus-mainnet.yml"

# Test validation functions
test_file_exists() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        log_success "$name configuration file exists"
        ((passed_tests++))
    else
        log_error "$name configuration file missing"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_file_syntax() {
    local file=$1
    local name=$2
    local syntax_checker=$3

    if [ -f "$file" ]; then
        if $syntax_checker "$file" 2>/dev/null; then
            log_success "$name configuration syntax is valid"
            ((passed_tests++))
        else
            log_error "$name configuration syntax is invalid"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for syntax check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_required_sections() {
    local file=$1
    local name=$2
    local required_sections=("${@:3}")

    if [ -f "$file" ]; then
        missing_sections=()
        for section in "${required_sections[@]}"; do
            if ! grep -q "\[$section\]" "$file"; then
                missing_sections+=("$section")
            fi
        done

        if [ ${#missing_sections[@]} -eq 0 ]; then
            log_success "$name contains all required sections"
            ((passed_tests++))
        else
            log_error "$name missing sections: ${missing_sections[*]}"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for section check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_network_configuration() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        # Check for network-related configurations
        if grep -q "network.*mainnet" "$file" && \
           grep -q "protocol_version" "$file" && \
           grep -q "port.*9798\|port.*8332" "$file"; then
            log_success "$name network configuration is valid"
            ((passed_tests++))
        else
            log_error "$name network configuration is incomplete"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for network check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_security_configuration() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        # Check for security-related configurations
        if grep -q "tls_enabled.*true" "$file" || \
           grep -q "ssl_enabled.*true" "$file" || \
           grep -q "rate_limit" "$file"; then
            log_success "$name security configuration is valid"
            ((passed_tests++))
        else
            log_error "$name security configuration is incomplete"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for security check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_monitoring_configuration() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        # Check for monitoring-related configurations
        if grep -q "prometheus\|grafana\|alertmanager" "$file" && \
           grep -q "metrics_path" "$file"; then
            log_success "$name monitoring configuration is valid"
            ((passed_tests++))
        else
            log_error "$name monitoring configuration is incomplete"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for monitoring check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_backup_configuration() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        # Check for backup-related configurations
        if grep -q "backup\|restore" "$file" || \
           grep -q "retention" "$file"; then
            log_success "$name backup configuration is valid"
            ((passed_tests++))
        else
            log_error "$name backup configuration is incomplete"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for backup check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_geographic_distribution() {
    local file=$1
    local name=$2

    if [ -f "$file" ]; then
        # Check for geographic distribution in seed nodes
        regions_found=0
        for region in "us-east" "us-west" "eu-central" "eu-west" "ap-southeast" "ap-northeast"; do
            if grep -q "$region" "$file"; then
                ((regions_found++))
            fi
        done

        if [ $regions_found -ge 4 ]; then
            log_success "$name has proper geographic distribution ($regions_found regions)"
            ((passed_tests++))
        else
            log_error "$name geographic distribution is insufficient ($regions_found regions)"
            ((failed_tests++))
        fi
    else
        log_error "$name configuration file missing for geographic check"
        ((failed_tests++))
    fi
    ((total_tests++))
}

test_prometheus_configuration() {
    local file=$1

    if [ -f "$file" ]; then
        # Check for essential Prometheus configurations
        if grep -q "global:" "$file" && \
           grep -q "scrape_configs:" "$file" && \
           grep -q "alerting:" "$file" && \
           grep -q "rule_files:" "$file"; then
            log_success "Prometheus configuration structure is valid"
            ((passed_tests++))
        else
            log_error "Prometheus configuration structure is incomplete"
            ((failed_tests++))
        fi

        # Check for seed node monitoring
        if grep -q "udaya-mainnet-seed-nodes" "$file"; then
            log_success "Prometheus includes seed node monitoring"
            ((passed_tests++))
        else
            log_error "Prometheus missing seed node monitoring"
            ((failed_tests++))
        fi

        # Check for RPC node monitoring
        if grep -q "udaya-mainnet-rpc-nodes" "$file"; then
            log_success "Prometheus includes RPC node monitoring"
            ((passed_tests++))
        else
            log_error "Prometheus missing RPC node monitoring"
            ((failed_tests++))
        fi

        # Check for explorer monitoring
        if grep -q "udaya-mainnet-explorer" "$file"; then
            log_success "Prometheus includes explorer monitoring"
            ((passed_tests++))
        else
            log_error "Prometheus missing explorer monitoring"
            ((failed_tests++))
        fi

        # Check for faucet monitoring
        if grep -q "udaya-testnet-faucet" "$file"; then
            log_success "Prometheus includes faucet monitoring"
            ((passed_tests++))
        else
            log_error "Prometheus missing faucet monitoring"
            ((failed_tests++))
        fi
    else
        log_error "Prometheus configuration file missing"
        ((failed_tests++))
    fi
}

# Main validation function
validate_all_configurations() {
    log_info "Starting Udaya Phase 5 Configuration Validation"
    log_info "==============================================="

    # Test file existence
    log_info "\n1. Testing configuration file existence..."
    test_file_exists "$SEED_NODES_CONFIG" "Seed Nodes"
    test_file_exists "$PUBLIC_RPC_CONFIG" "Public RPC"
    test_file_exists "$EXPLORER_CONFIG" "Explorer"
    test_file_exists "$FAUCET_CONFIG" "Faucet"
    test_file_exists "$MONITORING_CONFIG" "Monitoring"

    # Test configuration syntax (basic checks)
    log_info "\n2. Testing configuration syntax..."
    test_file_syntax "$SEED_NODES_CONFIG" "Seed Nodes" "grep -q '^#\|^\[.*\]\|^[a-z_].*='"
    test_file_syntax "$PUBLIC_RPC_CONFIG" "Public RPC" "grep -q '^#\|^\[.*\]\|^[a-z_].*='"
    test_file_syntax "$EXPLORER_CONFIG" "Explorer" "grep -q '^#\|^\[.*\]\|^[a-z_].*='"
    test_file_syntax "$FAUCET_CONFIG" "Faucet" "grep -q '^#\|^\[.*\]\|^[a-z_].*='"
    test_file_syntax "$MONITORING_CONFIG" "Monitoring" "grep -q '^#\|^[a-z_].*:'"

    # Test required sections
    log_info "\n3. Testing required configuration sections..."
    test_required_sections "$SEED_NODES_CONFIG" "Seed Nodes" "global" "seed_nodes" "dns_seeds" "monitoring" "security"
    test_required_sections "$PUBLIC_RPC_CONFIG" "Public RPC" "global" "rpc_node" "rpc" "security" "monitoring"
    test_required_sections "$EXPLORER_CONFIG" "Explorer" "global" "explorer" "database" "api" "monitoring"
    test_required_sections "$FAUCET_CONFIG" "Faucet" "global" "faucet" "database" "rate_limiting" "monitoring"

    # Test network configurations
    log_info "\n4. Testing network configurations..."
    test_network_configuration "$SEED_NODES_CONFIG" "Seed Nodes"
    test_network_configuration "$PUBLIC_RPC_CONFIG" "Public RPC"
    test_network_configuration "$EXPLORER_CONFIG" "Explorer"
    test_network_configuration "$FAUCET_CONFIG" "Faucet"

    # Test security configurations
    log_info "\n5. Testing security configurations..."
    test_security_configuration "$SEED_NODES_CONFIG" "Seed Nodes"
    test_security_configuration "$PUBLIC_RPC_CONFIG" "Public RPC"
    test_security_configuration "$EXPLORER_CONFIG" "Explorer"
    test_security_configuration "$FAUCET_CONFIG" "Faucet"

    # Test monitoring configurations
    log_info "\n6. Testing monitoring configurations..."
    test_monitoring_configuration "$SEED_NODES_CONFIG" "Seed Nodes"
    test_monitoring_configuration "$PUBLIC_RPC_CONFIG" "Public RPC"
    test_monitoring_configuration "$EXPLORER_CONFIG" "Explorer"
    test_monitoring_configuration "$FAUCET_CONFIG" "Faucet"

    # Test backup configurations
    log_info "\n7. Testing backup configurations..."
    test_backup_configuration "$SEED_NODES_CONFIG" "Seed Nodes"
    test_backup_configuration "$PUBLIC_RPC_CONFIG" "Public RPC"
    test_backup_configuration "$EXPLORER_CONFIG" "Explorer"
    test_backup_configuration "$FAUCET_CONFIG" "Faucet"

    # Test geographic distribution
    log_info "\n8. Testing geographic distribution..."
    test_geographic_distribution "$SEED_NODES_CONFIG" "Seed Nodes"

    # Test Prometheus configuration specifically
    log_info "\n9. Testing Prometheus configuration..."
    test_prometheus_configuration "$MONITORING_CONFIG"

    # Summary
    log_info "\n==============================================="
    log_info "Configuration Validation Summary"
    log_info "==============================================="
    log_info "Total tests run: $total_tests"
    log_success "Passed tests: $passed_tests"
    log_error "Failed tests: $failed_tests"

    if [ $failed_tests -eq 0 ]; then
        log_success "\n🎉 All configuration validations passed!"
        log_success "Phase 5 configurations are ready for deployment"
        return 0
    else
        log_error "\n❌ Configuration validation failed!"
        log_error "Please review the failed tests above"
        return 1
    fi
}

# Run validation
validate_all_configurations

# Exit with appropriate status
if [ $failed_tests -eq 0 ]; then
    exit 0
else
    exit 1
fi