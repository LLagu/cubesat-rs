// Based on Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

// Average solar flux in LEO (Watts per square meter):
// value taken from NASA's On-Orbit_Thermal_Environments_TFAWS_2014.pdf
const SOLAR_FLUX_LEO_AVG_W_M2: f64 = 1367.0; 

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


#[derive(Debug, Clone)]
struct SolarPanel {
    id: String,
    area_m2: f64,    
    efficiency: f64,    // Conversion efficiency (0.0 to 1.0)
    current_power_output_w: f64,
    is_deployed: bool,
    degradation_factor: f64, // Starts at 1.0, decreases over time
}

impl SolarPanel {
    fn new(id: String, area_m2: f64, efficiency: f64) -> Self {
        SolarPanel {
            id,
            area_m2,
            efficiency,
            current_power_output_w: 0.0,
            is_deployed: false, // Assuming panels need deployment
            degradation_factor: 1.0,
        }
    }

    fn deploy(&mut self) {
        self.is_deployed = true;
        println!("Solar panel {} deployed.", self.id);
    }

    fn update_power_output(&mut self, sun_intensity_w_m2: f64, angle_modifier: f64) {
        if self.is_deployed {
            // angle_modifier: 0.0 (no sun) to 1.0 (direct sun)
            // sun_intensity_w_m2: Can vary based on orbit position relative to Earth's shadow
            self.current_power_output_w = self.area_m2 * self.efficiency * sun_intensity_w_m2 * angle_modifier * self.degradation_factor;
        } else {
            self.current_power_output_w = 0.0;
        }
    }

    fn get_power_output_w(&self) -> f64 {
        self.current_power_output_w
    }

    // Simplified degradation over time 
    fn apply_degradation(&mut self, factor_decrease: f64) {
        self.degradation_factor = (self.degradation_factor - factor_decrease).max(0.0);
    }
}

#[derive(Debug, Clone)]
struct Battery {
    id: String,
    capacity_wh: f64,         // Total capacity in Watt-hours
    current_charge_wh: f64,   // Current charge in Watt-hours
    voltage_v: f64,           // Nominal voltage
    max_charge_rate_w: f64,   // Maximum power for charging
    max_discharge_rate_w: f64,// Maximum power for discharging
    state: BatteryState,
    health_percentage: f64,   // 0.0 to 100.0, affects capacity/rates
    charge_efficiency: f64,   // How efficiently it stores power (e.g., 0.95 for 95%)
    discharge_efficiency: f64,// How efficiently it delivers power (e.g., 0.95 for 95%)
    cycles: u32,
}

impl Battery {
    fn new(id: String, capacity_wh: f64, initial_charge_wh: f64, voltage_v: f64, max_charge_rate_w: f64, max_discharge_rate_w: f64) -> Self {
        let initial_charge_wh = initial_charge_wh.min(capacity_wh);
        Battery {
            id,
            capacity_wh,
            current_charge_wh: initial_charge_wh,
            voltage_v,
            max_charge_rate_w,
            max_discharge_rate_w,
            state: BatteryState::Idle,
            health_percentage: 100.0,
            charge_efficiency: 0.9, // Default 90%
            discharge_efficiency: 0.9, // Default 90%
            cycles: 0,
        }
    }

    fn get_effective_capacity_wh(&self) -> f64 {
        self.capacity_wh * (self.health_percentage / 100.0)
    }

    // Power in Watts, duration in hours
    fn charge(&mut self, mut power_w: f64, duration_h: f64) {
        if self.state == BatteryState::Fault("Over-temperature".to_string()) { // Example fault
            println!("Battery {} is faulty, cannot charge.", self.id);
            return;
        }

        power_w = power_w.min(self.max_charge_rate_w * (self.health_percentage / 100.0));
        let energy_to_add_wh = power_w * duration_h * self.charge_efficiency;
        let effective_capacity = self.get_effective_capacity_wh();

        if self.current_charge_wh >= effective_capacity {
            self.state = BatteryState::Full;
            self.current_charge_wh = effective_capacity; // Cap at full
            return;
        }

        self.state = BatteryState::Charging;
        self.current_charge_wh += energy_to_add_wh;

        if self.current_charge_wh >= effective_capacity {
            self.current_charge_wh = effective_capacity;
            self.state = BatteryState::Full;
            // Potentially count a charge cycle here based on depth
        }
    }

    // Returns actual power supplied in Watts, duration in hours
    fn discharge(&mut self, mut power_demand_w: f64, duration_h: f64) -> f64 {
        if self.state == BatteryState::Fault("Under-voltage".to_string()) { // Example fault
            println!("Battery {} is faulty, cannot discharge.", self.id);
            return 0.0;
        }

        power_demand_w = power_demand_w.min(self.max_discharge_rate_w * (self.health_percentage / 100.0));
        let energy_needed_wh = power_demand_w * duration_h / self.discharge_efficiency; // Account for discharge efficiency

        if self.current_charge_wh <= 0.0 {
            self.state = BatteryState::Empty;
            self.current_charge_wh = 0.0; // Cap at empty
            return 0.0; // Cannot supply power
        }

        self.state = BatteryState::Discharging;
        let energy_can_supply_wh = self.current_charge_wh.min(energy_needed_wh);
        self.current_charge_wh -= energy_can_supply_wh;

        if self.current_charge_wh <= 0.0 {
            self.current_charge_wh = 0.0;
            self.state = BatteryState::Empty;
            self.cycles += 1; // Increment cycle count on full discharge (simplified)
        }
        // Actual power supplied, considering what the battery could give
        energy_can_supply_wh * self.discharge_efficiency / duration_h
    }

    fn get_soc_percentage(&self) -> f64 {
        (self.current_charge_wh / self.get_effective_capacity_wh()) * 100.0
    }

    fn get_status(&self) -> &BatteryState {
        &self.state
    }

    fn apply_health_degradation(&mut self, percentage_decrease: f64) {
        self.health_percentage = (self.health_percentage - percentage_decrease).max(0.0);
        if self.health_percentage < 20.0 && self.state != BatteryState::Fault("Degraded".to_string()) {
             self.state = BatteryState::Fault("Severely Degraded".to_string());
        }
    }
}

#[derive(Debug, Clone)]
struct Load {
    id: String,
    power_consumption_w: f64,
    is_critical: bool,
    is_on: bool,
}

impl Load {
    fn new(id: String, power_consumption_w: f64, is_critical: bool) -> Self {
        Load {
            id,
            power_consumption_w,
            is_critical,
            is_on: false, // Default to off
        }
    }

    fn turn_on(&mut self) {
        self.is_on = true;
    }

    fn turn_off(&mut self) {
        self.is_on = false;
    }

    fn get_power_demand_w(&self) -> f64 {
        if self.is_on {
            self.power_consumption_w
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
struct PowerDistributionUnit {
    loads: Vec<Load>,
    // Can include inhibit switches, current limiters, etc.
}

impl PowerDistributionUnit {
    fn new() -> Self {
        PowerDistributionUnit { loads: Vec::new() }
    }

    fn add_load(&mut self, load: Load) {
        self.loads.push(load);
    }

    fn switch_load(&mut self, load_id: &str, new_state: bool) -> Result<(), String> {
        if let Some(load) = self.loads.iter_mut().find(|l| l.id == load_id) {
            if new_state {
                load.turn_on();
            } else {
                load.turn_off();
            }
            Ok(())
        } else {
            Err(format!("Load with ID {} not found.", load_id))
        }
    }

    fn get_total_demand_w(&self) -> f64 {
        self.loads.iter().map(|load| load.get_power_demand_w()).sum()
    }

    // Basic load shedding: turn off non-critical loads if needed
    fn shed_non_critical_loads(&mut self) -> f64 {
        let mut shed_power = 0.0;
        for load in self.loads.iter_mut() {
            if load.is_on && !load.is_critical {
                load.turn_off();
                shed_power += load.power_consumption_w;
                println!("Shedding non-critical load: {}", load.id);
            }
        }
        shed_power
    }
}

#[derive(Debug)]
struct EPS {
    solar_panels: Vec<SolarPanel>,
    battery: Battery,
    pdu: PowerDistributionUnit,
    current_mode: SatelliteOperationalMode,
}

impl EPS {
    fn new(solar_panels: Vec<SolarPanel>, battery: Battery, pdu: PowerDistributionUnit) -> Self {
        EPS {
            solar_panels,
            battery,
            pdu,
            current_mode: SatelliteOperationalMode::NominalSunlit, // Initial mode
        }
    }

    fn update_solar_power_generation(&mut self) {
        let (sun_intensity, angle_mod) = match self.current_mode {
            SatelliteOperationalMode::NominalSunlit | SatelliteOperationalMode::PayloadOperation => (SOLAR_FLUX_LEO_AVG_W_M2, 0.8), // Assume 80% effective angle
            SatelliteOperationalMode::NominalEclipse => (0.0, 0.0), // No sun in eclipse
            SatelliteOperationalMode::SafeMode => (SOLAR_FLUX_LEO_AVG_W_M2, 0.5), // Maybe pointing for survival
        };

        for panel in self.solar_panels.iter_mut() {
            panel.update_power_output(sun_intensity, angle_mod);
        }
    }

    fn get_total_generated_power_w(&self) -> f64 {
        self.solar_panels.iter().map(|p| p.get_power_output_w()).sum()
    }

    // time_step_h: duration of this simulation step in hours
    fn manage_power(&mut self, time_step_h: f64) {
        self.update_solar_power_generation();
        let generated_power_w = self.get_total_generated_power_w();
        let mut demanded_power_w = self.pdu.get_total_demand_w();

        println!(
            "--- Power Management Step (Mode: {:?}, Time Step: {:.2}h) ---",
            self.current_mode, time_step_h
        );
        println!(
            "Solar Generation: {:.2} W, Initial Demand: {:.2} W, Battery SoC: {:.1}% ({:?})",
            generated_power_w,
            demanded_power_w,
            self.battery.get_soc_percentage(),
            self.battery.get_status()
        );

        let mut net_power_w = generated_power_w - demanded_power_w;

        if net_power_w >= 0.0 { // Surplus power or exact match
            // Use surplus to charge the battery
            if net_power_w > 0.0 {
                println!("Surplus power {:.2} W available for charging.", net_power_w);
                self.battery.charge(net_power_w, time_step_h);
            } else {
                 println!("Power balanced by solar generation.");
                 if self.battery.state == BatteryState::Charging || self.battery.state == BatteryState::Discharging {
                    self.battery.state = BatteryState::Idle; // Or Full/Empty if applicable
                 }
            }
        } else { // Power deficit, need to use battery
            let deficit_w = -net_power_w;
            println!("Power deficit of {:.2} W. Attempting to use battery.", deficit_w);
            let power_from_battery_w = self.battery.discharge(deficit_w, time_step_h);

            if power_from_battery_w < deficit_w * 0.99 { // Check if battery could meet demand (with a small tolerance)
                println!(
                    "Battery supplied {:.2} W, but {:.2} W was needed. Load shedding may be required.",
                    power_from_battery_w, deficit_w
                );
                // Enter safe mode or shed loads if battery cannot cover deficit
                if self.battery.get_status() == &BatteryState::Empty || self.battery.get_soc_percentage() < 10.0 { // Low SoC threshold
                     println!("Battery empty or critically low. Attempting to shed non-critical loads.");
                     self.pdu.shed_non_critical_loads();
                     demanded_power_w = self.pdu.get_total_demand_w(); // Recalculate demand
                     net_power_w = generated_power_w - demanded_power_w;
                     if net_power_w < 0.0 {
                        let new_deficit_w = -net_power_w;
                        let _ = self.battery.discharge(new_deficit_w, time_step_h); // Try again with reduced load
                        println!("Critical power situation. Demanded: {:.2} W. Entering Safe Mode.", demanded_power_w);
                        self.current_mode = SatelliteOperationalMode::SafeMode;
                        // Turn off all non-critical loads in safe mode.
                        for load in self.pdu.loads.iter_mut() {
                            if !load.is_critical { load.turn_off(); }
                        }
                     }
                }
            } else {
                println!("Battery supplied {:.2} W to cover deficit.", power_from_battery_w);
            }
        }
         println!(
            "End of Step: Battery SoC: {:.1}% ({:?}), Total Demand: {:.2}W",
            self.battery.get_soc_percentage(),
            self.battery.get_status(),
            self.pdu.get_total_demand_w()
        );
        println!("------------------------------------------------------------------");
    }

    fn set_satellite_mode(&mut self, mode: SatelliteOperationalMode) {
        println!("Changing satellite mode from {:?} to {:?}", self.current_mode, mode);
        self.current_mode = mode;
        // Adjust loads based on mode
        match self.current_mode {
            SatelliteOperationalMode::SafeMode => {
                for load in self.pdu.loads.iter_mut() {
                    if !load.is_critical {
                        load.turn_off();
                    } else {
                        load.turn_on(); // Ensure critical loads are on
                    }
                }
            }
            SatelliteOperationalMode::PayloadOperation => {
                // Example: Turn on payload
                let _ = self.pdu.switch_load("PayloadCam", true);
                let _ = self.pdu.switch_load("PayloadTx", true);
            }
            SatelliteOperationalMode::NominalSunlit | SatelliteOperationalMode::NominalEclipse => {
                 // Example: Turn off payload if not in payload op mode
                let _ = self.pdu.switch_load("PayloadCam", false);
                let _ = self.pdu.switch_load("PayloadTx", false);
                let _ = self.pdu.switch_load("OBC", true); // OBC always on
                let _ = self.pdu.switch_load("COM_RX", true); // Receiver always on
            }
        }
    }
}

fn main() {
 
}