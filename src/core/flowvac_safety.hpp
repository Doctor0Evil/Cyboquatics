#pragma once

enum class FlowVacContext { URBAN, COASTAL };

struct FlowVacSafetyState {
    // CEIM/CPVM contaminant state (same order as EcoNet nodes)
    double C_PFBS_ngL;
    double C_Ecoli_MPN_100mL;
    double C_TP_mgL;
    double C_TDS_mgL;

    // Reference thresholds (C_ref) from CEIM/regs
    double Cref_PFBS_ngL;
    double Cref_Ecoli_MPN_100mL;
    double Cref_TP_mgL;
    double Cref_TDS_mgL;

    // Hydraulics and energy
    double Q_m3s;          // local discharge
    double v_ms;           // local velocity
    double E_avail_kWh;    // energy budget over horizon

    // Eco-safety indices
    double Kn_delta;       // expected CEIM node impact change
    double cpvm_value;     // CPVM safety/viability scalar
    double bio_stress_index; // 0–1 composite stress index

    FlowVacContext context; // URBAN or COASTAL
};
