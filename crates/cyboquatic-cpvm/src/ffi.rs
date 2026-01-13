use crate::state::CpvmState;

#[repr(C)]
pub struct QpuNodeRow {
    pub baseline_Cin: f64,
    pub baseline_Cout: f64,
    pub Q_cms: f64,
    pub cref: f64,
}

#[repr(C)]
pub enum CpvmStatus {
    CPVM_STATUS_OK = 0,
    CPVM_STATUS_ERR = 1,
}

#[no_mangle]
pub extern "C" fn cpvm_step(
    node: *const QpuNodeRow,
    state_in: *const CpvmState,
    state_out: *mut CpvmState,
) -> CpvmStatus {
    if node.is_null() || state_in.is_null() || state_out.is_null() {
        return CpvmStatus::CPVM_STATUS_ERR;
    }
    let n = unsafe { &*node };
    let sin = unsafe { &*state_in };
    let sout = unsafe { &mut *state_out };

    let cin = n.baseline_Cin;
    let cout = n.baseline_Cout;
    let cref = n.cref;
    let q = n.Q_cms;

    let ratio = if cref > 0.0 { (cin - cout) / cref } else { 0.0 };
    let mass_load = ratio * q;

    sout.state_mass = sin.state_mass + mass_load;
    sout.state_risk = sin.state_risk + ratio;

    CpvmStatus::CPVM_STATUS_OK
}
