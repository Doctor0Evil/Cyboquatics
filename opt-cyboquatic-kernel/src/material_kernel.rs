#[derive(Clone, Debug)]
pub struct MaterialStack {
    pub name: String,
    pub frac_bagasse: f64,
    pub frac_straw: f64,
    pub frac_starch: f64,
    pub frac_mineral: f64,
}

#[derive(Clone, Debug)]
pub struct MaterialCorridors {
    pub t90_max_days: f64,   // e.g., 180
    pub r_tox_max: f64,      // e.g., 0.10
    pub r_micro_max: f64,    // e.g., 0.05
}

#[derive(Clone, Debug)]
pub struct MaterialScore {
    pub t90_days: f64,
    pub r_t90: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub eco_impact: f64,
    pub corridor_ok: bool,
}

// Simple first‑order t90 model; replace k,Y,d with lab‑calibrated Phoenix values.
fn estimate_t90_days(k_day: f64) -> f64 {
    let k = k_day.max(1e-6);
    (10.0_f64.ln()) / k
}

// Example mapping from composition to baseline decay constant (very simplified)
fn k_from_stack(stack: &MaterialStack) -> f64 {
    let cellulosic = stack.frac_bagasse + stack.frac_straw;
    let starch = stack.frac_starch;
    0.03 * cellulosic + 0.07 * starch
}

pub fn score_material(
    stack: &MaterialStack,
    corridors: &MaterialCorridors,
    r_tox: f64,
    r_micro: f64,
) -> MaterialScore {
    let k = k_from_stack(stack);
    let t90 = estimate_t90_days(k);

    let r_t90 = (t90 / corridors.t90_max_days).min(1.0);

    // Eco‑impact rewards faster degradation inside corridor
    let eco_impact = if t90 <= corridors.t90_max_days && r_tox <= corridors.r_tox_max {
        0.90 + 0.10 * (1.0 - r_t90)
    } else {
        0.0
    };

    let corridor_ok =
        t90 <= corridors.t90_max_days &&
        r_tox <= corridors.r_tox_max &&
        r_micro <= corridors.r_micro_max;

    MaterialScore {
        t90_days: t90,
        r_t90,
        r_tox,
        r_micro,
        eco_impact,
        corridor_ok,
    }
}
