#ifndef ECOSAFETY_KER_PLANE_ADJUST_HPP
#define ECOSAFETY_KER_PLANE_ADJUST_HPP

#include <algorithm>

struct PlaneUncertainty {
    double r_carbon_unc;
    double r_materials_unc;
    double r_biodiversity_unc;
    double r_hydraulics_unc;
    double r_data_quality_unc;
};

struct PlaneUncertaintyCaps {
    double rcarbon_unc_max_prod;
    double rmat_unc_max_prod;
    double rbio_unc_max_prod;
    double rhyd_unc_max_prod;
    double rdata_unc_max_prod;
};

struct DataQualityState {
    double r_calib_prev;
    double r_calib_next;
    double r_sigma_prev;
    double r_sigma_next;
};

struct KerTriple {
    double K_prev;
    double E_prev;
    double R_prev;
    double K_next;
    double E_next;
    double R_next;
};

inline bool uncertainty_caps_satisfied_prod(const PlaneUncertainty& u,
                                            const PlaneUncertaintyCaps& caps)
{
    return (u.r_carbon_unc      <= caps.rcarbon_unc_max_prod) &&
           (u.r_materials_unc   <= caps.rmat_unc_max_prod)   &&
           (u.r_biodiversity_unc<= caps.rbio_unc_max_prod)   &&
           (u.r_hydraulics_unc  <= caps.rhyd_unc_max_prod)   &&
           (u.r_data_quality_unc<= caps.rdata_unc_max_prod);
}

inline void apply_uncertainty_cap_to_K_and_E(const PlaneUncertainty& u,
                                             KerTriple& ker,
                                             double hard_cap = 0.85,
                                             double high_unc_threshold = 0.7)
{
    const double max_unc = std::max({
        u.r_carbon_unc,
        u.r_materials_unc,
        u.r_biodiversity_unc,
        u.r_hydraulics_unc,
        u.r_data_quality_unc
    });

    if (max_unc >= high_unc_threshold) {
        ker.K_next = std::min(ker.K_next, hard_cap);
        ker.E_next = std::min(ker.E_next, hard_cap);
    }
}

inline void apply_data_quality_invariants(const DataQualityState& dq,
                                          KerTriple& ker)
{
    const bool dq_worsened =
        (dq.r_calib_next > dq.r_calib_prev) ||
        (dq.r_sigma_next > dq.r_sigma_prev);

    if (dq_worsened) {
        ker.K_next = std::min(ker.K_next, ker.K_prev);
        ker.E_next = std::min(ker.E_next, ker.E_prev);
        ker.R_next = std::max(ker.R_next, ker.R_prev);
    }
}

#endif // ECOSAFETY_KER_PLANE_ADJUST_HPP
