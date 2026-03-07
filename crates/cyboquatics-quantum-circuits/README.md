# Cyboquatics Quantum Circuits

[![Crates.io](https://img.shields.io/crates/v/cyboquatics-quantum-circuits.svg)](https://crates.io/crates/cyboquatics-quantum-circuits)
[![Documentation](https://docs.rs/cyboquatics-quantum-circuits/badge.svg)](https://docs.rs/cyboquatics-quantum-circuits)
[![License](https://img.shields.io/crates/l/cyboquatics-quantum-circuits.svg)](LICENSE)
[![Evidence Hex](https://img.shields.io/badge/evidence-0xCQ2026QUANTUM9F8E7D6C-blue)](https://github.com/Doctor0Evil/Cyboquatics)

Quantum-learning circuit implementations for Cyboquatics governance, soul-boundary enforcement, and autonomous compliance verification.

## Overview

This crate provides quantum-safe encryption, variational quantum circuits for governance optimization, and quantum verification protocols for soul-boundary checks. All quantum operations are subordinate to:

- `soul.guardrail.spec.v1`
- `bio.safety.envelope.citizen.v1`
- `nanoswarm.compliance.field.v1`

## Features

- **Quantum-Safe Encryption**: Post-quantum cryptographic algorithms (Kyber-1024, Dilithium-5)
- **Governance Optimization**: Variational quantum circuits for parameter optimization
- **Soul-Boundary Verification**: Quantum verification protocols for ALN particle compliance
- **Audit Integration**: Automatic logging of quantum operations to blockchain audit trails
- **Classical Fallback**: Graceful degradation to classical verification if quantum backend unavailable

## Installation

```bash
cargo add cyboquatics-quantum-circuits
```

### Feature Flags

- `quantum-simulation` (default): Enable quantum simulation backend
- `quantum-hardware`: Enable quantum hardware integration (IBM Q, Rigetti, etc.)
- `quantum-verification`: Enable zero-knowledge quantum verification
- `full`: Enable all features

## Quick Start

```rust
use cyboquatics_quantum_circuits::{
    initialize_quantum_backend, verify_soul_boundaries_quantum,
    CyboquaticsQuantumConfig,
};

// Initialize quantum backend
let config = CyboquaticsQuantumConfig::default();
let backend = initialize_quantum_backend(&config)?;

// Verify soul boundaries
let result = verify_soul_boundaries_quantum(&backend, &particles, &guardrail)?;

if result.passed {
    println!("✅ All particles comply with soul guardrails");
} else {
    println!("❌ {} particles failed verification", result.failed_count);
}
```

## CLI Usage

```bash
# Verify soul boundaries
quantum-automation verify --particles ./aln/particles/ --guardrail ./aln/particles/soul.guardrail.spec.v1.aln

# Optimize governance
quantum-automation optimize --objective maximize-eco-impact --output ./reports/optimization.json

# Run compliance audit
quantum-automation audit --repository . --standards neurorights-2026,quantum-safe-governance

# Generate quantum keys
quantum-automation generate-keys --algorithm kyber-1024 --output-dir ./keys/

# Health check
quantum-automation health-check --endpoint https://quantum-backend.cyboquatics.ai
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Cyboquatics Quantum                       │
│                      Circuits Crate                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Encryption  │  │  Circuits   │  │   Verification      │  │
│  │  Module     │  │   Module    │  │     Module          │  │
│  │             │  │             │  │                     │  │
│  │ - Kyber-1024│  │ - Governance│  │ - Soul Boundary     │  │
│  │ - Dilithium │  │ - Optimization│  │ - Particle Check  │  │
│  │ - Falcon    │  │ - Variational│  │ - Proof Generation │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Quantum Backend Interface               │   │
│  │  (Simulation | IBM Q | Rigetti | IonQ | Classical)  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              ALN Particle Integration                │   │
│  │  (soul.guardrail | bio.envelope | karma.metric)     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Security Considerations

- All quantum operations are logged to immutable audit trails
- Soul-boundary verification cannot be bypassed, even with quantum backend failure
- Classical fallback path ensures continuous operation
- Key rotation occurs every 24 hours by default
- Minimum quantum safety threshold: 0.95

## Compliance

This crate complies with:

- Neurorights 2026 Framework
- NIST Post-Quantum Cryptography Standards
- ALN Particle Specification v1.0
- Quantum-Safe Governance Protocol

## Evidence Hex

**Build Evidence:** `0xCQ2026QUANTUM9F8E7D6C`

**Knowledge Factor:** `F ≈ 0.92`

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual-licensed as above, without any additional terms or conditions.

## Contact

- **Author:** Doctor Jacob Scott Farmer
- **DID:** `did:ion:EiD8J2b3K8k9Q8x9L7m2n4p1q5r6s7t8u9v0w1x2y3z4A5B6C7D8E9F0`
- **GitHub:** [Doctor0Evil/Cyboquatics](https://github.com/Doctor0Evil/Cyboquatics)
- **Evidence Registry:** [CyberNet Validation](https://api.cybernet.ai/v1/validation/0xCQ2026QUANTUM9F8E7D6C)
```
