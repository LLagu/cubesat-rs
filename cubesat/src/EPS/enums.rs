#[derive(Debug, Clone, PartialEq)]
enum BatteryState {
    Charging,
    Discharging,
    Idle,
    Full,
    Empty,
    Fault(String),
}

#[derive(Debug, Clone, PartialEq)]
enum SatelliteOperationalMode {
    NominalSunlit,
    NominalEclipse,
    SafeMode,
    PayloadOperation,
}