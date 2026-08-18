<#
.Udaya Blockchain - Phase 5 Configuration Validation Script
Comprehensive testing and validation of all mainnet configurations
#>

# Configuration Validation Script
# This script validates all Phase 5 configurations for Udaya mainnet deployment

# Colors for output
$RED = [char]27 + "[31m"
$GREEN = [char]27 + "[32m"
$YELLOW = [char]27 + "[33m"
$BLUE = [char]27 + "[34m"
$NC = [char]27 + "[0m"

# Logging functions
function log-info {
    param($message)
    Write-Host "$BLUE[INFO]$NC $message"
}

function log-success {
    param($message)
    Write-Host "$GREEN[SUCCESS]$NC $message"
}

function log-warning {
    param($message)
    Write-Host "$YELLOW[WARNING]$NC $message"
}

function log-error {
    param($message)
    Write-Host "$RED[ERROR]$NC $message"
}

# Test counters
$totalTests = 0
$passedTests = 0
$failedTests = 0

# Configuration file paths
$SEED_NODES_CONFIG = "config/mainnet/seed-nodes.conf"
$PUBLIC_RPC_CONFIG = "config/mainnet/public-rpc.conf"
$EXPLORER_CONFIG = "config/mainnet/explorer.conf"
$FAUCET_CONFIG = "config/mainnet/faucet.conf"
$MONITORING_CONFIG = "deployments/monitoring/prometheus-mainnet.yml"

# Test validation functions
function test-file-exists {
    param($file, $name)

    if (Test-Path $file) {
        log-success "$name configuration file exists"
        $passedTests++
    } else {
        log-error "$name configuration file missing"
        $failedTests++
    }
    $totalTests++
}

function test-file-syntax {
    param($file, $name, $pattern)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        if ($content -match $pattern) {
            log-success "$name configuration syntax is valid"
            $passedTests++
        } else {
            log-error "$name configuration syntax is invalid"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for syntax check"
        $failedTests++
    }
    $totalTests++
}

function test-required-sections {
    param($file, $name, $requiredSections)

    if (Test-Path $file) {
        $missingSections = @()
        $content = Get-Content $file -Raw

        foreach ($section in $requiredSections) {
            if ($content -notmatch "\[$section\]") {
                $missingSections += $section
            }
        }

        if ($missingSections.Count -eq 0) {
            log-success "$name contains all required sections"
            $passedTests++
        } else {
            $missingList = $missingSections -join ", "
            log-error "$name missing sections: $missingList"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for section check"
        $failedTests++
    }
    $totalTests++
}

function test-network-configuration {
    param($file, $name)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        $hasNetwork = $content -match "network.*mainnet|network.*testnet"
        $hasProtocol = $content -match "protocol_version"
        $hasPort = $content -match "port.*9798|port.*8332|port.*8081"

        if ($hasNetwork -and $hasProtocol -and $hasPort) {
            log-success "$name network configuration is valid"
            $passedTests++
        } else {
            log-error "$name network configuration is incomplete"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for network check"
        $failedTests++
    }
    $totalTests++
}

function test-security-configuration {
    param($file, $name)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        $hasTLS = $content -match "tls_enabled.*true"
        $hasSSL = $content -match "ssl_enabled.*true"
        $hasRateLimit = $content -match "rate_limit"

        if ($hasTLS -or $hasSSL -or $hasRateLimit) {
            log-success "$name security configuration is valid"
            $passedTests++
        } else {
            log-error "$name security configuration is incomplete"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for security check"
        $failedTests++
    }
    $totalTests++
}

function test-monitoring-configuration {
    param($file, $name)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        $hasMonitoring = $content -match "prometheus|grafana|alertmanager"
        $hasMetrics = $content -match "metrics_path"

        if ($hasMonitoring -and $hasMetrics) {
            log-success "$name monitoring configuration is valid"
            $passedTests++
        } else {
            log-error "$name monitoring configuration is incomplete"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for monitoring check"
        $failedTests++
    }
    $totalTests++
}

function test-backup-configuration {
    param($file, $name)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        $hasBackup = $content -match "backup|restore"
        $hasRetention = $content -match "retention"

        if ($hasBackup -or $hasRetention) {
            log-success "$name backup configuration is valid"
            $passedTests++
        } else {
            log-error "$name backup configuration is incomplete"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for backup check"
        $failedTests++
    }
    $totalTests++
}

function test-geographic-distribution {
    param($file, $name)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        $regions = @("us-east", "us-west", "eu-central", "eu-west", "ap-southeast", "ap-northeast")
        $regionsFound = 0

        foreach ($region in $regions) {
            if ($content -match $region) {
                $regionsFound++
            }
        }

        if ($regionsFound -ge 4) {
            log-success "$name has proper geographic distribution ($regionsFound regions)"
            $passedTests++
        } else {
            log-error "$name geographic distribution is insufficient ($regionsFound regions)"
            $failedTests++
        }
    } else {
        log-error "$name configuration file missing for geographic check"
        $failedTests++
    }
    $totalTests++
}

function test-prometheus-configuration {
    param($file)

    if (Test-Path $file) {
        $content = Get-Content $file -Raw

        # Check for essential Prometheus configurations
        $hasGlobal = $content -match "global:"
        $hasScrapeConfigs = $content -match "scrape_configs:"
        $hasAlerting = $content -match "alerting:"
        $hasRuleFiles = $content -match "rule_files:"

        if ($hasGlobal -and $hasScrapeConfigs -and $hasAlerting -and $hasRuleFiles) {
            log-success "Prometheus configuration structure is valid"
            $passedTests++
        } else {
            log-error "Prometheus configuration structure is incomplete"
            $failedTests++
        }

        # Check for seed node monitoring
        if ($content -match "udaya-mainnet-seed-nodes") {
            log-success "Prometheus includes seed node monitoring"
            $passedTests++
        } else {
            log-error "Prometheus missing seed node monitoring"
            $failedTests++
        }

        # Check for RPC node monitoring
        if ($content -match "udaya-mainnet-rpc-nodes") {
            log-success "Prometheus includes RPC node monitoring"
            $passedTests++
        } else {
            log-error "Prometheus missing RPC node monitoring"
            $failedTests++
        }

        # Check for explorer monitoring
        if ($content -match "udaya-mainnet-explorer") {
            log-success "Prometheus includes explorer monitoring"
            $passedTests++
        } else {
            log-error "Prometheus missing explorer monitoring"
            $failedTests++
        }

        # Check for faucet monitoring
        if ($content -match "udaya-testnet-faucet") {
            log-success "Prometheus includes faucet monitoring"
            $passedTests++
        } else {
            log-error "Prometheus missing faucet monitoring"
            $failedTests++
        }
    } else {
        log-error "Prometheus configuration file missing"
        $failedTests++
    }
}

# Main validation function
function validate-all-configurations {
    log-info "Starting Udaya Phase 5 Configuration Validation"
    log-info "==============================================="

    # Test file existence
    log-info "`n1. Testing configuration file existence..."
    test-file-exists $SEED_NODES_CONFIG "Seed Nodes"
    test-file-exists $PUBLIC_RPC_CONFIG "Public RPC"
    test-file-exists $EXPLORER_CONFIG "Explorer"
    test-file-exists $FAUCET_CONFIG "Faucet"
    test-file-exists $MONITORING_CONFIG "Monitoring"

    # Test configuration syntax (basic checks)
    log-info "`n2. Testing configuration syntax..."
    test-file-syntax $SEED_NODES_CONFIG "Seed Nodes" "^#|^\[.*\]|^[a-z_].*="
    test-file-syntax $PUBLIC_RPC_CONFIG "Public RPC" "^#|^\[.*\]|^[a-z_].*="
    test-file-syntax $EXPLORER_CONFIG "Explorer" "^#|^\[.*\]|^[a-z_].*="
    test-file-syntax $FAUCET_CONFIG "Faucet" "^#|^\[.*\]|^[a-z_].*="
    test-file-syntax $MONITORING_CONFIG "Monitoring" "^#|^[a-z_].*:"

    # Test required sections
    log-info "`n3. Testing required configuration sections..."
    test-required-sections $SEED_NODES_CONFIG "Seed Nodes" @("global", "seed_nodes", "dns_seeds", "monitoring", "security")
    test-required-sections $PUBLIC_RPC_CONFIG "Public RPC" @("global", "rpc_node", "rpc", "security", "monitoring")
    test-required-sections $EXPLORER_CONFIG "Explorer" @("global", "explorer", "database", "api", "monitoring")
    test-required-sections $FAUCET_CONFIG "Faucet" @("global", "faucet", "database", "rate_limiting", "monitoring")

    # Test network configurations
    log-info "`n4. Testing network configurations..."
    test-network-configuration $SEED_NODES_CONFIG "Seed Nodes"
    test-network-configuration $PUBLIC_RPC_CONFIG "Public RPC"
    test-network-configuration $EXPLORER_CONFIG "Explorer"
    test-network-configuration $FAUCET_CONFIG "Faucet"

    # Test security configurations
    log-info "`n5. Testing security configurations..."
    test-security-configuration $SEED_NODES_CONFIG "Seed Nodes"
    test-security-configuration $PUBLIC_RPC_CONFIG "Public RPC"
    test-security-configuration $EXPLORER_CONFIG "Explorer"
    test-security-configuration $FAUCET_CONFIG "Faucet"

    # Test monitoring configurations
    log-info "`n6. Testing monitoring configurations..."
    test-monitoring-configuration $SEED_NODES_CONFIG "Seed Nodes"
    test-monitoring-configuration $PUBLIC_RPC_CONFIG "Public RPC"
    test-monitoring-configuration $EXPLORER_CONFIG "Explorer"
    test-monitoring-configuration $FAUCET_CONFIG "Faucet"

    # Test backup configurations
    log-info "`n7. Testing backup configurations..."
    test-backup-configuration $SEED_NODES_CONFIG "Seed Nodes"
    test-backup-configuration $PUBLIC_RPC_CONFIG "Public RPC"
    test-backup-configuration $EXPLORER_CONFIG "Explorer"
    test-backup-configuration $FAUCET_CONFIG "Faucet"

    # Test geographic distribution
    log-info "`n8. Testing geographic distribution..."
    test-geographic-distribution $SEED_NODES_CONFIG "Seed Nodes"

    # Test Prometheus configuration specifically
    log-info "`n9. Testing Prometheus configuration..."
    test-prometheus-configuration $MONITORING_CONFIG

    # Summary
    log-info "`n==============================================="
    log-info "Configuration Validation Summary"
    log-info "==============================================="
    log-info "Total tests run: $totalTests"
    log-success "Passed tests: $passedTests"
    log-error "Failed tests: $failedTests"

    if ($failedTests -eq 0) {
        log-success "`n🎉 All configuration validations passed!"
        log-success "Phase 5 configurations are ready for deployment"
        return $true
    } else {
        log-error "`n❌ Configuration validation failed!"
        log-error "Please review the failed tests above"
        return $false
    }
}

# Run validation
$validationResult = validate-all-configurations

# Exit with appropriate status
if ($validationResult) {
    exit 0
} else {
    exit 1
}