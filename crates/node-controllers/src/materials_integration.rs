use ecosafety_core::types::{LyapunovWeights, Residual, RiskVector};
use ecosafety_core::traits::SafeController;

use materials_plane::{compute_material_risks, MaterialsCorridors, MaterialKinetics};
use materials_plane::guard::{with_materials_plane, AntSafeSubstrateCorridorOk};

pub struct SubstrateConfig<S> {
    pub substrate: S,
    pub kinetics: MaterialKinetics,
    pub corridors: MaterialsCorridors,
}

impl<S> SubstrateConfig<S>
where
    S: AntSafeSubstrateCorridorOk,
{
    pub fn material_risks(&self) -> materials_plane::MaterialRisks {
        compute_material_risks(&self.kinetics, &self.corridors)
    }
}

pub struct CyboNodeController<S> {
    pub substrate_cfg: SubstrateConfig<S>,
}

impl<S> SafeController for CyboNodeController<S>
where
    S: AntSafeSubstrateCorridorOk,
{
    type State = crate::NodeState;
    type Actuation = crate::NodeActuation;

    fn propose_step(
        &mut self,
        state: &Self::State,
        _prev_residual: Residual,
        _weights: &LyapunovWeights,
    ) -> (Self::Actuation, RiskVector) {
        // Existing logic computes base RiskVector (without materials plane).
        let (act, mut rv_base) = self.propose_physical_step(state);

        // Compute materials risks and inject r_materials.
        let mat_risks = self.substrate_cfg.material_risks();
        let rv_full = with_materials_plane(rv_base, &mat_risks);

        (act, rv_full)
    }
}
