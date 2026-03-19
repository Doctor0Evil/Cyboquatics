#pragma once
#include "risk_coords.hpp"

namespace ecosafety {

struct MaterialKineticsInput {
    double t90_days;
    double toxicity_index;
    double micro_residue_mg_m2;
    double leachate_cec_ng_L;
    double pfas_residue_ng_L;
    double caloric_density_MJ_kg;
};

struct MaterialRisk {
    double r_t90;
    double r_tox;
    double r_micro;
};

MaterialRisk evaluate_material(const MaterialKineticsInput& in,
                               const CorridorTable& corridors);

} // namespace ecosafety
