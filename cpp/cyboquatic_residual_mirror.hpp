// File: cpp/cyboquatic_residual_mirror.hpp

#pragma once

#include <array>
#include <cmath>

struct RiskVectorC {
    double r_energy;
    double r_hydraulics;
    double r_biology;
    double r_carbon;
    double r_materials;
    double r_biodiversity;
    double r_sigma;
};

struct LyapunovWeightsC {
    double w_energy;
    double w_hydraulics;
    double w_biology;
    double w_carbon;
    double w_materials;
    double w_biodiversity;
    double w_sigma;
};

inline double residual_vt(const RiskVectorC &rv, const LyapunovWeightsC &w) {
    auto sq = [](double x) { return x * x; };
    return w.w_energy * sq(rv.r_energy)
         + w.w_hydraulics * sq(rv.r_hydraulics)
         + w.w_biology * sq(rv.r_biology)
         + w.w_carbon * sq(rv.r_carbon)
         + w.w_materials * sq(rv.r_materials)
         + w.w_biodiversity * sq(rv.r_biodiversity)
         + w.w_sigma * sq(rv.r_sigma);
}

inline bool vt_non_increasing(double vt_prev, double vt_next) {
    return vt_next <= vt_prev;
}
