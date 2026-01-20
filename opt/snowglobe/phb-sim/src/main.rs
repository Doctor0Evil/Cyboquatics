use std::f64::consts::E;

// Define eco-safety corridors as invariants
#[derive(Clone, Copy, Debug)]
struct PhbCorridor {
    toxin_threshold: f64, // mg/L, must be < 0.01
    ph_band: (f64, f64),  // (min, max)
    temp_band: (f64, f64),// Celsius
    redox_band: (f64, f64), // mV for microbial-health
}

// Pure function for PHB accumulation rate with safety checks
fn phb_rate(substrate: f64, nitrogen: f64, mu_max: f64, k_s: f64, q_max: f64, k_i: f64, corridor: &PhbCorridor, ph: f64, temp: f64, redox: f64) -> Result<f64, String> {
    if ph < corridor.ph_band.0 || ph > corridor.ph_band.1 {
        return Err("PH corridor violation".to_string());
    }
    if temp < corridor.temp_band.0 || temp > corridor.temp_band.1 {
        return Err("Temperature corridor violation".to_string());
    }
    if redox < corridor.redox_band.0 || redox > corridor.redox_band.1 {
        return Err("Redox corridor violation".to_string());
    }
    let growth = (mu_max * substrate) / (k_s + substrate);
    let inhibition = nitrogen / (k_i + nitrogen); // Nitrogen-limitation drives accumulation
    let rate = q_max * growth * (1.0 - inhibition);
    let toxin_est = rate / (1.0 + E.powf(ph - 7.0)); // Simplified model, adjust per evidence
    if toxin_est >= corridor.toxin_threshold {
        return Err("Toxin threshold exceeded".to_string());
    }
    Ok(rate)
}

fn main() {
    let corridor = PhbCorridor {
        toxin_threshold: 0.01,
        ph_band: (6.5, 7.5),
        temp_band: (20.0, 30.0),
        redox_band: (100.0, 300.0),
    };
    let result = phb_rate(50.0, 0.5, 0.2, 10.0, 0.1, 1.0, &corridor, 7.0, 25.0, 200.0);
    match result {
        Ok(rate) => println!("Safe PHB accumulation rate: {} g/L/h", rate),
        Err(msg) => println!("Gate blocked: {}", msg),
    }
}
