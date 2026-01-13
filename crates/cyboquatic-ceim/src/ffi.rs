use crate::ceim_core::{ceim_kn_for_node, CeimConfig, QpuNodeRow};

#[repr(C)]
pub struct CeimResult {
    pub Kn: f64,
}

#[no_mangle]
pub extern "C" fn ceim_kn_window(
    node: *const QpuNodeRow,
    cfg: *const CeimConfig,
    window_seconds: f64,
    out: *mut CeimResult,
) -> i32 {
    if node.is_null() || cfg.is_null() || out.is_null() {
        return 1;
    }
    let n = unsafe { &*node };
    let c = unsafe { &*cfg };
    let o = unsafe { &mut *out };

    o.Kn = ceim_kn_for_node(n, c, window_seconds);
    0
}
