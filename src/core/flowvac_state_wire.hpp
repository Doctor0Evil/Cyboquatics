#pragma once
#include <cstdint>

enum class FlowVacContext : std::uint8_t { URBAN = 0, COASTAL = 1 };

struct FlowVacStateWire final {
    // Context and flags (1 byte each where possible)
    std::uint8_t  context;          // FlowVacContext as uint8_t
    std::uint8_t  qr_ok;            // 1 = satisfies Quantum_Reflection, 0 = no
    std::uint8_t  reserved_flags1;
    std::uint8_t  reserved_flags2;

    // Contaminant concentrations (IEEE 754 double)
    double C_PFBS_ngL;
    double C_Ecoli_MPN_100mL;
    double C_TP_mgL;
    double C_TDS_mgL;

    // Reference thresholds
    double Cref_PFBS_ngL;
    double Cref_Ecoli_MPN_100mL;
    double Cref_TP_mgL;
    double Cref_TDS_mgL;

    // Hydraulics and energy
    double Q_m3s;
    double v_ms;
    double E_avail_kWh;

    // Eco-safety indices
    double Kn_delta;          // ΔK_n
    double cpvm_value;        // CPVM viability scalar
    double bio_stress_index;  // 0–1

    // Quantum_Reflection residuals (mass, energy) as explicit fields
    double qr_delta_mass;     // should be ≈ 0
    double qr_delta_energy;   // ≤ 0 when safe

    // Padding for future use (keep struct size stable for v1)
    std::uint8_t  reserved[16];
};

static_assert(sizeof(FlowVacStateWire) == 8 /*context+flags*/ +
                                       13 * sizeof(double) +
                                       16 /*reserved*/,
              "FlowVacStateWire size must be stable for v1");
static_assert(alignof(FlowVacStateWire) == alignof(double),
              "FlowVacStateWire alignment must match double");
