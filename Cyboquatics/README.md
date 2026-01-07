Cyboquatics is a cybernetic-engineering stack for **non-biomechanical** underwater recycling and water-remediation systems that operate without any direct integration with biological tissue or augmented-humans. The repository defines ALN particles, Rust core libraries, and C# simulators that model safe hydrokinetic power capture, contaminant removal, and telemetry routing with explicit marine-life safety envelopes.

Core goals:

- Encode cyboquatic safety and power profiles as first-class particles that can be ingested into Cybercore-Brain and The Great Perplexity via particles.export.manifest.json.
- Provide a Rust crate `cyboquatic-core` for hydrokinetic capture, intake-safety checks, and PFBS-class contaminant remediation with envelope-checked control laws.
- Provide a C# simulator `CyboquaticPowerSimulator` for coastal-node planning and QPU.Datashard-style CSV outputs that respect marine-life constraints and energy-closure rules.
- Supply CI pipelines that enforce zero-trust worklines, auditability, and deterministic builds for Rust and C# components.

This repository is intentionally non-biomechanical: all interfaces to marine ecosystems are via water flow, pressure, acoustic, chemical, and optical fields, never via implants, biologic anchors, or wetware controllers.
