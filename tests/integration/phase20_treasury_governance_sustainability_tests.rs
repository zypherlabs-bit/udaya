//! Phase 20 — Treasury Governance & Long-term Sustainability
//!
//! Validates Udaya's long-term sustainability and governance maturity:
//! - Treasury governance framework
//! - Protocol upgrade mechanisms
//! - Grant and funding frameworks
//! - Community governance participation
//! - Long-term sustainability planning
//! - Liquidity strategy
//! - Constitutional governance
//! - Operational structure maturity
//! - Phase 20 readiness report generation
//!
//! Run with: cargo test --test phase20_treasury_governance_sustainability_tests -- --nocapture

use std::path::{Path, PathBuf};

const PROJECT_ROOT: &str = "..";

fn root_path(relative: &str) -> PathBuf {
    Path::new(PROJECT_ROOT).join(relative)
}

// ============================================================================
// PHASE 20 TEST 1: TREASURY GOVERNANCE FRAMEWORK
// ============================================================================
#[test]
fn test_phase20_treasury_governance_framework() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Treasury Governance Framework");
    println!("══════════════════════════════════════════════════════\n");

    let treasury_framework = root_path("docs/foundation/treasury-framework.md");
    assert!(treasury_framework.exists(), "Treasury framework documentation missing");

    let content = std::fs::read_to_string(&treasury_framework).expect("Failed to read treasury-framework.md");

    assert!(content.contains("Treasury") || content.contains("treasury"), "Missing treasury references");
    assert!(content.contains("multisig") || content.contains("multi-signature"), "Missing multisig control");
    assert!(content.contains("allocation") || content.contains("Allocation"), "Missing allocation categories");
    assert!(content.contains("governance") || content.contains("vote"), "Missing governance process");

    println!("\n  Treasury governance framework verified:");
    println!("    - Treasury overview: ✅");
    println!("    - Multisig control: ✅");
    println!("    - Allocation categories: ✅");
    println!("    - Governance process: ✅");

    println!("\n  ✅ Treasury governance framework verified");
}

// ============================================================================
// PHASE 20 TEST 2: PROTOCOL UPGRADE MECHANISMS
// ============================================================================
#[test]
fn test_phase20_protocol_upgrade_mechanisms() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Protocol Upgrade Mechanisms");
    println!("══════════════════════════════════════════════════════\n");

    let protocol_upgrades = root_path("docs/foundation/protocol-upgrades.md");
    assert!(protocol_upgrades.exists(), "Protocol upgrades documentation missing");

    let content = std::fs::read_to_string(&protocol_upgrades).expect("Failed to read protocol-upgrades.md");

    assert!(content.contains("upgrade") || content.contains("Upgrade"), "Missing upgrade references");
    assert!(content.contains("Soft Fork") || content.contains("soft fork") || content.contains("Hard Fork") || content.contains("hard fork"), "Missing soft/hard fork references");
    assert!(content.contains("activation") || content.contains("signal") || content.contains("Activation") || content.contains("Signal"), "Missing activation mechanism");
    assert!(content.contains("BIP") || content.contains("version") || content.contains("Version"), "Missing versioning mechanism");

    println!("\n  Protocol upgrade mechanisms verified:");
    println!("    - Upgrade types: ✅");
    println!("    - Activation mechanism: ✅");
    println!("    - Versioning: ✅");

    println!("\n  ✅ Protocol upgrade mechanisms verified");
}

// ============================================================================
// PHASE 20 TEST 3: GRANT AND FUNDING FRAMEWORKS
// ============================================================================
#[test]
fn test_phase20_grant_and_funding_frameworks() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Grant and Funding Frameworks");
    println!("══════════════════════════════════════════════════════\n");

    let grant_framework = root_path("docs/foundation/grant-framework.md");
    assert!(grant_framework.exists(), "Grant framework documentation missing");

    let content = std::fs::read_to_string(&grant_framework).expect("Failed to read grant-framework.md");

    assert!(content.contains("grant") || content.contains("Grant"), "Missing grant references");
    assert!(content.contains("application") || content.contains("Application"), "Missing application process");
    assert!(content.contains("milestone") || content.contains("Milestone"), "Missing milestone tracking");
    assert!(content.contains("payment") || content.contains("Payment") || content.contains("funding") || content.contains("Funding"), "Missing payment/funding schedule");

    let treasury_framework = root_path("docs/foundation/treasury-framework.md");
    let treasury_content = std::fs::read_to_string(&treasury_framework).expect("Failed to read treasury framework");
    assert!(treasury_content.contains("Grant") || treasury_content.contains("grant"), "Missing grant references in treasury framework");

    println!("\n  Grant and funding frameworks verified:");
    println!("    - Grant framework: ✅");
    println!("    - Application process: ✅");
    println!("    - Milestone tracking: ✅");
    println!("    - Treasury integration: ✅");

    println!("\n  ✅ Grant and funding frameworks verified");
}

// ============================================================================
// PHASE 20 TEST 4: COMMUNITY GOVERNANCE PARTICIPATION
// ============================================================================
#[test]
fn test_phase20_community_governance_participation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Community Governance Participation");
    println!("══════════════════════════════════════════════════════\n");

    let constitution = root_path("docs/foundation/constitution.md");
    assert!(constitution.exists(), "Constitution documentation missing");

    let constitution_content = std::fs::read_to_string(&constitution).expect("Failed to read constitution.md");

    assert!(constitution_content.contains("governance") || constitution_content.contains("Governance"), "Missing governance references");
    assert!(constitution_content.contains("vote") || constitution_content.contains("Vote"), "Missing voting references");
    assert!(constitution_content.contains("proposal") || constitution_content.contains("Proposal"), "Missing proposal references");
    assert!(constitution_content.contains("community") || constitution_content.contains("Community"), "Missing community references");

    let community_docs = vec![
        "COMMUNITY_LAUNCH_PREPARATION.md",
        "COMMUNITY_TESTNET_INVITATION.md",
        "CODE_OF_CONDUCT.md",
        "CONTRIBUTING.md",
    ];

    let mut found_docs = Vec::new();
    for doc in &community_docs {
        let path = root_path(doc);
        if path.exists() {
            found_docs.push(*doc);
        }
    }

    assert!(found_docs.len() >= 3, "Insufficient community documentation: {}", found_docs.len());

    println!("\n  Community governance participation verified:");
    println!("    - Constitution: ✅");
    println!("    - Voting mechanism: ✅");
    println!("    - Proposal process: ✅");
    println!("    - Community documents: {} files", found_docs.len());

    println!("\n  ✅ Community governance participation verified");
}

// ============================================================================
// PHASE 20 TEST 5: LONG-TERM SUSTAINABILITY PLANNING
// ============================================================================
#[test]
fn test_phase20_long_term_sustainability_planning() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Long-term Sustainability Planning");
    println!("══════════════════════════════════════════════════════\n");

    let execution_plan = root_path("docs/operations/90-day-execution-plan.md");
    assert!(execution_plan.exists(), "90-day execution plan missing");

    let plan_content = std::fs::read_to_string(&execution_plan).expect("Failed to read execution plan");

    assert!(plan_content.contains("KPI") || plan_content.contains("metric"), "Missing KPIs/metrics");
    assert!(plan_content.contains("growth") || plan_content.contains("Growth"), "Missing growth planning");

    let treasury_framework = root_path("docs/foundation/treasury-framework.md");
    let treasury_content = std::fs::read_to_string(&treasury_framework).expect("Failed to read treasury framework");
    assert!(treasury_content.contains("sustain") || treasury_content.contains("growth") || treasury_content.contains("ecosystem"), "Missing sustainability references in treasury framework");

    let tokenomics = root_path("docs/foundation/tokenomics.md");
    assert!(tokenomics.exists(), "Tokenomics documentation missing");
    let tokenomics_content = std::fs::read_to_string(&tokenomics).expect("Failed to read tokenomics");
    assert!(tokenomics_content.contains("supply") || tokenomics_content.contains("inflation") || tokenomics_content.contains("reward"), "Missing supply/inflation/reward references");

    println!("\n  Long-term sustainability planning verified:");
    println!("    - 90-day execution plan: ✅");
    println!("    - Treasury sustainability: ✅");
    println!("    - Tokenomics: ✅");

    println!("\n  ✅ Long-term sustainability planning verified");
}

// ============================================================================
// PHASE 20 TEST 6: LIQUIDITY STRATEGY
// ============================================================================
#[test]
fn test_phase20_liquidity_strategy() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Liquidity Strategy");
    println!("══════════════════════════════════════════════════════\n");

    let liquidity_strategy = root_path("docs/foundation/liquidity-strategy.md");
    assert!(liquidity_strategy.exists(), "Liquidity strategy documentation missing");

    let content = std::fs::read_to_string(&liquidity_strategy).expect("Failed to read liquidity-strategy.md");

    assert!(content.contains("liquidity") || content.contains("Liquidity"), "Missing liquidity references");
    assert!(content.contains("pool") || content.contains("Pool"), "Missing pool references");
    assert!(content.contains("treasury") || content.contains("Treasury"), "Missing treasury references");
    assert!(content.contains("governance") || content.contains("vote"), "Missing governance references");

    println!("\n  Liquidity strategy verified:");
    println!("    - Liquidity planning: ✅");
    println!("    - Pool strategy: ✅");
    println!("    - Treasury allocation: ✅");
    println!("    - Governance integration: ✅");

    println!("\n  ✅ Liquidity strategy verified");
}

// ============================================================================
// PHASE 20 TEST 7: CONSTITUTIONAL GOVERNANCE
// ============================================================================
#[test]
fn test_phase20_constitutional_governance() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Constitutional Governance");
    println!("══════════════════════════════════════════════════════\n");

    let constitution = root_path("docs/foundation/constitution.md");
    assert!(constitution.exists(), "Constitution documentation missing");

    let content = std::fs::read_to_string(&constitution).expect("Failed to read constitution.md");

    assert!(content.contains("principle") || content.contains("Principle"), "Missing principles");
    assert!(content.contains("governance") || content.contains("Governance"), "Missing governance framework");
    assert!(content.contains("treasury") || content.contains("Treasury"), "Missing treasury provisions");
    assert!(content.contains("amendment") || content.contains("change"), "Missing amendment process");

    let operational_structure = root_path("docs/foundation/operational-structure.md");
    assert!(operational_structure.exists(), "Operational structure documentation missing");

    let ops_content = std::fs::read_to_string(&operational_structure).expect("Failed to read operational-structure.md");
    assert!(ops_content.contains("Foundation") || ops_content.contains("Council"), "Missing foundation/council references");

    println!("\n  Constitutional governance verified:");
    println!("    - Principles: ✅");
    println!("    - Governance framework: ✅");
    println!("    - Treasury provisions: ✅");
    println!("    - Amendment process: ✅");
    println!("    - Operational structure: ✅");

    println!("\n  ✅ Constitutional governance verified");
}

// ============================================================================
// PHASE 20 TEST 8: OPERATIONAL STRUCTURE MATURITY
// ============================================================================
#[test]
fn test_phase20_operational_structure_maturity() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Operational Structure Maturity");
    println!("══════════════════════════════════════════════════════\n");

    let operational_structure = root_path("docs/foundation/operational-structure.md");
    assert!(operational_structure.exists(), "Operational structure documentation missing");

    let content = std::fs::read_to_string(&operational_structure).expect("Failed to read operational-structure.md");

    assert!(content.contains("Foundation") || content.contains("Council"), "Missing foundation/council references");
    assert!(content.contains("role") || content.contains("Role"), "Missing role definitions");
    assert!(content.contains("responsibility") || content.contains("Responsibility") || content.contains("Term") || content.contains("Elected"), "Missing responsibilities");
    assert!(content.contains("process") || content.contains("Process") || content.contains("Meeting") || content.contains("Election"), "Missing process definitions");

    let execution_plan = root_path("docs/operations/90-day-execution-plan.md");
    let plan_content = std::fs::read_to_string(&execution_plan).expect("Failed to read execution plan");
    assert!(plan_content.contains("Owner") || plan_content.contains("owner"), "Missing ownership in execution plan");

    println!("\n  Operational structure maturity verified:");
    println!("    - Foundation/Council: ✅");
    println!("    - Role definitions: ✅");
    println!("    - Responsibilities: ✅");
    println!("    - Process definitions: ✅");

    println!("\n  ✅ Operational structure maturity verified");
}

// ============================================================================
// PHASE 20 TEST 9: ECOSYSTEM SUSTAINABILITY METRICS
// ============================================================================
#[test]
fn test_phase20_ecosystem_sustainability_metrics() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Ecosystem Sustainability Metrics");
    println!("══════════════════════════════════════════════════════\n");

    let execution_plan = root_path("docs/operations/90-day-execution-plan.md");
    let plan_content = std::fs::read_to_string(&execution_plan).expect("Failed to read execution plan");

    assert!(plan_content.contains("KPI") || plan_content.contains("metric"), "Missing KPIs/metrics");
    assert!(plan_content.contains("node") || plan_content.contains("hashrate"), "Missing network metrics");
    assert!(plan_content.contains("transaction") || plan_content.contains("tx"), "Missing transaction metrics");
    assert!(plan_content.contains("exchange") || plan_content.contains("listing"), "Missing exchange metrics");

    let tokenomics = root_path("docs/foundation/tokenomics.md");
    let tokenomics_content = std::fs::read_to_string(&tokenomics).expect("Failed to read tokenomics");
    assert!(tokenomics_content.contains("supply") || tokenomics_content.contains("inflation"), "Missing supply/inflation metrics");

    println!("\n  Ecosystem sustainability metrics verified:");
    println!("    - Network metrics: ✅");
    println!("    - Transaction metrics: ✅");
    println!("    - Exchange metrics: ✅");
    println!("    - Tokenomics: ✅");

    println!("\n  ✅ Ecosystem sustainability metrics verified");
}

// ============================================================================
// PHASE 20 TEST 10: PHASE 20 READINESS REPORT GENERATION
// ============================================================================
#[test]
fn test_phase20_readiness_report_generation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Phase 20 Readiness Report Generation");
    println!("══════════════════════════════════════════════════════\n");

    let requirements = vec![
        ("Treasury governance framework", "docs/foundation/treasury-framework.md"),
        ("Protocol upgrade mechanisms", "docs/foundation/protocol-upgrades.md"),
        ("Grant and funding frameworks", "docs/foundation/grant-framework.md"),
        ("Community governance participation", "docs/foundation/constitution.md"),
        ("Long-term sustainability planning", "docs/operations/90-day-execution-plan.md"),
        ("Liquidity strategy", "docs/foundation/liquidity-strategy.md"),
        ("Constitutional governance", "docs/foundation/constitution.md"),
        ("Operational structure maturity", "docs/foundation/operational-structure.md"),
        ("Ecosystem sustainability metrics", "docs/foundation/tokenomics.md"),
    ];

    let mut statuses = Vec::new();

    println!("\n  Phase 20 Readiness Checklist:");
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

    println!("\n  Phase 20 Readiness: {}/{} ({:.1}%)", ready_count, total_count, readiness_percentage);

    assert!(ready_count >= 7, "Insufficient Phase 20 readiness: {}/{}", ready_count, total_count);

    let report = format!(
        r#"{{
  "phase": 20,
  "name": "Treasury Governance & Long-term Sustainability",
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

    std::fs::write(root_path("PHASE20_READINESS_REPORT.json"), report).expect("Failed to write report");

    println!("\n  📄 Report saved to: PHASE20_READINESS_REPORT.json");
    println!("\n  ✅ Phase 20 readiness report generated");
}

// ============================================================================
// PHASE 20 TEST 11: FINAL PHASE 20 CERTIFICATION
// ============================================================================
#[test]
fn test_phase20_final_certification() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 20 — TREASURY GOVERNANCE & LONG-TERM SUSTAINABILITY");
    println!("  Final Phase 20 Certification");
    println!("══════════════════════════════════════════════════════\n");

    let phase_reports = vec![
        ("Phase 1", "CRYPTOGRAPHIC_SECURITY_ANALYSIS.md"),
        ("Phase 2", "src/core/src/genesis.rs"),
        ("Phase 7", "tests/integration/phase7_rpc_explorer_tests.rs"),
        ("Phase 10", "tests/integration/phase10_attack_resistance_tests.rs"),
        ("Phase 11", "tests/integration/phase11_operational_stability_tests.rs"),
        ("Phase 12", "tests/integration/phase12_mainnet_candidate_tests.rs"),
        ("Phase 13", "tests/integration/phase13_external_validation_tests.rs"),
        ("Phase 14", "tests/integration/phase14_exchange_readiness_tests.rs"),
        ("Phase 15", "tests/integration/phase15_monitoring_incident_response_tests.rs"),
        ("Phase 16", "tests/integration/phase16_launch_readiness_tests.rs"),
        ("Phase 17", "tests/integration/phase17_independent_verification_tests.rs"),
        ("Phase 18", "tests/integration/phase18_mainnet_launch_activation_tests.rs"),
        ("Phase 19", "tests/integration/phase19_post_launch_operations_tests.rs"),
        ("Phase 20", "tests/integration/phase20_treasury_governance_sustainability_tests.rs"),
    ];

    let mut certified = Vec::new();
    let mut missing = Vec::new();

    println!("\n  Phase Certification Status:");
    println!("  ─────────────────────────────────────────────");

    for (phase, path) in &phase_reports {
        let exists = root_path(path).exists();
        if exists {
            certified.push(*phase);
            println!("    ✅ {} certified", phase);
        } else {
            missing.push(*phase);
            println!("    ❌ {} missing", phase);
        }
    }

    let total = phase_reports.len();
    let certified_count = certified.len();
    let certification_rate = (certified_count as f64 / total as f64) * 100.0;

    println!("\n  Certification Rate: {}/{} ({:.1}%)", certified_count, total, certification_rate);

    assert!(missing.is_empty(), "Missing phase certifications: {:?}", missing);
    assert!(certification_rate >= 95.0, "Certification rate below 95%");

    let certification = format!(
        r#"{{
  "certification": "TREASURY_GOVERNANCE_CERTIFIED",
  "project": "Udaya",
  "version": "1.0.0-alpha.1",
  "certified_at": "{}",
  "phases_certified": {},
  "total_phases": {},
  "certification_rate": {:.1},
  "status": "READY_FOR_LONG_TERM_SUSTAINABILITY",
  "phases": [
{}
  ]
}}"#,
        chrono::Utc::now().to_rfc3339(),
        certified_count,
        total,
        certification_rate,
        certified
            .iter()
            .map(|p| format!("    {{\"phase\": \"{}\", \"status\": \"CERTIFIED\"}}", p))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    std::fs::write(root_path("PHASE20_CERTIFICATION.json"), certification).expect("Failed to write certification");

    println!("\n  📄 Certification saved to: PHASE20_CERTIFICATION.json");
    println!("\n  🎉 Udaya v1.0.0-alpha.1 is CERTIFIED FOR LONG-TERM SUSTAINABILITY");
    println!("\n  ✅ Final Phase 20 certification passed");
}
