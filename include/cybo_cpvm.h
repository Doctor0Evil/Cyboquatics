#pragma once
#include <cstdint>
#include "cybo_qpudata.h"

extern "C" {

// C ABI status for Rust CPVM kernel
typedef enum {
    CPVM_STATUS_OK = 0,
    CPVM_STATUS_ERR = 1
} CpvmStatus;

typedef struct {
    double state_mass;
    double state_risk;
} CpvmState;

CpvmStatus cpvm_step(
    const cybo::QpuNodeRow* node,
    const CpvmState* state_in,
    CpvmState* state_out
);

} // extern "C"
