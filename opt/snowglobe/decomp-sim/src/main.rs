use std::f64::consts::E;

// Define eco-safety corridors as invariants
struct DecompCorridor {
    toxin_threshold: f64, // mg/L, must be < 0.01
    ph_band: (f64, f64),  // (min, max)
    temp_band: (f64, f64),// Celsius
}

// Pure function for decomposition rate with safety check
fn decomp_rate(substrate: f64, enzyme: f64, v_max: f64, k_m: f64, corridor: &DecompCorridor, ph: f64, temp: f64) -> Result<f64, String> {
    if ph < corridor.ph_band.0 || ph > corridor.ph_band.1 {
        return Err("PH corridor violation".to_string());
    }
    if temp < corridor.temp_band.0 || temp > corridor.temp_band.1 {
        return Err("Temperature corridor violation".to_string());
    }
    let rate = (v_max * substrate) / (k_m + substrate);
    let toxin_est = rate / (1.0 + E.powf(ph - 7.0)); // Simplified toxin model
    if toxin_est >= corridor.toxin_threshold {
        return Err("Toxin threshold exceeded".to_string());
    }
    Ok(rate)
}

fn main() {
    let corridor = DecompCorridor {
        toxin_threshold: 0.01,
        ph_band: (6.5, 7.5),
        temp_band: (20.0, 30.0),
    };
    let result = decomp_rate(10.0, 1.0, 5.0, 2.0, &corridor, 7.0, 25.0);
    match result {
        Ok(rate) => println!("Safe decomposition rate: {}", rate),
        Err(msg) => println!("Gate blocked: {}", msg),
    }
}
