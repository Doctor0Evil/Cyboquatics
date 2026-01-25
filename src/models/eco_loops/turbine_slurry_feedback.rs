// turbine_slurry_feedback.rs: Rust model for turbine-slurry feedback loop control
// DID-signed: bostrom1qxljm5f632vgw0dh3y8cqqrv4jes6grxn9zqda9ncv9yddgtyv4qs3mrx4
// Hex-stamp: 0x9876543210FEDCBA

#[derive(Clone, Debug)]
pub struct FeedbackParams {
    pub kp: f64,     // Proportional gain
    pub ki: f64,     // Integral gain
    pub kd: f64,     // Derivative gain
    pub c_set: f64,  // Target consistency (%)
    pub dt: f64,     // Time step (s)
    pub max_u: f64,  // Max adjustment (pitch/speed delta)
}

pub struct FeedbackState {
    pub integral: f64,
    pub prev_error: f64,
}

pub fn pid_step(state: &mut FeedbackState, params: &FeedbackParams, c_current: f64) -> f64 {
    let error = params.c_set - c_current;
    state.integral += error * params.dt;
    let derivative = (error - state.prev_error) / params.dt;
    let u = params.kp * error + params.ki * state.integral + params.kd * derivative;
    state.prev_error = error;
    u.clamp(-params.max_u, params.max_u)
}

pub fn simulate_loop(params: &FeedbackParams, initial_c: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let mut state = FeedbackState { integral: 0.0, prev_error: 0.0 };
    let mut results = Vec::with_capacity(steps);
    let mut c = initial_c;
    let mut t = 0.0;
    results.push((t, c, 0.0));

    for _ in 1..steps {
        let u = pid_step(&mut state, params, c);
        // Simplified plant: c changes toward set with gain 0.8 per step
        c += 0.8 * u * params.dt;
        c = c.clamp(0.5, 4.0); // Realistic bounds
        t += params.dt;
        results.push((t, c, u));
    }
    results
}

// Invariant: eco_impact > 0.92 if final error < 0.1% and max |u| < 0.5
pub fn eco_impact(results: &[(f64, f64, f64)], params: &FeedbackParams) -> f64 {
    let final_c = results.last().map(|&(_, c, _)| c).unwrap_or(0.0);
    let max_u = results.iter().map(|&(_, _, u)| u.abs()).fold(f64::NEG_INFINITY, f64::max);
    let error = (params.c_set - final_c).abs();
    if error < 0.1 && max_u < 0.5 {
        0.95 - error * 0.1
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_loop() {
        let params = FeedbackParams { kp: 2.0, ki: 0.5, kd: 0.1, c_set: 2.0, dt: 0.1, max_u: 0.5 };
        let results = simulate_loop(&params, 1.0, 200);
        let impact = eco_impact(&results, &params);
        assert!(impact > 0.92);
        assert!((results.last().unwrap().1 - 2.0).abs() < 0.1);
    }
}
