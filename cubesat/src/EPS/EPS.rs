// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

// Page 5, Section 1.1
#[derive(Debug)]
struct EPS {
    solar_panels: Vec<SolarPanel>,
    battery: Battery,
    pdu: PowerDistributionUnit,
    current_mode: SatelliteOperationalMode,
}

// TODO: impl EPS