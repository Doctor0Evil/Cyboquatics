# Brain-Identity Deployment & Operations Guide

**Version:** 1.0.0  
**Specification:** BrainIdentityCybo2026  
**Last Updated:** 2026-01-15  
**Classification:** Production Operations

---

## Table of Contents

1. [System Architecture Overview](#system-architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Deployment Steps](#deployment-steps)
4. [Configuration](#configuration)
5. [Monitoring & Observability](#monitoring--observability)
6. [Backup & Recovery](#backup--recovery)
7. [Security Hardening](#security-hardening)
8. [Performance Tuning](#performance-tuning)
9. [Incident Response](#incident-response)
10. [Upgrade Procedures](#upgrade-procedures)

---

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Brain-Identity Ecosystem                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Rust Core  │    │  ALN Kernel  │    │  C++ Mirror  │      │
│  │   (Safety)   │◄──►│  (Validation)│◄──►│  (Numeric)   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         │                   │                   │               │
│         └───────────────────┼───────────────────┘               │
│                             ▼                                   │
│                  ┌─────────────────────┐                        │
│                  │  qpudatashards CSV  │                        │
│                  │  (Audit Trail)      │                        │
│                  └─────────────────────┘                        │
│                             │                                   │
│         ┌───────────────────┼───────────────────┐               │
│         ▼                   ▼                   ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Kotlin/Andr. │    │   API Layer  │    │  Governance  │      │
│  │   (Mobile)   │    │   (REST)     │    │   (Voting)   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Component Dependencies

| Component | Language | Purpose | Criticality |
|-----------|----------|---------|-------------|
| cyboquatic_brain_identity_core | Rust | Core safety logic | Critical |
| cyboquatic_bi_integration | Rust | Physical+BI integration | Critical |
| cyboquatic_bi_simulation | Rust | Testing & validation | High |
| BrainIdentityCybo2026.aln | ALN | Validation rules | Critical |
| brain_identity_validation.lua | Lua | Dynamic kernels | High |
| brain_identity_residual.hpp | C++ | Numeric mirror | Medium |
| BrainIdentityShardManager.kt | Kotlin | Mobile integration | Medium |
| cyboquatic_bi_tests | Rust | Test suite | High |

---

## Prerequisites

### Hardware Requirements

| Environment | CPU | RAM | Storage | Network |
|-------------|-----|-----|---------|---------|
| Development | 4 cores | 8 GB | 50 GB SSD | 100 Mbps |
| Staging | 8 cores | 16 GB | 100 GB SSD | 1 Gbps |
| Production | 16 cores | 32 GB | 500 GB SSD | 10 Gbps |

### Software Requirements

```bash
# Rust toolchain
rustc >= 1.75.0
cargo >= 1.75.0

# Build dependencies
build-essential
libssl-dev
pkg-config

# Optional (for mobile)
Android SDK >= 33
Kotlin >= 1.9.0

# Optional (for C++ integration)
g++ >= 11.0
cmake >= 3.20
```

### Environment Verification

```bash
# Verify Rust installation
rustc --version
cargo --version

# Verify build tools
gcc --version
make --version

# Clone and build
git clone https://github.com/econet/cyboquatic-bi.git
cd cyboquatic-bi
cargo build --release

# Run test suite
cargo test --release
```

---

## Deployment Steps

### Step 1: Repository Setup

```bash
# Clone main repository
git clone https://github.com/econet/cyboquatic-bi.git
cd cyboquatic-bi

# Initialize submodules for ALN kernels
git submodule update --init --recursive

# Verify directory structure
ls -la crates/
ls -la qpudatashards/
ls -la c_kernels/
```

### Step 2: Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit configuration
vim .env

# Required environment variables:
# - BI_API_BASE_URL
# - BI_DATABASE_URL
# - BI_ENCRYPTION_KEY
# - BI_AUDIT_LOG_PATH
# - BI_KER_THRESHOLD_K
# - BI_KER_THRESHOLD_E
# - BI_KER_THRESHOLD_R
# - BI_KARMA_NONSLASH_ENFORCED
```

### Step 3: Database Initialization

```sql
-- Create Brain-Identity shards table
CREATE TABLE brain_identity_shards (
    brainidentityid VARCHAR(64) PRIMARY KEY,
    hexstamp VARCHAR(64) NOT NULL,
    ecoimpactscore FLOAT NOT NULL CHECK (ecoimpactscore >= 0.0 AND ecoimpactscore <= 1.0),
    neurorights_status INTEGER NOT NULL CHECK (neurorights_status IN (0, 1, 2)),
    karma_floor FLOAT NOT NULL CHECK (karma_floor >= 0.0),
    data_sensitivity_level INTEGER NOT NULL CHECK (data_sensitivity_level BETWEEN 1 AND 5),
    evidence_mode INTEGER NOT NULL CHECK (evidence_mode IN (0, 1, 2)),
    rsoul_residual FLOAT NOT NULL CHECK (rsoul_residual >= 0.0 AND rsoul_residual <= 1.0),
    social_exposure_coord FLOAT NOT NULL CHECK (social_exposure_coord >= 0.0 AND social_exposure_coord <= 1.0),
    timestamp_unix BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create audit log table
CREATE TABLE bi_audit_log (
    audit_entry_id VARCHAR(64) PRIMARY KEY,
    brainidentityid VARCHAR(64) NOT NULL REFERENCES brain_identity_shards(brainidentityid),
    timestamp_unix BIGINT NOT NULL,
    vt_previous FLOAT NOT NULL,
    vt_current FLOAT NOT NULL,
    vt_delta FLOAT NOT NULL,
    decision VARCHAR(32) NOT NULL,
    karma_floor_before FLOAT NOT NULL,
    karma_floor_after FLOAT NOT NULL,
    ker_deployable BOOLEAN NOT NULL,
    hexstamp VARCHAR(64) NOT NULL,
    INDEX idx_brainidentityid (brainidentityid),
    INDEX idx_timestamp (timestamp_unix),
    INDEX idx_decision (decision)
);

-- Create KER window table
CREATE TABLE bi_ker_windows (
    brainidentityid VARCHAR(64) PRIMARY KEY REFERENCES brain_identity_shards(brainidentityid),
    steps INTEGER NOT NULL DEFAULT 0,
    safe_steps INTEGER NOT NULL DEFAULT 0,
    max_r FLOAT NOT NULL DEFAULT 0.0,
    karma_preserved BOOLEAN NOT NULL DEFAULT true,
    window_start_unix BIGINT NOT NULL,
    window_end_unix BIGINT,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Step 4: Build and Deploy

```bash
# Build release binaries
cargo build --release --package cyboquatic_brain_identity_core
cargo build --release --package cyboquatic_bi_integration
cargo build --release --package cyboquatic_bi_simulation

# Copy binaries to deployment directory
cp target/release/cyboquatic_bi_* /opt/cyboquatic/bin/

# Set permissions
chmod 755 /opt/cyboquatic/bin/*
chown cyboquatic:cyboquatic /opt/cyboquatic/bin/*

# Deploy ALN kernels
cp qpudatashards/specs/BrainIdentityCybo2026.aln /opt/cyboquatic/kernels/
cp qpudatashards/kernels/brain_identity_validation.lua /opt/cyboquatic/kernels/

# Deploy C++ numerics
cp c_kernels/include/brain_identity_residual.hpp /opt/cyboquatic/include/

# Start services
systemctl start cyboquatic-bi-api
systemctl start cyboquatic-bi-validator
systemctl enable cyboquatic-bi-api
systemctl enable cyboquatic-bi-validator
```

### Step 5: Health Check

```bash
# Verify API endpoint
curl -s https://api.econet.cyboquatic.org/v1/bi/health | jq

# Expected response:
# {
#   "status": "healthy",
#   "version": "1.0.0",
#   "timestamp": 1704067200,
#   "components": {
#     "database": "connected",
#     "aln_kernel": "loaded",
#     "audit_log": "writing",
#     "ker_tracker": "active"
#   }
# }

# Run integration tests
/opt/cyboquatic/bin/cyboquatic_bi_tests --release

# Verify shard creation
curl -X POST https://api.econet.cyboquatic.org/v1/bi/shards \
  -H "Authorization: Bearer $API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"initial_karma": 100.0, "data_sensitivity_level": 2}'
```

---

## Configuration

### Production Configuration Template

```toml
# /opt/cyboquatic/config/production.toml

[general]
environment = "production"
log_level = "warn"
audit_enabled = true
hexstamp_required = true

[safety]
epsilon = 0.001
karma_nonslash_enforced = true
vt_stability_required = true
ker_deployable_required = true

[ker_thresholds]
k_min = 0.90
e_min = 0.90
r_max = 0.13

[weights]
w_energy = 0.15
w_hydraulic = 0.10
w_biology = 0.10
w_carbon = 0.15
w_materials = 0.10
w_neurorights = 0.15
w_soul = 0.10
w_social = 0.08
w_ecoimpact = 0.07

[database]
url = "postgresql://cyboquatic:***@localhost:5432/cyboquatic_bi"
pool_size = 20
timeout_seconds = 30

[audit]
log_path = "/var/log/cyboquatic/bi_audit.log"
rotation = "daily"
retention_days = 365
compression = true

[api]
bind_address = "0.0.0.0"
port = 8443
tls_enabled = true
tls_cert_path = "/etc/ssl/cyboquatic/api.crt"
tls_key_path = "/etc/ssl/cyboquatic/api.key"
rate_limit_per_minute = 1000
rate_limit_per_day = 100000

[governance]
quorum_minimum = 0.51
pass_threshold = 0.67
veto_karma_floor_min = 200.0
proposal_execution_delay_hours = 24
```

### Environment-Specific Overrides

```bash
# Development overrides
cp /opt/cyboquatic/config/production.toml /opt/cyboquatic/config/development.toml
# Edit: log_level = "debug", karma_nonslash_enforced = false (for testing)

# Staging overrides
cp /opt/cyboquatic/config/production.toml /opt/cyboquatic/config/staging.toml
# Edit: log_level = "info", rate_limit_per_minute = 500
```

---

## Monitoring & Observability

### Metrics to Track

| Metric | Type | Threshold | Alert Level |
|--------|------|-----------|-------------|
| `bi_vt_current` | Gauge | > 0.5 | Warning |
| `bi_vt_delta` | Gauge | > 0.01 | Critical |
| `bi_karma_violations` | Counter | > 0 | Critical |
| `bi_ker_deployable_ratio` | Gauge | < 0.90 | Warning |
| `bi_audit_log_latency_ms` | Histogram | > 100 | Warning |
| `bi_api_request_latency_ms` | Histogram | > 500 | Warning |
| `bi_shard_count` | Gauge | N/A | Info |
| `bi_active_identities` | Gauge | N/A | Info |

### Prometheus Configuration

```yaml
# /etc/prometheus/prometheus.yml

scrape_configs:
  - job_name: 'cyboquatic-bi'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s

rule_files:
  - '/etc/prometheus/rules/bi_alerts.yml'
```

### Alert Rules

```yaml
# /etc/prometheus/rules/bi_alerts.yml

groups:
  - name: brain_identity_alerts
    rules:
      - alert: KarmaViolationDetected
        expr: increase(bi_karma_violations_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Karma non-slashing violation detected"
          description: "A karma violation was attempted on brain identity {{ $labels.brainidentityid }}"

      - alert: VTInstability
        expr: bi_vt_delta > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Lyapunov residual instability detected"
          description: "Vt delta exceeded threshold on {{ $labels.brainidentityid }}"

      - alert: KERThresholdBreach
        expr: bi_ker_deployable_ratio < 0.90
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "KER deployability below threshold"
          description: "Only {{ $value | humanizePercentage }} of identities are KER-deployable"
```

### Grafana Dashboard

Import dashboard ID `cyboquatic-bi-ops-2026` for pre-built Brain-Identity monitoring panels including:
- Real-time Vt tracking per identity
- KARMA floor evolution over time
- KER metric trends (K, E, R)
- Decision distribution (Accept/Derate/Stop)
- Audit log throughput
- API latency percentiles

---

## Backup & Recovery

### Backup Schedule

| Data Type | Frequency | Retention | Storage |
|-----------|-----------|-----------|---------|
| Shard Database | Hourly | 7 days | Local SSD |
| Shard Database | Daily | 30 days | S3 Glacier |
| Audit Logs | Daily | 365 days | S3 Standard |
| ALN Kernels | On Change | Permanent | Git + S3 |
| Configuration | On Change | Permanent | Git + S3 |

### Backup Script

```bash
#!/bin/bash
# /opt/cyboquatic/scripts/backup.sh

set -euo pipefail

BACKUP_DIR="/var/backups/cyboquatic-bi"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
S3_BUCKET="s3://econet-backups/cyboquatic-bi"

# Create backup directory
mkdir -p "${BACKUP_DIR}/${TIMESTAMP}"

# Dump database
pg_dump -h localhost -U cyboquatic cyboquatic_bi | \
  gzip > "${BACKUP_DIR}/${TIMESTAMP}/database.sql.gz"

# Copy audit logs
tar -czf "${BACKUP_DIR}/${TIMESTAMP}/audit_logs.tar.gz" \
  /var/log/cyboquatic/bi_audit.log

# Copy configuration
cp /opt/cyboquatic/config/*.toml "${BACKUP_DIR}/${TIMESTAMP}/"

# Upload to S3
aws s3 sync "${BACKUP_DIR}/${TIMESTAMP}" "${S3_BUCKET}/${TIMESTAMP}"

# Cleanup local backups older than 7 days
find "${BACKUP_DIR}" -type d -mtime +7 -exec rm -rf {} \;

echo "Backup completed: ${TIMESTAMP}"
```

### Recovery Procedure

```bash
# 1. Stop services
systemctl stop cyboquatic-bi-api
systemctl stop cyboquatic-bi-validator

# 2. Download backup from S3
aws s3 sync s3://econet-backups/cyboquatic-bi/20260115_030000 /var/backups/cyboquatic-bi/restore

# 3. Restore database
gunzip -c /var/backups/cyboquatic-bi/restore/database.sql.gz | \
  psql -h localhost -U cyboquatic cyboquatic_bi

# 4. Restore audit logs
tar -xzf /var/backups/cyboquatic-bi/restore/audit_logs.tar.gz -C /

# 5. Restore configuration
cp /var/backups/cyboquatic-bi/restore/*.toml /opt/cyboquatic/config/

# 6. Verify integrity
/opt/cyboquatic/bin/cyboquatic_bi_tests --verify-shards

# 7. Restart services
systemctl start cyboquatic-bi-validator
systemctl start cyboquatic-bi-api

# 8. Verify health
curl https://api.econet.cyboquatic.org/v1/bi/health
```

---

## Security Hardening

### TLS Configuration

```nginx
# /etc/nginx/conf.d/cyboquatic-bi.conf

server {
    listen 443 ssl http2;
    server_name api.econet.cyboquatic.org;

    ssl_certificate /etc/ssl/cyboquatic/api.crt;
    ssl_certificate_key /etc/ssl/cyboquatic/api.key;
    ssl_protocols TLSv1.3;
    ssl_ciphers TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;

    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;

    location /v1/bi/ {
        proxy_pass http://localhost:8443;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Access Control

```bash
# Create dedicated service user
useradd -r -s /bin/false cyboquatic

# Set file permissions
chown -R cyboquatic:cyboquatic /opt/cyboquatic
chmod 750 /opt/cyboquatic/bin
chmod 640 /opt/cyboquatic/config/*.toml

# Database user with minimal privileges
psql -U postgres -c "CREATE USER cyboquatic WITH PASSWORD '***';"
psql -U postgres -c "GRANT CONNECT ON DATABASE cyboquatic_bi TO cyboquatic;"
psql -U postgres -c "GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO cyboquatic;"
```

### API Token Security

```rust
// Token generation example
use jsonwebtoken::{encode, Algorithm, Header};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,  // brainidentityid
    exp: usize,   // expiration timestamp
    scope: Vec<String>,
    iat: usize,
}

fn generate_token(brainidentityid: &str, scope: Vec<String>) -> String {
    let claims = Claims {
        sub: brainidentityid.to_string(),
        exp: (chrono::Utc::now().timestamp() as usize) + 3600,
        scope,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    encode(
        &Header::new(Algorithm::EdDSA),
        &claims,
        &EncodingKey::from_secret(ENV["BI_ENCRYPTION_KEY"].as_bytes())
    ).unwrap()
}
```

### Audit Log Integrity

```bash
# Enable audit log signing
/opt/cyboquatic/bin/cyboquatic_bi_signer --enable --key-path /etc/cyboquatic/audit_signing.key

# Verify audit log integrity
/opt/cyboquatic/bin/cyboquatic_bi_signer --verify --log-path /var/log/cyboquatic/bi_audit.log

# Expected output: "Audit log integrity verified: 100% valid"
```

---

## Performance Tuning

### Database Optimization

```sql
-- Create optimized indexes
CREATE INDEX CONCURRENTLY idx_shards_karma_floor ON brain_identity_shards(karma_floor);
CREATE INDEX CONCURRENTLY idx_shards_neurorights ON brain_identity_shards(neurorights_status);
CREATE INDEX CONCURRENTLY idx_audit_timestamp_desc ON bi_audit_log(timestamp_unix DESC);
CREATE INDEX CONCURRENTLY idx_audit_decision ON bi_audit_log(decision);

-- Analyze tables for query optimization
ANALYZE brain_identity_shards;
ANALYZE bi_audit_log;
ANALYZE bi_ker_windows;

-- Configure connection pooling
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
SELECT pg_reload_conf();
```

### Application Tuning

```toml
# /opt/cyboquatic/config/performance.toml

[threading]
worker_threads = 16
max_blocking_threads = 8

[cache]
shard_cache_size_mb = 512
ker_cache_size_mb = 128
cache_ttl_seconds = 300

[batching]
audit_log_batch_size = 100
audit_log_flush_interval_ms = 500
ker_update_batch_size = 50

[compression]
audit_log_compression = true
compression_level = 6
```

### Load Testing

```bash
# Install cargo-bench
cargo install cargo-bench

# Run load tests
cargo bench --package cyboquatic_bi_tests --bench load_test

# Expected results:
# - 10,000 requests/second sustained
# - p99 latency < 50ms
# - 0% error rate at 5,000 concurrent users
```

---

## Incident Response

### Severity Levels

| Level | Description | Response Time | Escalation |
|-------|-------------|---------------|------------|
| P0 | System down, data loss | 15 minutes | Immediate |
| P1 | Critical safety violation | 30 minutes | 1 hour |
| P2 | Degraded performance | 2 hours | 4 hours |
| P3 | Non-critical bug | 24 hours | 48 hours |

### Runbook: Karma Violation Detected

```bash
# 1. Identify affected identity
grep "StopKarmaViolation" /var/log/cyboquatic/bi_audit.log | tail -1

# 2. Check identity state
curl https://api.econet.cyboquatic.org/v1/bi/shards/{brainidentityid} \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 3. Review audit trail
curl https://api.econet.cyboquatic.org/v1/bi/shards/{brainidentityid}/audit \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 4. Verify invariant held
/opt/cyboquatic/bin/cyboquatic_bi_tests --verify-karma {brainidentityid}

# 5. Document incident
# Create incident report in /var/log/cyboquatic/incidents/

# 6. Notify stakeholders
# Send alert to security@econet.cyboquatic.org
```

### Runbook: Vt Instability

```bash
# 1. Check current Vt values
curl https://api.econet.cyboquatic.org/v1/bi/metrics/vt \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 2. Identify unstable identities
grep "vt_delta.*0.01" /var/log/cyboquatic/bi_audit.log | tail -20

# 3. Temporarily increase epsilon (if needed)
# Edit /opt/cyboquatic/config/production.toml
# epsilon = 0.005  # Temporary increase

# 4. Restart validator
systemctl restart cyboquatic-bi-validator

# 5. Monitor for stabilization
watch -n 5 'curl -s https://api.econet.cyboquatic.org/v1/bi/metrics/vt | jq'

# 6. Restore original epsilon after stabilization
```

### Communication Templates

```
Subject: [INCIDENT] Brain-Identity System - {Severity} - {Brief Description}

Team,

Incident detected at: {timestamp}
Affected component: {component}
Current status: {investigating/identified/mitigating/resolved}

Impact:
- {description of user impact}
- {number of affected identities}

Actions taken:
1. {action}
2. {action}

Next update: {time}

Incident commander: {name}
```

---

## Upgrade Procedures

### Pre-Upgrade Checklist

- [ ] Backup all databases and audit logs
- [ ] Notify stakeholders of maintenance window
- [ ] Prepare rollback plan
- [ ] Test upgrade in staging environment
- [ ] Verify compatibility matrix
- [ ] Document current version and configuration

### Upgrade Steps

```bash
# 1. Stop services
systemctl stop cyboquatic-bi-api
systemctl stop cyboquatic-bi-validator

# 2. Backup current binaries
cp -r /opt/cyboquatic/bin /opt/cyboquatic/bin.backup.$(date +%Y%m%d)

# 3. Download new release
curl -L https://github.com/econet/cyboquatic-bi/releases/download/v1.1.0/cyboquatic-bi-1.1.0.tar.gz | \
  tar -xzf - -C /opt/cyboquatic/

# 4. Run migration scripts
/opt/cyboquatic/bin/cyboquatic_bi_migrate --from 1.0.0 --to 1.1.0

# 5. Verify migration
/opt/cyboquatic/bin/cyboquatic_bi_tests --verify-migration

# 6. Start services
systemctl start cyboquatic-bi-validator
systemctl start cyboquatic-bi-api

# 7. Health check
curl https://api.econet.cyboquatic.org/v1/bi/health

# 8. Monitor for 24 hours
# Watch metrics dashboard for anomalies
```

### Rollback Procedure

```bash
# 1. Stop services
systemctl stop cyboquatic-bi-api
systemctl stop cyboquatic-bi-validator

# 2. Restore binaries
rm -rf /opt/cyboquatic/bin
cp -r /opt/cyboquatic/bin.backup.20260115 /opt/cyboquatic/bin

# 3. Restore database from backup
# Follow recovery procedure in Backup & Recovery section

# 4. Restart services
systemctl start cyboquatic-bi-validator
systemctl start cyboquatic-bi-api

# 5. Verify rollback
curl https://api.econet.cyboquatic.org/v1/bi/health
```

### Version Compatibility Matrix

| Version | Rust | ALN | Database | API |
|---------|------|-----|----------|-----|
| 1.0.0 | 1.75+ | 2.0+ | PostgreSQL 14+ | v1 |
| 1.1.0 | 1.75+ | 2.1+ | PostgreSQL 14+ | v1 |
| 2.0.0 | 1.80+ | 3.0+ | PostgreSQL 15+ | v2 |

---

## Appendix A: Quick Reference Commands

```bash
# Health check
curl https://api.econet.cyboquatic.org/v1/bi/health

# Create shard
curl -X POST https://api.econet.cyboquatic.org/v1/bi/shards \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"initial_karma": 100.0}'

# Evaluate step
curl -X POST https://api.econet.cyboquatic.org/v1/bi/shards/{id}/evaluate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"proposed_karma": 101.0}'

# Export CSV
curl -X POST https://api.econet.cyboquatic.org/v1/bi/export/csv \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"from": 1704067200, "to": 1704153600}'

# Run tests
cargo test --package cyboquatic_bi_tests --release

# Check logs
tail -f /var/log/cyboquatic/bi_audit.log

# View metrics
curl https://api.econet.cyboquatic.org/v1/bi/metrics
```

---

## Appendix B: Contact Information

| Role | Contact | Escalation |
|------|---------|------------|
| On-Call Engineer | oncall@econet.cyboquatic.org | PagerDuty |
| Security Team | security@econet.cyboquatic.org | Immediate |
| Platform Lead | platform-lead@econet.cyboquatic.org | P1+ |
| CTO | cto@econet.cyboquatic.org | P0 only |

---

**Document Control:**  
**Owner:** Platform Engineering  
**Review Cycle:** Quarterly  
**Last Review:** 2026-01-15  
**Next Review:** 2026-04-15
