use cyboquatic_ecosafety_core::*;
use econet_material_cybo::*;

struct NodeState {
    // Simplified: energy (J), hydraulic surcharge risk, biology risk, carbon, materials.
    energy_j: f64,
    r_hydraulic: f64,
    r_biology: f64,
    r_carbon: f64,
    material_kin: MaterialKinetics,
}

#[derive(Clone, Debug)]
struct NodeActuation {
    pump_power_kw: f64,
    valve_opening: f64,
}

struct CyboNodeController {
    material_corr: MaterialCorridors,
    material_weights: MaterialWeights,
    material_gate: DefaultSubstrateGate,
}

const PLANES: usize = 5; // energy, hydraulics, biology, carbon, materials

impl SafeController<PLANES> for CyboNodeController {
    type State = NodeState;
    type Actuation = NodeActuation;

    fn propose_step(
        &self,
        state: &Self::State,
    ) -> (Self::Actuation, RiskVector<PLANES>) {
        // Hard gate: refuse if substrate is unsafe.
        if !self
            .material_gate
            .corridor_ok(&state.material_kin, &self.material_corr)
        {
            // Force high material risk to steer ecosafety gate to Stop.
            let rv = RiskVector::new([
                RiskCoord::one(),
                RiskCoord::one(),
                RiskCoord::one(),
                RiskCoord::one(),
                RiskCoord::one(),
            ]);
            return (
                NodeActuation {
                    pump_power_kw: 0.0,
                    valve_opening: 0.0,
                },
                rv,
            );
        }

        let mat_risks = MaterialRisks::from_kinetics(&state.material_kin, &self.material_corr);
        let r_materials = mat_risks.r_materials(&self.material_weights);

        // Simple corridor mappings (in real nodes, use calibrated CorridorBands).
        let energy_corr = CorridorBands {
            safe_max: 0.0,
            gold_max: 5000.0,
            hard_max: 15000.0,
        };
        let energy_coord = energy_corr.normalize(state.energy_j);

        let r_h = RiskCoord::new_clamped(state.r_hydraulic);
        let r_b = RiskCoord::new_clamped(state.r_biology);
        let r_c = RiskCoord::new_clamped(state.r_carbon);

        let rv = RiskVector::new([
            energy_coord, // energy
            r_h,          // hydraulics
            r_b,          // biology
            r_c,          // carbon
            r_materials,  // materials
        ]);

        // Default actuation proposal (subject to ecosafety gate).
        let act = NodeActuation {
            pump_power_kw: 2.5,
            valve_opening: 0.7,
        };

        (act, rv)
    }

    fn apply_step(&mut self, state: &mut Self::State, act: &Self::Actuation) {
        // In production, this would call hardware drivers.
        state.energy_j += act.pump_power_kw * 3600.0;
    }
}

fn main() {
    let mut controller = CyboNodeController {
        material_corr: MaterialCorridors::default(),
        material_weights: MaterialWeights::default(),
        material_gate: DefaultSubstrateGate,
    };

    let mut state = NodeState {
        energy_j: 0.0,
        r_hydraulic: 0.2,
        r_biology: 0.1,
        r_carbon: 0.1,
        material_kin: MaterialKinetics {
            t90_days: 120.0,
            r_tox: 0.03,
            r_micro: 0.04,
            r_leach_cec: 0.05,
            r_pfas_resid: 0.01,
            caloric_density_kj_per_g: 3.0,
        },
    };

    let mut kernel = EcoSafetyKernel::<PLANES>::new(
        [1.0, 1.0, 1.0, 1.0, 1.5],
        0.001,
        0.05,
    );
    let mut ker_window = KerWindow::<PLANES>::new();

    for _ in 0..100 {
        let decision = ecosafety_step(&mut controller, &mut state, &mut kernel, &mut ker_window);
        if decision == CorridorDecision::Stop {
            break;
        }
    }

    let triad = ker_window.triad();
    println!(
        "KER: K={:.3}, E={:.3}, R={:.3}",
        triad.k_knowledge, triad.e_ecoimpact, triad.r_risk_of_harm
    );
}
