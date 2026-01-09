from core import FogNodeState, FogImpactConfig, computeFogImpact
from io import read_sensors

fn main_loop():
    var loop_count: Int = 0
    let cfg = FogImpactConfig(100.0, 1.2, 6.7e5)
    while loop_count < 10:  # Fixed iterations for deterministic simulation
        let reading = read_sensors()
        let s = FogNodeState(300.0, 60.0, reading.flow, 2880.0)  # 48min step
        let result = computeFogImpact(s, cfg)
        print("Loop", loop_count, "| Karma:", result.node_impact_K)
        loop_count += 1

fn main():
    main_loop()
