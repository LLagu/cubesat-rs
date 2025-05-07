// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

// TODO: allow multiple concurrent states (?)
#[derive(Debug, Clone, PartialEq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Idle,
    Full,
    Empty,
    Fault(String), // Page 25, Section 3.4
}

// Page 12, Section 1.3
#[derive(Debug, Clone, PartialEq)]
pub enum SatelliteOperationalMode {
    NominalSunlit,
    NominalEclipse,
    SafeMode,
    PayloadOperation,
}