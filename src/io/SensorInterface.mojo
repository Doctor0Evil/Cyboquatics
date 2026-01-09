struct SensorReading:
    var flow: Float64
    var temp: Float64
    var level: Float64

fn read_sensors() -> SensorReading:
    # Stub for real hardware read; in production, interface with GPIO/ADC
    return SensorReading(20.0, 55.0, 0.75)

fn main():
    let reading = read_sensors()
    print("Flow:", reading.flow, "L/min | Temp:", reading.temp, "C | Level:", reading.level)
