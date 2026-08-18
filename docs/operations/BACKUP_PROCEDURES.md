# Udaya Blockchain Backup Procedures

## Table of Contents
1. [Overview](#overview)
2. [Backup Strategy](#backup-strategy)
3. [Database Backup Procedures](#database-backup-procedures)
4. [Wallet Backup Procedures](#wallet-backup-procedures)
5. [Configuration Backup Procedures](#configuration-backup-procedures)
6. [Automated Backup Scripts](#automated-backup-scripts)
7. [Backup Verification](#backup-verification)
8. [Restore Procedures](#restore-procedures)
9. [Disaster Recovery](#disaster-recovery)
10. [Monitoring and Alerting](#monitoring-and-alerting)
11. [Security Considerations](#security-considerations)
12. [Compliance and Retention](#compliance-and-retention)

## Overview

This document outlines the comprehensive backup procedures for the Udaya Blockchain infrastructure. Proper backup procedures are essential for ensuring data integrity, disaster recovery, and business continuity.

## Backup Strategy

### Backup Types
- **Full Backups**: Complete copy of all data (weekly)
- **Incremental Backups**: Only changes since last backup (daily)
- **Differential Backups**: Changes since last full backup (daily)
- **Snapshot Backups**: Point-in-time copies for critical systems (hourly)

### Backup Frequency
- **Blockchain Data**: Hourly incremental, daily full
- **Database**: Every 6 hours incremental, weekly full
- **Wallets**: Real-time encrypted backups
- **Configurations**: Daily automated backups
- **Logs**: Daily backups with 30-day retention

### Backup Locations
- **Primary**: AWS S3 (us-east-1)
- **Secondary**: Google Cloud Storage (europe-west1)
- **Tertiary**: On-premises NAS (geographically distributed)
- **Cold Storage**: AWS Glacier Deep Archive (quarterly archives)

### Retention Policy
- **Daily Backups**: 30 days
- **Weekly Backups**: 12 weeks
- **Monthly Backups**: 12 months
- **Yearly Backups**: 7 years
- **Critical Backups**: Indefinite (genesis block, foundation wallets)

## Database Backup Procedures

### PostgreSQL Backup (Explorer & Faucet)

```bash
# Full backup
pg_dump -U ${DB_USER} -h ${DB_HOST} -p ${DB_PORT} -F c -b -v -f /backups/udaya_explorer_$(date +%Y%m%d).dump udaya_explorer

# Incremental backup (using WAL archiving)
pg_basebackup -D /backups/udaya_explorer_incr_$(date +%Y%m%d) -h ${DB_HOST} -U ${DB_USER} -P -v -z -Ft -Xs

# Verify backup integrity
pg_restore --verify --dbname=udaya_explorer /backups/udaya_explorer_$(date +%Y%m%d).dump
```

### RocksDB Backup (Blockchain)

```bash
# Hot backup (online)
/usr/local/bin/udaya-db-backup --source /data/udaya/mainnet --destination /backups/udaya_blockchain_$(date +%Y%m%d) --type hot

# Cold backup (offline - requires node shutdown)
systemctl stop udaya-node
cp -r /data/udaya/mainnet /backups/udaya_blockchain_cold_$(date +%Y%m%d)
systemctl start udaya-node

# Verify backup
/usr/local/bin/udaya-db-verify --backup /backups/udaya_blockchain_$(date +%Y%m%d)
```

## Wallet Backup Procedures

### HD Wallet Backup

```bash
# Export wallet seed (BIP39 mnemonic)
udaya-cli wallet export-seed --wallet-file /data/udaya/wallets/mainnet.dat --output /backups/wallet_seed_$(date +%Y%m%d).txt

# Encrypt wallet backup
openssl enc -aes-256-cbc -salt -in /data/udaya/wallets/mainnet.dat -out /backups/wallet_enc_$(date +%Y%m%d).dat -pass file:/etc/udaya/backup_key.pem

# Verify wallet backup
udaya-cli wallet verify-backup --backup /backups/wallet_enc_$(date +%Y%m%d).dat --key /etc/udaya/backup_key.pem
```

### Multi-Signature Wallet Backup

```bash
# Export multi-sig configuration
udaya-cli wallet export-multisig --wallet mainnet-multisig --output /backups/multisig_$(date +%Y%m%d).json

# Backup individual signer keys (separate locations)
for signer in signer1 signer2 signer3; do
    udaya-cli wallet export-key --signer $signer --output /backups/${signer}_key_$(date +%Y%m%d).dat
    openssl enc -aes-256-cbc -in /backups/${signer}_key_$(date +%Y%m%d).dat -out /backups/${signer}_key_enc_$(date +%Y%m%d).dat -pass file:/etc/udaya/${signer}_key.pem
done
```

## Configuration Backup Procedures

### Node Configuration Backup

```bash
# Backup all configuration files
tar -czvf /backups/config_$(date +%Y%m%d).tar.gz \
    /etc/udaya/*.conf \
    /etc/udaya/tls/*.pem \
    /etc/systemd/system/udaya*.service \
    /etc/nginx/conf.d/udaya*.conf

# Encrypt configuration backup
gpg --encrypt --recipient backup@udaya.org --output /backups/config_enc_$(date +%Y%m%d).tar.gz.gpg /backups/config_$(date +%Y%m%d).tar.gz

# Verify configuration backup
gpg --decrypt --output /tmp/config_test.tar.gz /backups/config_enc_$(date +%Y%m%d).tar.gz.gpg
tar -tzf /tmp/config_test.tar.gz | grep -E "(\.conf|\.pem|\.service)" | wc -l
```

## Automated Backup Scripts

### Main Backup Script (`/usr/local/bin/udaya-backup.sh`)

```bash
#!/bin/bash

# Udaya Comprehensive Backup Script
set -euo pipefail

# Configuration
BACKUP_DIR="/backups/udaya"
DATE=$(date +%Y%m%d_%H%M%S)
LOG_FILE="/var/log/udaya/backup_${DATE}.log"
RETENTION_DAYS=30

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Create backup directory
mkdir -p "$BACKUP_DIR"
log "Starting Udaya backup process"

# 1. Database Backup
log "Backing up databases..."
/usr/local/bin/udaya-db-backup.sh || { log "Database backup failed"; exit 1; }

# 2. Blockchain Backup
log "Backing up blockchain..."
/usr/local/bin/udaya-blockchain-backup.sh || { log "Blockchain backup failed"; exit 1; }

# 3. Wallet Backup
log "Backing up wallets..."
/usr/local/bin/udaya-wallet-backup.sh || { log "Wallet backup failed"; exit 1; }

# 4. Configuration Backup
log "Backing up configurations..."
/usr/local/bin/udaya-config-backup.sh || { log "Configuration backup failed"; exit 1; }

# 5. Logs Backup
log "Backing up logs..."
/usr/local/bin/udaya-logs-backup.sh || { log "Logs backup failed"; exit 1; }

# 6. Encrypt all backups
log "Encrypting backups..."
find "$BACKUP_DIR" -name "*.tar.gz" -o -name "*.dump" -o -name "*.dat" | while read -r file; do
    gpg --encrypt --recipient backup@udaya.org --output "${file}.gpg" "$file" && rm "$file"
done

# 7. Upload to cloud storage
log "Uploading to cloud storage..."
aws s3 sync "$BACKUP_DIR" "s3://udaya-backups/mainnet/" --delete
gsutil -m rsync -r "$BACKUP_DIR" "gs://udaya-backups/mainnet/"

# 8. Clean up old backups
log "Cleaning up old backups..."
find "$BACKUP_DIR" -name "*.gpg" -mtime +$RETENTION_DAYS -delete

# 9. Verify backups
log "Verifying backups..."
/usr/local/bin/udaya-backup-verify.sh || { log "Backup verification failed"; exit 1; }

log "Backup process completed successfully"
exit 0
```

### Backup Verification Script

```bash
#!/bin/bash

# Udaya Backup Verification Script
set -euo pipefail

BACKUP_DIR="/backups/udaya"
VERIFY_DIR="/tmp/backup_verify"
DATE=$(date +%Y%m%d_%H%M%S)
LOG_FILE="/var/log/udaya/verify_${DATE}.log"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Create verification directory
mkdir -p "$VERIFY_DIR"
log "Starting backup verification"

# Test database backup
log "Verifying database backup..."
latest_db=$(find "$BACKUP_DIR" -name "udaya_explorer_*.dump.gpg" | sort | tail -1)
gpg --decrypt "$latest_db" > "$VERIFY_DIR/latest.dump"
pg_restore --verify --dbname=postgres "$VERIFY_DIR/latest.dump" && log "Database backup verified" || log "Database backup verification failed"

# Test blockchain backup
log "Verifying blockchain backup..."
latest_blockchain=$(find "$BACKUP_DIR" -name "udaya_blockchain_*.tar.gz.gpg" | sort | tail -1)
gpg --decrypt "$latest_blockchain" > "$VERIFY_DIR/latest.tar.gz"
tar -tzf "$VERIFY_DIR/latest.tar.gz" | grep -E "(blocks|chainstate)" | wc -l | grep -q "[1-9]" && log "Blockchain backup verified" || log "Blockchain backup verification failed"

# Test wallet backup
log "Verifying wallet backup..."
latest_wallet=$(find "$BACKUP_DIR" -name "wallet_*.dat.gpg" | sort | tail -1)
gpg --decrypt "$latest_wallet" > "$VERIFY_DIR/latest.dat"
file "$VERIFY_DIR/latest.dat" | grep -q "data" && log "Wallet backup verified" || log "Wallet backup verification failed"

# Clean up
rm -rf "$VERIFY_DIR"
log "Backup verification completed"
```

## Restore Procedures

### Database Restore

```bash
# Stop services
systemctl stop udaya-explorer
systemctl stop udaya-faucet

# Restore from backup
latest_backup=$(find /backups/udaya -name "udaya_explorer_*.dump.gpg" | sort | tail -1)
gpg --decrypt "$latest_backup" | pg_restore -U ${DB_USER} -h ${DB_HOST} -p ${DB_PORT} -d udaya_explorer -c -v

# Verify restore
psql -U ${DB_USER} -h ${DB_HOST} -p ${DB_PORT} -d udaya_explorer -c "SELECT COUNT(*) FROM blocks;" | grep -q "[1-9]"

# Restart services
systemctl start udaya-explorer
systemctl start udaya-faucet
```

### Blockchain Restore

```bash
# Stop node
systemctl stop udaya-node

# Restore blockchain data
latest_backup=$(find /backups/udaya -name "udaya_blockchain_*.tar.gz.gpg" | sort | tail -1)
gpg --decrypt "$latest_backup" | tar -xz -C /data/udaya/mainnet --strip-components=1

# Verify restore
/usr/local/bin/udaya-db-verify --data /data/udaya/mainnet

# Reindex if necessary
udaya-cli reindex --data-dir /data/udaya/mainnet

# Restart node
systemctl start udaya-node
```

### Wallet Restore

```bash
# Stop wallet services
systemctl stop udaya-wallet

# Restore wallet
latest_backup=$(find /backups/udaya -name "wallet_*.dat.gpg" | sort | tail -1)
gpg --decrypt "$latest_backup" > /data/udaya/wallets/mainnet_restore.dat

# Verify wallet integrity
udaya-cli wallet verify --wallet-file /data/udaya/wallets/mainnet_restore.dat

# Replace active wallet (after verification)
mv /data/udaya/wallets/mainnet.dat /data/udaya/wallets/mainnet_backup_$(date +%Y%m%d).dat
mv /data/udaya/wallets/mainnet_restore.dat /data/udaya/wallets/mainnet.dat

# Restart wallet services
systemctl start udaya-wallet
```

## Disaster Recovery

### Full Node Recovery Procedure

1. **Declare Disaster**: Activate disaster recovery team
2. **Assess Damage**: Determine scope of data loss
3. **Select Recovery Point**: Choose most recent valid backup
4. **Provision New Infrastructure**: Spin up replacement nodes
5. **Restore from Backup**: Follow restore procedures above
6. **Sync with Network**: Reconnect to P2P network
7. **Validate Data**: Verify blockchain integrity
8. **Resume Operations**: Bring services back online
9. **Post-Mortem**: Document incident and lessons learned

### Recovery Time Objectives (RTO)
- **Seed Nodes**: 2 hours
- **RPC Nodes**: 1 hour
- **Explorer**: 30 minutes
- **Faucet**: 15 minutes
- **Databases**: 1 hour

### Recovery Point Objectives (RPO)
- **Blockchain Data**: 1 hour
- **Database Data**: 6 hours
- **Wallet Data**: Real-time (encrypted)
- **Configuration**: 24 hours

## Monitoring and Alerting

### Backup Monitoring

```yaml
# Prometheus alert rules for backup monitoring
groups:
- name: backup-alerts
  rules:
  - alert: BackupFailed
    expr: udaya_backup_success == 0
    for: 15m
    labels:
      severity: critical
      service: backup
    annotations:
      summary: "Backup failed for {{ $labels.instance }}"
      description: "Backup process failed. Last successful backup: {{ $labels.last_success }}"

  - alert: BackupStale
    expr: time() - udaya_backup_last_success > 86400
    for: 1h
    labels:
      severity: warning
      service: backup
    annotations:
      summary: "Backup stale for {{ $labels.instance }}"
      description: "No successful backup in the last 24 hours"

  - alert: BackupStorageLow
    expr: udaya_backup_storage_free_bytes / udaya_backup_storage_total_bytes < 0.2
    for: 30m
    labels:
      severity: warning
      service: backup
    annotations:
      summary: "Backup storage low on {{ $labels.instance }}"
      description: "Only {{ $value | printf \"%.2f\" }}% storage remaining"
```

### Alerting Configuration

```yaml
# Alertmanager configuration for backup alerts
route:
  group_by: ['alertname', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'backup-team'

receivers:
- name: 'backup-team'
  email_configs:
  - to: 'backup-alerts@udaya.org'
    from: 'alertmanager@udaya.org'
    smarthost: 'smtp.udaya.org:587'
    require_tls: true
  slack_configs:
  - channel: '#udaya-backup-alerts'
    api_url: 'https://hooks.slack.com/services/XXX'
  pagerduty_configs:
  - routing_key: 'udaya-backup-key'
    service_key: 'udaya-backup-service'

inhibit_rules:
- source_match:
    severity: 'critical'
  target_match:
    severity: 'warning'
  equal: ['alertname', 'instance']
```

## Security Considerations

### Encryption Requirements
- **At Rest**: AES-256 encryption for all backup files
- **In Transit**: TLS 1.3 for all backup transfers
- **Key Management**: Hardware Security Modules (HSM) for master keys
- **Key Rotation**: Quarterly rotation of encryption keys

### Access Control
- **Backup Access**: Limited to backup team and disaster recovery team
- **Restore Access**: Requires dual approval for critical systems
- **Audit Logging**: All backup/restore operations logged
- **Multi-Factor Authentication**: Required for backup system access

### Compliance Requirements
- **GDPR**: Personal data handling procedures
- **HIPAA**: Health data protection (if applicable)
- **PCI DSS**: Payment data protection (if applicable)
- **SOC 2**: Security and availability controls

## Compliance and Retention

### Retention Policy Implementation

```bash
# Automated retention enforcement
#!/bin/bash

# Daily backup retention (30 days)
find /backups/udaya/daily -name "*.gpg" -mtime +30 -exec rm -f {} \;

# Weekly backup retention (12 weeks)
find /backups/udaya/weekly -name "*.gpg" -mtime +84 -exec rm -f {} \;

# Monthly backup retention (12 months)
find /backups/udaya/monthly -name "*.gpg" -mtime +365 -exec rm -f {} \;

# Yearly backup retention (7 years)
find /backups/udaya/yearly -name "*.gpg" -mtime +2555 -exec rm -f {} \;

# Cloud storage lifecycle management
aws s3 lifecycle-config --bucket udaya-backups --rule-id "RetentionPolicy" \
    --expiration-days 2555 --transition-days 365 --storage-class GLACIER
```

### Audit and Compliance

```bash
# Backup audit script
#!/bin/bash

# Check backup completeness
echo "Checking backup completeness..."
find /backups/udaya -name "*.gpg" -mtime -1 | wc -l | grep -q "[1-9]" && echo "Daily backups present" || echo "Daily backups missing"

# Check backup encryption
echo "Checking backup encryption..."
find /backups/udaya -name "*.gpg" -exec gpg --verify {} \; | grep -q "Good signature" && echo "Backups properly encrypted" || echo "Encryption verification failed"

# Check cloud sync
echo "Checking cloud synchronization..."
aws s3 ls s3://udaya-backups/mainnet/ | wc -l | grep -q "[1-9]" && echo "Cloud backup present" || echo "Cloud backup missing"

# Generate compliance report
echo "Generating compliance report..."
/usr/local/bin/udaya-compliance-report.sh > /reports/backup_compliance_$(date +%Y%m%d).txt
```

## Appendix

### Backup Checklist

- [ ] Verify backup scripts are executable
- [ ] Test backup verification procedures
- [ ] Confirm cloud storage credentials
- [ ] Check encryption keys are accessible
- [ ] Validate retention policies
- [ ] Test restore procedures quarterly
- [ ] Document all backup locations
- [ ] Train team on disaster recovery
- [ ] Update documentation after changes
- [ ] Monitor backup success rates

### Common Issues and Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Backup fails | Insufficient disk space | Clean up old backups, increase storage |
| Verification fails | Corrupted backup file | Restore from secondary backup location |
| Slow backups | Network congestion | Schedule during off-peak hours |
| Encryption errors | Missing GPG keys | Verify key availability and permissions |
| Cloud sync fails | Credential issues | Rotate cloud credentials and retry |

### Contact Information

- **Backup Team**: backup@udaya.org
- **Disaster Recovery**: dr@udaya.org
- **Security Team**: security@udaya.org
- **24/7 Support**: +1 (800) UDYA-HELP