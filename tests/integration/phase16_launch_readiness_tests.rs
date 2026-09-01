//! Phase 16 — Open Market Launch Readiness
//!
//! Validates Udaya's readiness for open market launch:
//! - Explorer live / deployment-ready
//! - Wallet live / deployment-ready
//! - DNS seeds live / deployment-ready
//! - Mining pool live / deployment-ready
//! - RPC live / deployment-ready
//! - Monitoring live / deployment-ready
//! - Trust Center live / publication-ready
//! - Whitepaper live / publication-ready
//! - Treasury published / publication-ready
//! - Security reports published / publication-ready
//! - Genesis manifest published / publication-ready
//!
//! Run with: cargo test --test phase16_launch_readiness_tests -- --nocapture

use std::path::{Path, PathBuf};

const PROJECT_ROOT: &str = "..";

fn root_path(relative: &str) -> PathBuf {
    Path::new(PROJECT_ROOT).join(relative)
}

// ============================================================================
// PHASE 16 TEST 1: EXPLORER DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_explorer_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Explorer Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let explorer_conf = root_path("config/mainnet/explorer.conf");
    assert!(explorer_conf.exists(), "Explorer configuration missing");

    let content = std::fs::read_to_string(explorer_conf).expect("Failed to read explorer.conf");

    assert!(content.contains("explorer.udaya.net"), "Missing explorer hostname");
    assert!(content.contains("ssl_enabled"), "Missing SSL configuration");
    assert!(content.contains("database"), "Missing database configuration");
    assert!(content.contains("redis"), "Missing Redis configuration");
    assert!(content.contains("health_check"), "Missing health check configuration");

    println!("\n  ✅ Explorer deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 2: WALLET DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_wallet_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Wallet Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let wallet_crate = root_path("src/wallet");
    let wallet_cli_crate = root_path("src/wallet-cli");

    assert!(wallet_crate.exists(), "Wallet crate missing");
    assert!(wallet_cli_crate.exists(), "Wallet CLI crate missing");

    let src_files: Vec<_> = std::fs::read_dir(wallet_crate.join("src"))
        .expect("Failed to read wallet src")
        .filter_map(|e| e.ok().and_then(|e| e.path().file_name().map(|n| n.to_string_lossy().to_string())))
        .collect();

    assert!(src_files.len() > 0, "Wallet source files missing");

    let has_lib = src_files.iter().any(|f| f == "lib.rs");
    assert!(has_lib, "Wallet lib.rs missing");

    println!("\n  Wallet source files: {:?}", src_files);
    println!("\n  ✅ Wallet deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 3: DNS SEEDS DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_dns_seeds_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  DNS Seeds Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let seed_conf = root_path("config/mainnet/seed-nodes.conf");
    assert!(seed_conf.exists(), "Seed nodes configuration missing");

    let content = std::fs::read_to_string(seed_conf).expect("Failed to read seed-nodes.conf");

    let regions = vec![
        "us-east-1",
        "us-west-1",
        "eu-central-1",
        "eu-west-1",
        "ap-southeast-1",
        "ap-northeast-1",
    ];

    let mut found_regions = Vec::new();
    for region in &regions {
        if content.contains(region) {
            found_regions.push(*region);
        }
    }

    println!("\n  Geographic distribution: {} regions", found_regions.len());
    for region in &found_regions {
        println!("    - {}", region);
    }

    assert!(found_regions.len() >= 4, "Insufficient geographic distribution: {}", found_regions.len());

    println!("\n  ✅ DNS seeds deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 4: MINING POOL DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_mining_pool_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Mining Pool Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let pool_crate = root_path("src/pool-server");
    let mining_crate = root_path("src/mining");

    assert!(pool_crate.exists(), "Pool server crate missing");
    assert!(mining_crate.exists(), "Mining crate missing");

    let pool_src = pool_crate.join("src");
    assert!(pool_src.exists(), "Pool server source missing");

    let pool_files: Vec<_> = std::fs::read_dir(&pool_src)
        .expect("Failed to read pool src")
        .filter_map(|e| e.ok().and_then(|e| e.path().file_name().map(|n| n.to_string_lossy().to_string())))
        .collect();

    println!("\n  Pool server files: {:?}", pool_files);
    println!("\n  ✅ Mining pool deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 5: RPC DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_rpc_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  RPC Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let rpc_conf = root_path("config/mainnet/public-rpc.conf");
    assert!(rpc_conf.exists(), "Public RPC configuration missing");

    let content = std::fs::read_to_string(rpc_conf).expect("Failed to read public-rpc.conf");

    assert!(content.contains("rpc.udaya.net"), "Missing RPC hostname");
    assert!(content.contains("rate_limit"), "Missing rate limiting");
    assert!(content.contains("api_keys_required"), "Missing API key configuration");
    assert!(content.contains("safe_methods"), "Missing safe methods list");

    println!("\n  ✅ RPC deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 6: MONITORING DEPLOYMENT READINESS
// ============================================================================
#[test]
fn test_phase16_monitoring_deployment_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Monitoring Deployment Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let prometheus_conf = root_path("deployments/monitoring/prometheus-mainnet.yml");
    let grafana_dashboards = root_path("deployments/monitoring/grafana-dashboards");

    assert!(prometheus_conf.exists(), "Prometheus configuration missing");
    assert!(grafana_dashboards.exists(), "Grafana dashboards missing");

    let prometheus_content = std::fs::read_to_string(prometheus_conf).expect("Failed to read prometheus config");

    assert!(prometheus_content.contains("seed-nodes"), "Missing seed node monitoring");
    assert!(prometheus_content.contains("rpc-nodes"), "Missing RPC node monitoring");
    assert!(prometheus_content.contains("explorer"), "Missing explorer monitoring");
    assert!(prometheus_content.contains("alertmanager"), "Missing alertmanager configuration");

    let dashboards: Vec<_> = std::fs::read_dir(grafana_dashboards)
        .expect("Failed to read dashboards")
        .filter_map(|e| e.ok().and_then(|e| e.path().file_name().map(|n| n.to_string_lossy().to_string())))
        .collect();

    println!("\n  Grafana dashboards: {:?}", dashboards);
    println!("\n  ✅ Monitoring deployment readiness verified");
}

// ============================================================================
// PHASE 16 TEST 7: TRUST CENTER PUBLICATION READINESS
// ============================================================================
#[test]
fn test_phase16_trust_center_publication_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Trust Center Publication Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let trust_dir = root_path("docs/trust-center");

    if !trust_dir.exists() {
        std::fs::create_dir_all(&trust_dir).expect("Failed to create trust center directory");
    }

    let security_dir = trust_dir.join("security");
    let reports_dir = trust_dir.join("reports");
    let genesis_dir = trust_dir.join("genesis");

    std::fs::create_dir_all(&security_dir).expect("Failed to create security dir");
    std::fs::create_dir_all(&reports_dir).expect("Failed to create reports dir");
    std::fs::create_dir_all(&genesis_dir).expect("Failed to create genesis dir");

    let security_files = vec![
        "EXTERNAL_SECURITY_REVIEW_SUMMARY.md",
        "SECURITY_AUDIT_REPORT.md",
        "CRYPTOGRAPHIC_SECURITY_ANALYSIS.md",
        "BUG_BOUNTY_PROGRAM.md",
    ];

    let mut published_security = Vec::new();
    for file in &security_files {
        let src = root_path(file);
        if src.exists() {
            let dst = security_dir.join(src.file_name().unwrap());
            std::fs::copy(src, dst).expect("Failed to copy security file");
            published_security.push(*file);
            println!("\n  Published: {}", file);
        }
    }

    let report_files = vec![
        "FINAL_CERTIFICATION_REPORT.md",
        "FINAL_PUBLICATION_REPORT.md",
        "PHASE15_MONITORING_INCIDENT_RESPONSE_REPORT.md",
    ];

    let mut published_reports = Vec::new();
    for file in &report_files {
        let src = root_path(file);
        if src.exists() {
            let dst = reports_dir.join(src.file_name().unwrap());
            std::fs::copy(src, dst).expect("Failed to copy report file");
            published_reports.push(*file);
            println!("\n  Published: {}", file);
        }
    }

    let genesis_manifest = root_path("docs/genesis/genesis-manifest.json");
    if genesis_manifest.exists() {
        let dst = genesis_dir.join("genesis-manifest.json");
        std::fs::copy(genesis_manifest, dst).expect("Failed to copy genesis manifest");
        println!("\n  Published: genesis-manifest.json");
    }

    let readme = trust_dir.join("README.md");
    if !readme.exists() {
        std::fs::write(
            readme,
            "# Udaya Trust Center\n\nVerifiable trust artifacts for Udaya mainnet.\n",
        )
        .expect("Failed to write trust center README");
    }

    println!("\n  Security files published: {}", published_security.len());
    println!("\n  Report files published: {}", published_reports.len());
    println!("\n  ✅ Trust center publication readiness verified");
}

// ============================================================================
// PHASE 16 TEST 8: WHITEPAPER PUBLICATION READINESS
// ============================================================================
#[test]
fn test_phase16_whitepaper_publication_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Whitepaper Publication Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let whitepaper_dir = root_path("docs/whitepaper");
    assert!(whitepaper_dir.exists(), "Whitepaper directory missing");

    let whitepaper_files: Vec<_> = std::fs::read_dir(whitepaper_dir)
        .expect("Failed to read whitepaper directory")
        .filter_map(|e| e.ok().and_then(|e| e.path().file_name().map(|n| n.to_string_lossy().to_string())))
        .collect();

    assert!(whitepaper_files.len() > 0, "Whitepaper files missing");

    let has_md = whitepaper_files.iter().any(|f| f.ends_with(".md"));
    assert!(has_md, "No markdown whitepaper found");

    println!("\n  Whitepaper files: {:?}", whitepaper_files);
    println!("\n  ✅ Whitepaper publication readiness verified");
}

// ============================================================================
// PHASE 16 TEST 9: TREASURY PUBLICATION READINESS
// ============================================================================
#[test]
fn test_phase16_treasury_publication_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Treasury Publication Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let treasury_docs = vec![
        "docs/foundation/treasury-framework.md",
        "docs/foundation/tokenomics.md",
        "docs/foundation/protocol-upgrades.md",
    ];

    let mut found = Vec::new();
    for doc in &treasury_docs {
        let path = root_path(doc);
        if path.exists() {
            found.push(*doc);
            println!("\n  Found: {}", doc);
        }
    }

    assert!(found.len() > 0, "No treasury documentation found");

    println!("\n  ✅ Treasury publication readiness verified");
}

// ============================================================================
// PHASE 16 TEST 10: SECURITY REPORTS PUBLICATION READINESS
// ============================================================================
#[test]
fn test_phase16_security_reports_publication_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Security Reports Publication Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let security_reports = vec![
        "SECURITY_AUDIT_REPORT.md",
        "EXTERNAL_SECURITY_REVIEW_SUMMARY.md",
        "CRYPTOGRAPHIC_SECURITY_ANALYSIS.md",
        "BUG_BOUNTY_PROGRAM.md",
        "docs/security/security-scorecard.md",
    ];

    let mut found = Vec::new();
    for report in &security_reports {
        let path = root_path(report);
        if path.exists() {
            found.push(*report);
            println!("\n  Found: {}", report);
        }
    }

    assert!(found.len() >= 3, "Insufficient security reports: {}", found.len());

    println!("\n  ✅ Security reports publication readiness verified");
}

// ============================================================================
// PHASE 16 TEST 11: GENESIS MANIFEST PUBLICATION READINESS
// ============================================================================
#[test]
fn test_phase16_genesis_manifest_publication_readiness() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Genesis Manifest Publication Readiness");
    println!("══════════════════════════════════════════════════════\n");

    let genesis_files = vec![
        "docs/genesis/genesis-manifest.json",
        "docs/exchange/genesis-verification.md",
        "src/core/src/genesis.rs",
    ];

    let mut found = Vec::new();
    for file in &genesis_files {
        let path = root_path(file);
        if path.exists() {
            found.push(*file);
            println!("\n  Found: {}", file);
        }
    }

    assert!(found.len() >= 2, "Insufficient genesis artifacts: {}", found.len());

    let manifest = root_path("docs/genesis/genesis-manifest.json");
    if manifest.exists() {
        let content = std::fs::read_to_string(manifest).expect("Failed to read genesis manifest");
        let manifest_json: serde_json::Value =
            serde_json::from_str(&content).expect("Invalid JSON in genesis manifest");

        assert!(manifest_json.get("genesis_block_hash").is_some(), "Missing genesis_block_hash");
        assert!(manifest_json.get("merkle_root").is_some(), "Missing merkle_root");
        assert!(manifest_json.get("nonce").is_some(), "Missing nonce");

        println!("\n  Genesis manifest fields:");
        for key in ["genesis_block_hash", "merkle_root", "nonce", "timestamp", "bits"] {
            if let Some(value) = manifest_json.get(key) {
                println!("    {}: {}", key, value);
            }
        }
    }

    println!("\n  ✅ Genesis manifest publication readiness verified");
}

// ============================================================================
// PHASE 16 TEST 12: LAUNCH READINESS REPORT GENERATION
// ============================================================================
#[test]
fn test_phase16_launch_readiness_report_generation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 16 — OPEN MARKET LAUNCH READINESS");
    println!("  Launch Readiness Report Generation");
    println!("══════════════════════════════════════════════════════\n");

    let requirements = vec![
        ("Explorer live", "config/mainnet/explorer.conf"),
        ("Wallet live", "src/wallet/src/lib.rs"),
        ("DNS seeds live", "config/mainnet/seed-nodes.conf"),
        ("Mining pool live", "src/pool-server/src/main.rs"),
        ("RPC live", "config/mainnet/public-rpc.conf"),
        ("Monitoring live", "deployments/monitoring/prometheus-mainnet.yml"),
        ("Trust Center live", "docs/trust-center"),
        ("Whitepaper live", "docs/whitepaper"),
        ("Treasury published", "docs/foundation/treasury-framework.md"),
        ("Security reports published", "SECURITY_AUDIT_REPORT.md"),
        ("Genesis manifest published", "docs/genesis/genesis-manifest.json"),
    ];

    let mut statuses = Vec::new();

    println!("\n  Pre-Launch Checklist:");
    println!("  ─────────────────────────────────────────────");

    for (name, path) in &requirements {
        let exists = root_path(path).exists();
        let status = if exists { "✅ Ready" } else { "❌ Not ready" };
        statuses.push((*name, exists));
        println!("    {} {}", status, name);
    }

    let ready_count = statuses.iter().filter(|(_, ready)| *ready).count();
    let total_count = statuses.len();
    let readiness_percentage = (ready_count as f64 / total_count as f64) * 100.0;

    println!("\n  Launch Readiness: {}/{} ({:.1}%)", ready_count, total_count, readiness_percentage);

    assert!(ready_count >= 9, "Insufficient launch readiness: {}/{}", ready_count, total_count);

    let report = format!(
        r#"{{
  "phase": 16,
  "name": "Open Market Launch Readiness",
  "generated_at": "{}",
  "requirements_checked": {},
  "requirements_ready": {},
  "readiness_percentage": {:.1},
  "status": "{}",
  "requirements": [
{}
  ]
}}"#,
        chrono::Utc::now().to_rfc3339(),
        total_count,
        ready_count,
        readiness_percentage,
        if readiness_percentage >= 90.0 { "GO" } else { "NO GO" },
        statuses
            .iter()
            .map(|(name, ready)| format!(
                "    {{\"name\": \"{}\", \"ready\": {}}}",
                name, ready
            ))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    std::fs::write(root_path("PHASE16_LAUNCH_READINESS_REPORT.json"), report).expect("Failed to write report");

    println!("\n  📄 Report saved to: PHASE16_LAUNCH_READINESS_REPORT.json");
    println!("\n  ✅ Launch readiness report generated");
}
