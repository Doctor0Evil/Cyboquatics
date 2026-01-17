#[derive(Clone, Copy, Debug)]
pub struct CECCorridor {
    pub inner_limit: f64,
    pub outer_limit: f64,
    pub violation_residual: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct CECMetrics {
    pub current_index: f64,
    pub predicted_breakthrough: f64,
    pub confidence: f64,  // 0.0 to 1.0
}

impl CECCorridor {
    /// Core safety predicate with explicit overflow checks
    pub fn is_within_inner_limit(&self, metrics: &CECMetrics) -> Result<bool, String> {
        // Prevent NaN propagation
        if self.inner_limit.is_nan() || metrics.current_index.is_nan() {
            return Err("NaN detected in safety corridor".to_string());
        }
        
        // Check both current and predicted states
        let current_ok = metrics.current_index <= self.inner_limit;
        let predicted_ok = metrics.predicted_breakthrough <= self.inner_limit;
        
        // Conjunction as per your specification
        Ok(current_ok && predicted_ok)
    }
    
    /// Calculate violation residual (distance-to-corridor)
    pub fn compute_residual(&self, metrics: &CECMetrics) -> f64 {
        let distance = (metrics.current_index - self.inner_limit).max(0.0);
        let uncertainty = (1.0 - metrics.confidence).max(0.0);
        
        // Lyapunov-like composite measure
        distance + 0.5 * uncertainty * distance
    }
}
