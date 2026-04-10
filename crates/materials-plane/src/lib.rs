//! Cyboquatic Materials Plane v1
//! Spec ID: Cyboquatic.Ecosafety.MaterialsPlane.v1
//! Version: 1.0.0

mod types;
mod normalize;
mod guard;

pub use types::{MaterialKinetics, MaterialRisks};
pub use normalize::{compute_material_risks, MaterialsCorridors};
pub use guard::AntSafeSubstrateCorridorOk;
