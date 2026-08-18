# Udaya Blockchain Upgrade Procedures

## Table of Contents
1. [Overview](#overview)
2. [Upgrade Types](#upgrade-types)
3. [Pre-Upgrade Checklist](#pre-upgrade-checklist)
4. [Software Upgrade Procedures](#software-upgrade-procedures)
5. [Protocol Upgrade Procedures](#protocol-upgrade-procedures)
6. [Rollback Procedures](#rollback-procedures)
7. [Testing and Validation](#testing-and-validation)
8. [Communication Plan](#communication-plan)
9. [Monitoring and Alerting](#monitoring-and-alerting)
10. [Documentation Updates](#documentation-updates)
11. [Post-Upgrade Procedures](#post-upgrade-procedures)
12. [Version Compatibility Matrix](#version-compatibility-matrix)

## Overview

This document outlines the comprehensive upgrade procedures for the Udaya Blockchain infrastructure. Proper upgrade procedures are essential for maintaining network stability, security, and compatibility.

## Upgrade Types

### Software Upgrades
- **Patch Upgrades**: Bug fixes and security patches (e.g., 1.0.0 → 1.0.1)
- **Minor Upgrades**: Feature additions and improvements (e.g., 1.0.0 → 1.1.0)
- **Major Upgrades**: Breaking changes and new functionality (e.g., 1.0.0 → 2.0.0)

### Protocol Upgrades
- **Soft Forks**: Backward-compatible upgrades
- **Hard Forks**: Non-backward-compatible upgrades
- **Network Upgrades**: Consensus rule changes

### Infrastructure Upgrades
- **Node Upgrades**: Individual node software updates
- **Service Upgrades**: Explorer, RPC, Faucet updates
- **Dependency Upgrades**: Library and component updates

## Pre-Upgrade Checklist

### Preparation Phase
- [ ] Review release notes and changelog
- [ ] Identify breaking changes and dependencies
- [ ] Test upgrade in staging environment
- [ ] Create backup of current state
- [ ] Notify stakeholders and community
- [ ] Schedule maintenance window
- [ ] Prepare rollback plan
- [ ] Update monitoring and alerting

### Compatibility Check
```bash
# Check current version
udaya-cli --version

# Check network compatibility
udaya-cli getnetworkinfo | grep protocolversion

# Check peer compatibility
udaya-cli getpeerinfo | grep version
```

### Backup Verification
```bash
# Verify recent backup exists
ls -lh /backups/udaya/udaya_blockchain_*.tar.gz.gpg | tail -1

# Test backup restore
/usr/local/bin/udaya-backup-verify.sh

# Check database backup
pg_isready -h ${DB_HOST} -p ${DB_PORT} && echo "Database ready"
```

## Software Upgrade Procedures

### Node Software Upgrade

```bash
# 1. Announce maintenance
echo "Starting Udaya node upgrade at $(date)" | tee -a /var/log/udaya/upgrade.log

# 2. Stop services
systemctl stop udaya-node
systemctl stop udaya-explorer
systemctl stop udaya-faucet

# 3. Backup current installation
tar -czvf /backups/udaya/udaya-software-$(date +%Y%m%d).tar.gz \
    /usr/local/bin/udaya* \
    /etc/udaya/*.conf \
    /lib/systemd/system/udaya*.service

# 4. Download new version
wget https://github.com/zypherlabs-bit/udaya/releases/download/v${NEW_VERSION}/udaya-${NEW_VERSION}-linux-amd64.tar.gz
tar -xzvf udaya-${NEW_VERSION}-linux-amd64.tar.gz -C /usr/local/bin/

# 5. Update configuration (if needed)
/usr/local/bin/udaya-config-migrate --from v${CURRENT_VERSION} --to v${NEW_VERSION}

# 6. Verify installation
/usr/local/bin/udaya-node --version | grep ${NEW_VERSION}
/usr/local/bin/udaya-cli --version | grep ${NEW_VERSION}

# 7. Start services
systemctl start udaya-node
systemctl start udaya-explorer
systemctl start udaya-faucet

# 8. Verify operation
udaya-cli getblockchaininfo | grep blocks | grep -q "[1-9]"
udaya-cli getnetworkinfo | grep connections | grep -q "[1-9]"

# 9. Log completion
echo "Udaya node upgrade to v${NEW_VERSION} completed at $(date)" | tee -a /var/log/udaya/upgrade.log
```

### Docker Upgrade Procedure

```bash
# 1. Pull new image
docker pull udaya/udaya-node:${NEW_VERSION}

# 2. Stop current containers
docker stop udaya-node udaya-explorer udaya-faucet

# 3. Backup current containers
docker commit udaya-node udaya/udaya-node-backup:$(date +%Y%m%d)
docker commit udaya-explorer udaya/udaya-explorer-backup:$(date +%Y%m%d)
docker commit udaya-faucet udaya/udaya-faucet-backup:$(date +%Y%m%d)

# 4. Update docker-compose.yml
sed -i "s|image: udaya/udaya-node:.*|image: udaya/udaya-node:${NEW_VERSION}|" docker-compose.yml

# 5. Start new containers
docker-compose up -d

# 6. Verify operation
docker logs udaya-node | grep "Udaya node started"
docker exec udaya-node udaya-cli getblockchaininfo | grep version | grep ${NEW_VERSION}
```

### Kubernetes Upgrade Procedure

```yaml
# Rolling update strategy
apiVersion: apps/v1
kind: Deployment
metadata:
  name: udaya-node
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: udaya-node
        image: udaya/udaya-node:${NEW_VERSION}
        ports:
        - containerPort: 9798
        readinessProbe:
          httpGet:
            path: /health
            port: 18332
          initialDelaySeconds: 30
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 18332
          initialDelaySeconds: 60
          periodSeconds: 15
```

## Protocol Upgrade Procedures

### Soft Fork Activation

```bash
# 1. Announce soft fork
echo "Initiating Udaya soft fork activation" | tee -a /var/log/udaya/softfork.log

# 2. Update node software
/usr/local/bin/udaya-softfork-prepare --version ${NEW_VERSION} --activation-height ${ACTIVATION_HEIGHT}

# 3. Configure version bits
udaya-cli setversionbits --bit ${BIP9_BIT} --start-height ${START_HEIGHT} --end-height ${END_HEIGHT}

# 4. Monitor miner signaling
watch -n 60 "udaya-cli getblockchaininfo | grep softforks"

# 5. Verify activation
udaya-cli getblockchaininfo | grep -A 10 softforks | grep ${SOFTFORK_NAME} | grep active

# 6. Log completion
echo "Udaya soft fork ${SOFTFORK_NAME} activated at height ${ACTIVATION_HEIGHT}" | tee -a /var/log/udaya/softfork.log
```

### Hard Fork Activation

```bash
# 1. Announce hard fork (community coordination required)
echo "Initiating Udaya hard fork preparation" | tee -a /var/log/udaya/hardfork.log

# 2. Schedule maintenance window
# 3. Coordinate with exchanges and services
# 4. Update all nodes to compatible version
# 5. Set activation height
udaya-cli sethardfork --height ${ACTIVATION_HEIGHT} --rules ${NEW_RULES}

# 6. Monitor network readiness
udaya-cli getnetworkinfo | grep "upgrade_ready" | grep true

# 7. Execute hard fork at scheduled height
# 8. Monitor chain split detection
udaya-cli getchaininfo | grep "chain" | grep "main"

# 9. Verify post-fork operation
udaya-cli getblockchaininfo | grep height | grep -A 1 ${ACTIVATION_HEIGHT}

# 10. Log completion
echo "Udaya hard fork completed at height ${ACTIVATION_HEIGHT}" | tee -a /var/log/udaya/hardfork.log
```

## Rollback Procedures

### Software Rollback

```bash
# 1. Announce rollback
echo "Initiating Udaya rollback from v${NEW_VERSION} to v${OLD_VERSION}" | tee -a /var/log/udaya/rollback.log

# 2. Stop services
systemctl stop udaya-node udaya-explorer udaya-faucet

# 3. Restore from backup
latest_backup=$(find /backups/udaya -name "udaya-software-*.tar.gz" | sort | tail -1)
tar -xzvf "$latest_backup" -C /

# 4. Restore database (if needed)
/usr/local/bin/udaya-db-restore.sh --version ${OLD_VERSION}

# 5. Verify rollback
/usr/local/bin/udaya-node --version | grep ${OLD_VERSION}

# 6. Start services
systemctl start udaya-node udaya-explorer udaya-faucet

# 7. Verify operation
udaya-cli getblockchaininfo | grep version | grep ${OLD_VERSION}

# 8. Log completion
echo "Udaya rollback to v${OLD_VERSION} completed" | tee -a /var/log/udaya/rollback.log
```

### Database Rollback

```bash
# 1. Stop database-dependent services
systemctl stop udaya-explorer udaya-faucet

# 2. Identify backup point
latest_good_backup=$(find /backups/udaya -name "udaya_explorer_*.dump.gpg" -mtime -7 | sort | tail -1)

# 3. Restore database
gpg --decrypt "$latest_good_backup" | pg_restore -U ${DB_USER} -h ${DB_HOST} -p ${DB_PORT} -d udaya_explorer -c -v

# 4. Verify data integrity
psql -U ${DB_USER} -h ${DB_HOST} -p ${DB_PORT} -d udaya_explorer -c "SELECT COUNT(*) FROM blocks;" | grep -q "[1-9]"

# 5. Reindex if necessary
udaya-cli reindex --service explorer

# 6. Restart services
systemctl start udaya-explorer udaya-faucet

# 7. Log rollback
echo "Database rolled back to $(basename "$latest_good_backup")" | tee -a /var/log/udaya/rollback.log
```

## Testing and Validation

### Pre-Upgrade Testing

```bash
# Test in staging environment
docker-compose -f docker-compose.staging.yml up -d

# Run integration tests
cd tests/integration && cargo test --all

# Run performance benchmarks
cd benches && cargo bench --all

# Validate API compatibility
udaya-cli test-api-compatibility --version ${NEW_VERSION}

# Test rollback procedure
/usr/local/bin/udaya-rollback-test.sh --version ${NEW_VERSION}
```

### Post-Upgrade Validation

```bash
# 1. Verify node operation
udaya-cli getblockchaininfo | grep -E "(version|blocks|connections)" | grep -q "[1-9]"

# 2. Test RPC methods
udaya-cli getblockcount && udaya-cli getblockhash 1 && udaya-cli getblock 1

# 3. Test explorer functionality
curl -s https://explorer.udaya.net/api/blocks/latest | grep -q "hash"

# 4. Test faucet operation
curl -s https://faucet.udaya.net/health | grep -q "ok"

# 5. Test P2P connectivity
udaya-cli getpeerinfo | grep -q "connected"

# 6. Run post-upgrade checks
/usr/local/bin/udaya-post-upgrade-check.sh --version ${NEW_VERSION}

# 7. Monitor for 24 hours
/usr/local/bin/udaya-monitor-post-upgrade.sh --duration 24h
```

## Communication Plan

### Stakeholder Communication

```markdown
# Udaya Upgrade Communication Template

## Upgrade Announcement

**Subject**: Udaya Network Upgrade Scheduled - v${CURRENT_VERSION} → v${NEW_VERSION}

**Date**: [Upgrade Date]
**Time**: [Upgrade Time] UTC
**Expected Duration**: [Duration] hours
**Network Impact**: [Minimal/Moderate/Significant]

### Upgrade Details

- **Version**: v${NEW_VERSION}
- **Type**: [Software/Protocol/Infrastructure]
- **Changes**: [Brief description of changes]
- **Backward Compatibility**: [Yes/No]
- **Rollback Plan**: Available if needed

### Affected Services

- [ ] Node Software
- [ ] RPC Services
- [ ] Block Explorer
- [ ] Faucet Service
- [ ] Wallet Services

### User Actions Required

1. **Node Operators**: Update to v${NEW_VERSION} before [deadline]
2. **Exchange Operators**: Suspend deposits/withdrawals during upgrade window
3. **Wallet Users**: No action required (automatic compatibility)
4. **Mining Pools**: Update mining software to v${NEW_VERSION}

### Support Channels

- **Documentation**: https://docs.udaya.org/upgrade-guide-v${NEW_VERSION}
- **Support Email**: support@udaya.org
- **Community Chat**: https://discord.gg/udaya
- **Status Page**: https://status.udaya.org
```

### Status Updates

```json
{
  "upgrade": {
    "version": "v1.1.0",
    "status": "in_progress",
    "start_time": "2026-08-03T00:00:00Z",
    "current_phase": "node_upgrade",
    "completion_percent": 65,
    "estimated_completion": "2026-08-03T02:30:00Z",
    "services": {
      "nodes": "upgrading",
      "rpc": "operational",
      "explorer": "operational",
      "faucet": "operational"
    },
    "issues": [],
    "next_steps": "Final validation and monitoring"
  }
}
```

## Monitoring and Alerting

### Upgrade Monitoring Dashboard

```yaml
# Grafana dashboard for upgrade monitoring
apiVersion: 1

providers:
- name: 'Udaya Upgrade Monitoring'
  orgId: 1
  folder: 'Upgrades'
  type: file
  disableDeletion: false
  updateIntervalSeconds: 30
  options:
    path: /var/lib/grafana/dashboards/upgrades
```

### Alert Rules

```yaml
# Prometheus alert rules for upgrades
groups:
- name: upgrade-alerts
  rules:
  - alert: UpgradeFailed
    expr: udaya_upgrade_status == 0
    for: 15m
    labels:
      severity: critical
      service: upgrade
    annotations:
      summary: "Upgrade failed for {{ $labels.instance }}"
      description: "Upgrade process failed at phase {{ $labels.phase }}"

  - alert: UpgradeStalled
    expr: time() - udaya_upgrade_last_progress > 3600
    for: 30m
    labels:
      severity: warning
      service: upgrade
    annotations:
      summary: "Upgrade stalled for {{ $labels.instance }}"
      description: "No progress in upgrade for 1 hour"

  - alert: NodeVersionMismatch
    expr: count(udaya_node_version) by (version) > 1
    for: 10m
    labels:
      severity: warning
      service: upgrade
    annotations:
      summary: "Node version mismatch detected"
      description: "Multiple node versions detected: {{ $labels.version }}"

  - alert: PostUpgradeErrors
    expr: rate(udaya_errors_total[5m]) > 10
    for: 15m
    labels:
      severity: critical
      service: upgrade
    annotations:
      summary: "High error rate after upgrade"
      description: "Error rate {{ $value }} errors/min after upgrade"
```

## Documentation Updates

### Version-Specific Documentation

```markdown
# Udaya v${NEW_VERSION} Release Notes

## New Features

- [Feature 1]: Description of new feature
- [Feature 2]: Description of new feature
- [Feature 3]: Description of new feature

## Improvements

- [Improvement 1]: Performance enhancement details
- [Improvement 2]: Security improvement details
- [Improvement 3]: Usability improvement details

## Bug Fixes

- [Bug Fix 1]: Description of fixed issue
- [Bug Fix 2]: Description of fixed issue
- [Bug Fix 3]: Description of fixed issue

## Breaking Changes

- [Breaking Change 1]: Description and migration path
- [Breaking Change 2]: Description and migration path

## Deprecations

- [Deprecated Feature 1]: Replacement and timeline
- [Deprecated Feature 2]: Replacement and timeline

## Migration Guide

### From v${OLD_VERSION} to v${NEW_VERSION}

1. **Backup**: Create full backup of node data
2. **Update**: Install new software version
3. **Configure**: Apply new configuration settings
4. **Test**: Verify functionality in staging
5. **Deploy**: Roll out to production
6. **Monitor**: Observe for 24-48 hours
```

### API Documentation Updates

```yaml
openapi: 3.0.0
info:
  title: Udaya API
  version: ${NEW_VERSION}
  description: Udaya Blockchain API Documentation
paths:
  /api/v2/blocks:
    get:
      summary: Get blocks (v2)
      description: Enhanced block retrieval with new fields
      parameters:
        - name: height
          in: query
          description: Block height
          required: false
          schema:
            type: integer
        - name: hash
          in: query
          description: Block hash
          required: false
          schema:
            type: string
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BlockV2'
```

## Post-Upgrade Procedures

### Monitoring and Validation

```bash
# 1. Extended monitoring period
/usr/local/bin/udaya-post-upgrade-monitor.sh --duration 72h

# 2. Performance benchmarking
cd benches && cargo bench --all -- --save-baseline PostUpgrade

# 3. Compare with pre-upgrade baseline
/usr/local/bin/udaya-benchmark-compare.sh --pre PreUpgrade --post PostUpgrade

# 4. Community feedback collection
echo "Upgrade completed. Please report any issues to support@udaya.org" | \
    mail -s "Udaya v${NEW_VERSION} Upgrade Complete" community@udaya.org

# 5. Update status page
curl -X POST https://status.udaya.org/api/incidents \
    -H "Authorization: Bearer ${STATUS_API_KEY}" \
    -H "Content-Type: application/json" \
    -d '{"status": "resolved", "message": "Upgrade to v${NEW_VERSION} completed successfully"}'
```

### Documentation Archive

```bash
# Archive old documentation
mkdir -p /docs/archive/v${OLD_VERSION}
cp -r /docs/current/* /docs/archive/v${OLD_VERSION}/

# Update version references
find /docs/current -type f -exec sed -i "s/v${OLD_VERSION}/v${NEW_VERSION}/g" {} \;

# Generate PDF documentation
cd /docs && pandoc -s current/*.md -o udaya-v${NEW_VERSION}-documentation.pdf

# Update website
cd /website && npm run build && rsync -avz dist/ user@docs.udaya.org:/var/www/docs/
```

## Version Compatibility Matrix

| Version | Compatible With | Notes |
|---------|-----------------|-------|
| v1.0.0 | v1.0.x | Initial release |
| v1.1.0 | v1.0.x, v1.1.x | Backward compatible |
| v1.2.0 | v1.1.x, v1.2.x | Breaking changes in API v2 |
| v2.0.0 | v2.x | Major protocol upgrade |

### Node Compatibility

| Node Version | P2P Protocol | RPC Protocol | Notes |
|--------------|--------------|--------------|-------|
| v1.0.0 | 70015 | 1.0 | Initial release |
| v1.1.0 | 70016 | 1.1 | Enhanced P2P |
| v1.2.0 | 70017 | 1.2 | TLS 1.3 support |
| v2.0.0 | 80001 | 2.0 | New consensus rules |

### Wallet Compatibility

| Wallet Version | Node Version | Notes |
|----------------|--------------|-------|
| v1.0.0 | v1.0.x | Initial wallet |
| v1.1.0 | v1.0.x, v1.1.x | HD wallet support |
| v1.2.0 | v1.1.x, v1.2.x | Hardware wallet integration |
| v2.0.0 | v2.x | New address formats |

## Appendix

### Upgrade Checklist

- [ ] Review release notes and changelog
- [ ] Test upgrade in staging environment
- [ ] Create comprehensive backup
- [ ] Notify all stakeholders
- [ ] Schedule maintenance window
- [ ] Prepare rollback plan
- [ ] Update monitoring and alerting
- [ ] Execute upgrade procedure
- [ ] Validate post-upgrade operation
- [ ] Monitor for 24-48 hours
- [ ] Update documentation
- [ ] Communicate completion
- [ ] Conduct post-mortem review

### Common Upgrade Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Version mismatch | Not all nodes upgraded | Enforce minimum version requirement |
| Chain split | Protocol incompatibility | Coordinate hard fork activation |
| Performance degradation | New features overhead | Optimize configuration settings |
| API compatibility | Breaking changes | Maintain backward compatibility layer |
| Database migration | Schema changes | Test migration thoroughly |

### Contact Information

- **Upgrade Team**: upgrade@udaya.org
- **Support**: support@udaya.org
- **Security**: security@udaya.org
- **24/7 Hotline**: +1 (800) UDYA-HELP
- **Status Page**: https://status.udaya.org