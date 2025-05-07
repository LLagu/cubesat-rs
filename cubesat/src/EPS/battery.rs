// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars
mod enums;

#[derive(Debug, Clone)]
struct Battery {
    id: String,
    capacity_wh: f64,         // Page 22, Section 3.2
    current_charge_wh: f64,   // Page 22, Section 3.2
    voltage_v: f64,           // Page 22, Section 3.2
    max_discharge_rate_w: f64,// Page 22, Section 3.2
    max_charge_rate_w: f64,   // Inferred from max_discharge_rate_w
    state: enums::BatteryState,
    health_percentage: f64,   // Interpretation of page 26-27, Section 3.5
    charge_efficiency: f64,   // How efficiently it stores power (e.g. 95%)
    discharge_efficiency: f64,// How efficiently it delivers power (e.g. 95%)
    cycles: u32,              // Battery lifespan, standard battery metric
}

// TODO: impl Battery