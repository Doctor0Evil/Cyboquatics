use std::f64::consts::PI;

/// Small dense matrix type for low-order LTI models.
#[derive(Debug, Clone)]
pub struct Matrix {
    pub nrows: usize,
    pub ncols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn new(nrows: usize, ncols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), nrows * ncols);
        Matrix { nrows, ncols, data }
    }

    pub fn zeros(nrows: usize, ncols: usize) -> Self {
        Matrix {
            nrows,
            ncols,
            data: vec![0.0; nrows * ncols],
        }
    }

    #[inline]
    pub fn idx(&self, r: usize, c: usize) -> usize {
        r * self.ncols + c
    }

    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[self.idx(r, c)]
    }

    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        let idx = self.idx(r, c);
        self.data[idx] = v;
    }

    pub fn mul_vec(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.ncols);
        let mut y = vec![0.0; self.nrows];
        for i in 0..self.nrows {
            let mut acc = 0.0;
            for j in 0..self.ncols {
                acc += self.get(i, j) * x[j];
            }
            y[i] = acc;
        }
        y
    }

    pub fn add(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.nrows, other.nrows);
        assert_eq!(self.ncols, other.ncols);
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();
        Matrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data,
        }
    }

    pub fn scale(&self, alpha: f64) -> Matrix {
        let data = self.data.iter().map(|v| alpha * v).collect();
        Matrix {
            nrows: self.nrows,
            ncols: self.ncols,
            data,
        }
    }

    pub fn transpose(&self) -> Matrix {
        let mut out = Matrix::zeros(self.ncols, self.nrows);
        for i in 0..self.nrows {
            for j in 0..self.ncols {
                out.set(j, i, self.get(i, j));
            }
        }
        out
    }
}

/// Basic vector helpers.
pub fn vec_add(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

pub fn vec_scale(a: &[f64], alpha: f64) -> Vec<f64> {
    a.iter().map(|x| alpha * x).collect()
}

pub fn vec_norm2(a: &[f64]) -> f64 {
    a.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// LTI cyboquatic surrogate model ẋ = A x + B u.
#[derive(Debug, Clone)]
pub struct CyboquaticLti {
    pub a: Matrix,
    pub b: Matrix,
}

impl CyboquaticLti {
    pub fn new(a: Matrix, b: Matrix) -> Self {
        assert_eq!(a.nrows, a.ncols, "A must be square");
        assert_eq!(b.nrows, a.nrows, "B must have same rows as A");
        CyboquaticLti { a, b }
    }

    /// One-step forward Euler integration for simplicity.
    /// For real deployments, a more accurate integrator can wrap this state.
    pub fn step(&self, x: &[f64], u: &[f64], dt: f64) -> Vec<f64> {
        let ax = self.a.mul_vec(x);
        let bu = self.b.mul_vec(u);
        let dx = vec_add(&ax, &bu);
        vec_add(x, &vec_scale(&dx, dt))
    }
}

/// Simple eigen-analysis wrapper for small 2x2 or 3x3 systems.
/// For now, this supports only 2x2 matrices for resonance insight.
#[derive(Debug, Clone)]
pub struct EigenPair {
    pub real: f64,
    pub imag: f64,
}

/// Compute eigenvalues of a 2x2 matrix A.
///
/// A = [a b; c d]
/// λ = (tr ± sqrt(tr^2 - 4 det)) / 2
pub fn eigenvalues_2x2(a: &Matrix) -> Option<(EigenPair, EigenPair)> {
    if a.nrows != 2 || a.ncols != 2 {
        return None;
    }
    let a11 = a.get(0, 0);
    let a12 = a.get(0, 1);
    let a21 = a.get(1, 0);
    let a22 = a.get(1, 1);

    let tr = a11 + a22;
    let det = a11 * a22 - a12 * a21;
    let disc = tr * tr - 4.0 * det;

    if disc >= 0.0 {
        let sqrt_disc = disc.sqrt();
        let l1 = (tr + sqrt_disc) / 2.0;
        let l2 = (tr - sqrt_disc) / 2.0;
        Some((
            EigenPair { real: l1, imag: 0.0 },
            EigenPair { real: l2, imag: 0.0 },
        ))
    } else {
        let sqrt_disc = (-disc).sqrt();
        let real = tr / 2.0;
        let imag = sqrt_disc / 2.0;
        Some((
            EigenPair { real, imag },
            EigenPair { real, imag: -imag },
        ))
    }
}

/// Classification of resonance bands for actuation.
#[derive(Debug, Clone)]
pub struct ResonanceBand {
    pub f_min_hz: f64,
    pub f_max_hz: f64,
}

/// Envelope constraints for safe resonance.
#[derive(Debug, Clone)]
pub struct ResonanceSafetyConfig {
    /// Maximum allowed state norm.
    pub max_state_norm: f64,
    /// Maximum allowed pollutant amplitude in any mode (index into x).
    pub pollutant_indices: Vec<usize>,
    pub max_pollutant: f64,
}

/// Result of safety evaluation at a given state.
#[derive(Debug, Clone)]
pub struct ResonanceSafetyStatus {
    pub is_safe: bool,
    pub state_norm: f64,
    pub max_pollutant_observed: f64,
}

/// Evaluate whether current state lies inside safe envelope.
pub fn evaluate_resonance_safety(
    x: &[f64],
    cfg: &ResonanceSafetyConfig,
) -> ResonanceSafetyStatus {
    let state_norm = vec_norm2(x);
    let mut max_p = 0.0;
    for &idx in &cfg.pollutant_indices {
        if let Some(val) = x.get(idx) {
            let abs_val = val.abs();
            if abs_val > max_p {
                max_p = abs_val;
            }
        }
    }
    let is_safe = state_norm <= cfg.max_state_norm && max_p <= cfg.max_pollutant;
    ResonanceSafetyStatus {
        is_safe,
        state_norm,
        max_pollutant_observed: max_p,
    }
}

/// Frequency-domain weight for actuation, used to avoid harmful bands
/// and encourage safe resonant mixing.
///
/// This is a simple analytic filter; a real implementation could operate
/// on FFTs of measured signals.
pub fn actuation_frequency_weight(
    freq_hz: f64,
    safe_bands: &[ResonanceBand],
    harmful_bands: &[ResonanceBand],
) -> f64 {
    let mut weight = 1.0;

    for band in harmful_bands {
        if freq_hz >= band.f_min_hz && freq_hz <= band.f_max_hz {
            // Strongly suppress harmful bands.
            weight *= 0.05;
        }
    }

    for band in safe_bands {
        if freq_hz >= band.f_min_hz && freq_hz <= band.f_max_hz {
            // Mildly boost safe mixing bands.
            weight *= 1.5;
        }
    }

    // Clamp to a reasonable range to avoid numerical issues.
    weight.clamp(0.01, 5.0)
}

/// Design a simple static feedback gain K for a 2x2 system using
/// pole placement at desired eigenvalues (real parts).
///
/// This is a placeholder algebraic design for low-order prototypes.
/// For production, a more robust solver or external optimization is recommended.
pub fn design_static_feedback_2x2(
    a: &Matrix,
    b: &Matrix,
    desired_real_part: f64,
) -> Option<Matrix> {
    if a.nrows != 2 || a.ncols != 2 || b.nrows != 2 || b.ncols != 1 {
        return None;
    }

    // For simplicity, we use heuristic tuning:
    // K = [k1 k2] such that A_cl = A + B K has trace near 2 * desired_real_part
    let a11 = a.get(0, 0);
    let a22 = a.get(1, 1);
    let tr_a = a11 + a22;

    let tr_target = 2.0 * desired_real_part;
    let delta_tr = tr_target - tr_a;

    // Assume B = [b1; b2], distribute correction by row magnitude.
    let b1 = b.get(0, 0);
    let b2 = b.get(1, 0);
    let sum_b = b1.abs() + b2.abs();
    if sum_b == 0.0 {
        return None;
    }

    let w1 = b1.abs() / sum_b;
    let w2 = b2.abs() / sum_b;

    let k1 = delta_tr * w1.signum();
    let k2 = delta_tr * w2.signum();

    Some(Matrix::new(1, 2, vec![k1, k2]))
}

/// Cyboquatic controller that wraps the LTI model, safety config,
/// and frequency shaping for actuators.
#[derive(Debug, Clone)]
pub struct CyboquaticController {
    pub model: CyboquaticLti,
    pub safety_cfg: ResonanceSafetyConfig,
    pub safe_bands: Vec<ResonanceBand>,
    pub harmful_bands: Vec<ResonanceBand>,
    /// Feedback gain K (m x n) such that u = -K x.
    pub k_feedback: Matrix,
}

impl CyboquaticController {
    pub fn new(
        model: CyboquaticLti,
        safety_cfg: ResonanceSafetyConfig,
        safe_bands: Vec<ResonanceBand>,
        harmful_bands: Vec<ResonanceBand>,
        k_feedback: Matrix,
    ) -> Self {
        CyboquaticController {
            model,
            safety_cfg,
            safe_bands,
            harmful_bands,
            k_feedback,
        }
    }

    /// Compute feedback control u = -K x.
    pub fn feedback_control(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(self.k_feedback.ncols, x.len());
        let mut u = vec![0.0; self.k_feedback.nrows];
        for i in 0..self.k_feedback.nrows {
            let mut acc = 0.0;
            for j in 0..self.k_feedback.ncols {
                acc += self.k_feedback.get(i, j) * x[j];
            }
            u[i] = -acc;
        }
        u
    }

    /// Apply a simple sinusoidal reference at frequency f_ref,
    /// filtered through safe/harmful bands.
    pub fn resonant_reference(
        &self,
        amplitude: f64,
        freq_hz: f64,
        t: f64,
    ) -> Vec<f64> {
        let w = actuation_frequency_weight(freq_hz, &self.safe_bands, &self.harmful_bands);
        let u0 = amplitude * w * (2.0 * PI * freq_hz * t).sin();
        vec![u0]
    }

    /// One closed-loop step with combined feedback + resonant reference.
    pub fn step_closed_loop(
        &self,
        x: &[f64],
        t: f64,
        dt: f64,
        ref_amp: f64,
        ref_freq_hz: f64,
    ) -> (Vec<f64>, ResonanceSafetyStatus) {
        let u_fb = self.feedback_control(x);
        let u_ref = self.resonant_reference(ref_amp, ref_freq_hz, t);

        // Combine: u_total = u_fb + u_ref (assuming same dimension).
        let mut u_total = vec![0.0; u_fb.len()];
        for i in 0..u_fb.len() {
            let ref_i = *u_ref.get(i).unwrap_or(&0.0);
            u_total[i] = u_fb[i] + ref_i;
        }

        let x_next = self.model.step(x, &u_total, dt);
        let safety = evaluate_resonance_safety(&x_next, &self.safety_cfg);
        (x_next, safety)
    }
}

/// Example construction of a small cyboquatic resonator for a single tank
/// or canal control volume, with 2 states:
/// x = [h; c] where h is water-level deviation, c is concentration deviation.
pub fn example_cyboquatic_controller() -> CyboquaticController {
    // Simple damped oscillator + concentration coupling:
    // ḣ = 0 * h + 1 * c + 0 * u
    // ċ = -ω^2 * h - 2ζω c + b_u * u
    let omega = 0.05; // rad/s (low-frequency sloshing)
    let zeta = 0.2;   // damping ratio

    let a = Matrix::new(
        2,
        2,
        vec![
            0.0, 1.0,
            -omega * omega, -2.0 * zeta * omega,
        ],
    );
    let b = Matrix::new(2, 1, vec![0.0, 1.0]);

    let model = CyboquaticLti::new(a.clone(), b.clone());

    // Safety: limit state norm and concentration amplitude.
    let safety_cfg = ResonanceSafetyConfig {
        max_state_norm: 5.0,
        pollutant_indices: vec![1],
        max_pollutant: 2.0,
    };

    // Define safe and harmful frequency bands in Hz.
    let safe_bands = vec![
        ResonanceBand {
            f_min_hz: 0.0005,
            f_max_hz: 0.002,
        },
    ];
    let harmful_bands = vec![
        ResonanceBand {
            f_min_hz: 0.005,
            f_max_hz: 0.02,
        },
    ];

    // Feedback design: push eigenvalues to desired negative real part.
    let desired_real_part = -0.01;
    let k_feedback = design_static_feedback_2x2(&a, &b, desired_real_part)
        .unwrap_or_else(|| Matrix::new(1, 2, vec![0.0, 0.0]));

    CyboquaticController::new(model, safety_cfg, safe_bands, harmful_bands, k_feedback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eigenvalues_oscillatory() {
        // Undamped oscillator
        let omega = 1.0;
        let a = Matrix::new(2, 2, vec![0.0, 1.0, -omega * omega, 0.0]);
        let (l1, l2) = eigenvalues_2x2(&a).unwrap();
        assert!((l1.real.abs() < 1e-6) && (l1.imag.abs() - omega).abs() < 1e-6);
        assert!((l2.real.abs() < 1e-6) && (l2.imag.abs() - omega).abs() < 1e-6);
    }

    #[test]
    fn test_safety_evaluation() {
        let cfg = ResonanceSafetyConfig {
            max_state_norm: 2.0,
            pollutant_indices: vec![1],
            max_pollutant: 1.0,
        };
        let x_safe = vec![0.5, 0.5];
        let status = evaluate_resonance_safety(&x_safe, &cfg);
        assert!(status.is_safe);

        let x_unsafe = vec![3.0, 0.5];
        let status2 = evaluate_resonance_safety(&x_unsafe, &cfg);
        assert!(!status2.is_safe);
    }

    #[test]
    fn test_closed_loop_step() {
        let ctrl = example_cyboquatic_controller();
        let x0 = vec![0.1, 0.1];
        let (_x1, status) = ctrl.step_closed_loop(&x0, 0.0, 1.0, 0.1, 0.001);
        // Should remain within broad safety envelope for this small step.
        assert!(status.is_safe);
    }
}
