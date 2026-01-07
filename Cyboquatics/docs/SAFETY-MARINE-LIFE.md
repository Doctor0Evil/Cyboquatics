# Marine-Life Safety

Cyboquatic units must operate inside marine safety envelopes that bound flow, acoustic, EMF, light, and intake conditions to avoid harm to mammals, fish, turtles, and benthic species.

Design rules:

- Intake flow and gradients are capped using `MarineSafetyEnvelope` and validated in `intake_safety.rs`.
- Node layouts and eco-impact scores penalize any configuration that exceeds envelope-derived thresholds.
- Telemetry profiles require continuous monitoring of power, flow, and envelope violations for upstream CyberRank governance.
