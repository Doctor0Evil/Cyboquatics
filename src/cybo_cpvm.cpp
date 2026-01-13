#include "cybo_cpvm.h"

extern "C" {

CpvmStatus cpvm_step(
    const cybo::QpuNodeRow* node,
    const CpvmState* state_in,
    CpvmState* state_out
) {
    if (!node || !state_in || !state_out) return CPVM_STATUS_ERR;

    // Simple proxy dynamics: risk proportional to load ratio
    const double Cin  = node->baseline_Cin;
    const double Cout = node->baseline_Cout;
    const double Cref = node->cref;
    const double Q    = node->Q_cms;

    double ratio = 0.0;
    if (Cref > 0.0) {
        ratio = (Cin - Cout) / Cref;
    }
    double mass_load = ratio * Q;

    state_out->state_mass = state_in->state_mass + mass_load;
    state_out->state_risk = state_in->state_risk + ratio;

    return CPVM_STATUS_OK;
}

}
