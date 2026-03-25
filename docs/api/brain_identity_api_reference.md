# Brain-Identity API Reference

**Version:** 1.0.0  
**Specification:** BrainIdentityCybo2026  
**Last Updated:** 2026-01-15  
**Status:** Production-Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Core Endpoints](#core-endpoints)
4. [Data Structures](#data-structures)
5. [Error Codes](#error-codes)
6. [Rate Limits](#rate-limits)
7. [Webhooks](#webhooks)
8. [SDK Examples](#sdk-examples)

---

## Overview

The Brain-Identity API provides programmatic access to augmented citizen identity shards within the Cyboquatic ecosafety ecosystem. All operations enforce Lyapunov stability constraints (Vt non-increase), KER corridor compliance (K ≥ 0.90, E ≥ 0.90, R ≤ 0.13), and karma non-slashing invariants.

### Base URL

```
Production: https://api.econet.cyboquatic.org/v1/bi
Staging:    https://staging-api.econet.cyboquatic.org/v1/bi
```

### Supported Formats

| Format | Content-Type |
|--------|--------------|
| JSON | `application/json` |
| CSV (RFC-4180) | `text/csv` |
| ALN Shard | `application/aln` |

---

## Authentication

All API requests require Bearer token authentication via the `Authorization` header.

```http
Authorization: Bearer <your_api_token>
```

### Token Acquisition

```http
POST /v1/auth/token
Content-Type: application/json

{
  "brainidentityid": "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
  "evidence_mode": "HASHONLY",
  "timestamp_unix": 1704067200
}
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "shard:read shard:write ker:read audit:read"
}
```

### Permission Scopes

| Scope | Description |
|-------|-------------|
| `shard:read` | Read identity shard data |
| `shard:write` | Update identity shard fields |
| `ker:read` | Access KER window metrics |
| `ker:write` | Submit KER evaluation results |
| `audit:read` | Query audit log entries |
| `audit:write` | Submit new audit entries |
| `governance:vote` | Participate in governance decisions |

---

## Core Endpoints

### GET /shards/{brainidentityid}

Retrieve a complete Brain-Identity shard by ID.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `brainidentityid` | string | Yes | 64-character hex identity ID |
| `evidence_mode` | string | No | REDACTED, HASHONLY, or FULLTRACE |

**Request:**

```http
GET /v1/bi/shards/7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b?evidence_mode=HASHONLY
Authorization: Bearer <token>
```

**Response (200 OK):**

```json
{
  "brainidentityid": "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
  "hexstamp": "3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a",
  "ecoimpactscore": 0.12,
  "neurorights_status": 0,
  "karma_floor": 100.00,
  "data_sensitivity_level": 2,
  "evidence_mode": 1,
  "rsoul_residual": 0.08,
  "social_exposure_coord": 0.15,
  "timestamp_unix": 1704067200,
  "version": 1
}
```

**Error Responses:**

| Code | Description |
|------|-------------|
| 404 | Shard not found |
| 403 | Insufficient evidence_mode permissions |
| 429 | Rate limit exceeded |

---

### POST /shards

Create a new Brain-Identity shard.

**Request:**

```http
POST /v1/bi/shards
Content-Type: application/json
Authorization: Bearer <token>

{
  "initial_karma": 100.0,
  "data_sensitivity_level": 2,
  "evidence_mode": "REDACTED",
  "neurorights_status": "Active"
}
```

**Response (201 Created):**

```json
{
  "brainidentityid": "8c3b9f4d0e2f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b",
  "hexstamp": "4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b",
  "karma_floor": 100.0,
  "created_at": 1704067200,
  "status": "active"
}
```

---

### PUT /shards/{brainidentityid}

Update mutable fields of an existing shard.

**Request:**

```http
PUT /v1/bi/shards/7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b
Content-Type: application/json
Authorization: Bearer <token>

{
  "ecoimpactscore": 0.11,
  "rsoul_residual": 0.07,
  "social_exposure_coord": 0.14,
  "proposed_karma": 100.50
}
```

**Response (200 OK):**

```json
{
  "brainidentityid": "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
  "hexstamp": "5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c",
  "karma_floor": 100.50,
  "karma_nonslash_verified": true,
  "updated_at": 1704153600
}
```

**Validation Rules:**

- `karma_floor` can only increase (non-slashing invariant)
- `neurorights_status` changes require FULLTRACE evidence_mode
- All risk coordinates must remain in [0.0, 1.0]

---

### POST /shards/{brainidentityid}/evaluate

Submit a safety evaluation step for Lyapunov and KER validation.

**Request:**

```http
POST /v1/bi/shards/7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b/evaluate
Content-Type: application/json
Authorization: Bearer <token>

{
  "physical_risk_vector": {
    "r_energy": 0.30,
    "r_hydraulic": 0.20,
    "r_biology": 0.25,
    "r_carbon": 0.35,
    "r_materials": 0.28
  },
  "vt_previous": 0.078,
  "proposed_karma": 100.50,
  "weights": {
    "w_energy": 0.15,
    "w_hydraulic": 0.10,
    "w_biology": 0.10,
    "w_carbon": 0.15,
    "w_materials": 0.10,
    "w_neurorights": 0.15,
    "w_soul": 0.10,
    "w_social": 0.08,
    "w_ecoimpact": 0.07
  }
}
```

**Response (200 OK):**

```json
{
  "decision": "Accept",
  "vt_current": 0.082,
  "vt_delta": 0.004,
  "vt_stable": true,
  "karma_nonslash_verified": true,
  "ker_deployable": true,
  "R_residual": 0.09,
  "K_score": 0.95,
  "E_score": 0.92,
  "audit_entry_id": "audit_7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c"
}
```

**Decision Values:**

| Decision | Description |
|----------|-------------|
| `Accept` | Step passes all safety gates |
| `Derate` | Vt increased but within epsilon; reduce activity |
| `Stop` | Hard corridor violation detected |
| `StopKarmaViolation` | Proposed karma would violate non-slashing invariant |

---

### GET /shards/{brainidentityid}/ker

Retrieve KER window metrics for a shard.

**Request:**

```http
GET /v1/bi/shards/7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b/ker
Authorization: Bearer <token>
```

**Response (200 OK):**

```json
{
  "brainidentityid": "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
  "steps": 1000,
  "safe_steps": 950,
  "K": 0.95,
  "E": 0.92,
  "R": 0.09,
  "karma_preserved": true,
  "deployable": true,
  "window_start": 1704067200,
  "window_end": 1704153600
}
```

---

### GET /shards/{brainidentityid}/audit

Query audit log entries for a shard.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `from` | integer | No | Start timestamp (unix) |
| `to` | integer | No | End timestamp (unix) |
| `limit` | integer | No | Max entries (default: 100) |
| `decision` | string | No | Filter by decision type |

**Request:**

```http
GET /v1/bi/shards/7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b/audit?from=1704067200&to=1704153600&limit=50
Authorization: Bearer <token>
```

**Response (200 OK):**

```json
{
  "entries": [
    {
      "audit_entry_id": "audit_7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c",
      "timestamp_unix": 1704067200,
      "vt_previous": 0.075,
      "vt_current": 0.078,
      "vt_delta": 0.003,
      "decision": "Accept",
      "karma_floor_before": 100.00,
      "karma_floor_after": 100.00,
      "ker_deployable": true,
      "hexstamp": "3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a"
    }
  ],
  "total": 1,
  "has_more": false
}
```

---

### POST /export/csv

Export shard data as RFC-4180 compliant CSV.

**Request:**

```http
POST /v1/bi/export/csv
Content-Type: application/json
Authorization: Bearer <token>

{
  "brainidentityids": [
    "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
    "8c3b9f4d0e2f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b"
  ],
  "fields": [
    "brainidentityid",
    "hexstamp",
    "ecoimpactscore",
    "karma_floor",
    "K_score",
    "E_score",
    "R_residual"
  ],
  "from": 1704067200,
  "to": 1704240000
}
```

**Response (200 OK):**

```
Content-Type: text/csv
Content-Disposition: attachment; filename="bi_export_20260101_20260103.csv"

node_id,brainidentityid,hexstamp,ecoimpactscore,karma_floor,K_score,E_score,R_residual
NODE_001,7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b,3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a,0.12,100.00,0.95,0.92,0.09
NODE_002,8c3b9f4d0e2f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b,4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b,0.08,75.00,0.97,0.95,0.06
```

---

## Data Structures

### BrainIdentityShard

```json
{
  "brainidentityid": "string (64 hex chars)",
  "hexstamp": "string (64 hex chars)",
  "ecoimpactscore": "float [0.0, 1.0]",
  "neurorights_status": "integer [0, 1, 2]",
  "karma_floor": "float >= 0.0",
  "data_sensitivity_level": "integer [1-5]",
  "evidence_mode": "integer [0, 1, 2]",
  "rsoul_residual": "float [0.0, 1.0]",
  "social_exposure_coord": "float [0.0, 1.0]",
  "timestamp_unix": "integer"
}
```

### RiskVector

```json
{
  "r_energy": "float [0.0, 1.0]",
  "r_hydraulic": "float [0.0, 1.0]",
  "r_biology": "float [0.0, 1.0]",
  "r_carbon": "float [0.0, 1.0]",
  "r_materials": "float [0.0, 1.0]",
  "r_neurorights": "float [0.0, 1.0]",
  "r_soul": "float [0.0, 1.0]",
  "r_social": "float [0.0, 1.0]",
  "r_ecoimpact": "float [0.0, 1.0]"
}
```

### KerWindow

```json
{
  "steps": "integer >= 0",
  "safe_steps": "integer >= 0",
  "K": "float [0.0, 1.0]",
  "E": "float [0.0, 1.0]",
  "R": "float [0.0, 1.0]",
  "karma_preserved": "boolean",
  "deployable": "boolean"
}
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 400 | BAD_REQUEST | Invalid request format or parameters |
| 401 | UNAUTHORIZED | Missing or invalid authentication |
| 403 | FORBIDDEN | Insufficient permissions for resource |
| 404 | NOT_FOUND | Resource does not exist |
| 409 | CONFLICT | Karma non-slashing violation |
| 422 | UNPROCESSABLE | Safety gate violation (Vt, KER, corridor) |
| 429 | RATE_LIMITED | Too many requests |
| 500 | INTERNAL_ERROR | Server-side error |
| 503 | UNAVAILABLE | Service temporarily unavailable |

### Error Response Format

```json
{
  "error": {
    "code": "KARMA_NONSLASH_VIOLATION",
    "message": "Proposed karma floor (95.0) is less than current (100.0)",
    "details": {
      "karma_before": 100.0,
      "karma_proposed": 95.0,
      "invariant": "karma_floor(t+1) >= karma_floor(t)"
    },
    "request_id": "req_7f8a9b0c1d2e3f4a"
  }
}
```

---

## Rate Limits

| Tier | Requests/Minute | Requests/Day | Burst |
|------|-----------------|--------------|-------|
| Free | 60 | 1,000 | 10 |
| Standard | 300 | 10,000 | 50 |
| Enterprise | 1,000 | 100,000 | 200 |

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 287
X-RateLimit-Reset: 1704067260
```

---

## Webhooks

Subscribe to real-time events for shard state changes.

### POST /webhooks

**Request:**

```http
POST /v1/bi/webhooks
Content-Type: application/json
Authorization: Bearer <token>

{
  "url": "https://your-server.com/webhooks/bi",
  "events": [
    "shard.created",
    "shard.updated",
    "ker.threshold_exceeded",
    "karma.violation_attempted",
    "neurorights.status_changed"
  ],
  "secret": "whsec_7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c"
}
```

### Webhook Payload

```json
{
  "id": "evt_7f8a9b0c1d2e3f4a",
  "type": "ker.threshold_exceeded",
  "created": 1704067200,
  "data": {
    "brainidentityid": "7b2a8f3c9d1e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b",
    "K": 0.88,
    "E": 0.87,
    "R": 0.15,
    "threshold": {
      "K_min": 0.90,
      "E_min": 0.90,
      "R_max": 0.13
    }
  }
}
```

### Signature Verification (Rust)

All webhooks include an `X-Webhook-Signature` header for HMAC-SHA256 verification.

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn verify_webhook(payload: &str, signature: &str, secret: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());
    expected == signature
}
```

---

## SDK Examples

### Rust

```rust
use cyboquatic_bi_client::{BiClient, BrainIdentityShard, BiSafeStepConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BiClient::new("https://api.econet.cyboquatic.org/v1/bi", "your_token")?;
    
    // Create new shard
    let shard = client.create_shard(100.0, 2, "REDACTED").await?;
    println!("Created shard: {}", shard.brainidentityid.to_hex());
    
    // Evaluate safety step
    let result = client.evaluate_step(
        &shard.brainidentityid,
        &physical_rv,
        vt_previous,
        proposed_karma,
        &BiSafeStepConfig { epsilon: 0.001, enforce_karma: true }
    ).await?;
    
    println!("Decision: {:?}", result.decision);
    
    Ok(())
}
```

### Kotlin/Android

```kotlin
val client = BiApiClient("https://api.econet.cyboquatic.org/v1/bi", "your_token")

// Load shard
val shard = client.loadShard(brainidentityid)

// Evaluate step
val result = client.evaluateStep(
    brainidentityid,
    physicalRv,
    vtPrevious,
    proposedKarma,
    BiSafeStepConfig(0.001f, true)
)

when (result.decision) {
    BiSafeStepDecision.Accept -> println("Step accepted")
    BiSafeStepDecision.Derate -> println("Step derated")
    BiSafeStepDecision.Stop -> println("Step stopped")
    BiSafeStepDecision.StopKarmaViolation -> println("Karma violation blocked")
}
```

### C++

```cpp
#include "bi_client.hpp"
#include <iostream>

int main() {
    bi::Client client("https://api.econet.cyboquatic.org/v1/bi", "your_token");
    
    // Create shard
    bi::ShardConfig config;
    config.initial_karma = 100.0f;
    config.sensitivity = 2;
    
    auto shard = client.create_shard(config);
    std::cout << "Created shard: " << shard.brainidentityid << std::endl;
    
    // Evaluate step
    bi::RiskVector physical_rv = {0.3f, 0.2f, 0.25f, 0.35f, 0.28f};
    bi::SafeStepConfig step_config = {0.001f, true};
    
    auto result = client.evaluate_step(
        shard.brainidentityid,
        physical_rv,
        0.078f,
        100.5f,
        step_config
    );
    
    if (result.decision == bi::Decision::Accept) {
        std::cout << "Step accepted" << std::endl;
    }
    
    return 0;
}
```

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-15 | Initial production release |
| 0.9.0 | 2025-12-01 | Added karma non-slashing enforcement |
| 0.8.0 | 2025-11-01 | Added neurorights status tracking |
| 0.7.0 | 2025-10-01 | Initial beta with basic shard CRUD |

---

## Support

- **Documentation:** https://docs.econet.cyboquatic.org/bi
- **Status Page:** https://status.econet.cyboquatic.org
- **Email:** api-support@econet.cyboquatic.org
- **Discord:** https://discord.gg/econet-cyboquatic
