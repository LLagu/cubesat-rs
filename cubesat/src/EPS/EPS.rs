// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

// Page 5, Section 1.1: Typically, the EPS Consists of power generation, power storage, power control & distribution
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

    /// Updates the power output of all solar panels based on the current satellite operational mode.
    /// References:
    /// - Solar Power as Primary Generation: page 14, Section 2.1
    /// - Dependence on Sun Intensity: page 14, Section 2.1
    /// - Dependence on Satellite Attitude (Angle to Sun) page 12, Section 1.3
    /// - Eclipse Condition: page 21, Section 3.1
    fn update_solar_power_generation(&mut self) {
        let (sun_intensity, angle_mod) = match self.current_mode {
            SatelliteOperationalMode::NominalSunlit | SatelliteOperationalMode::PayloadOperation => (SOLAR_FLUX_LEO_AVG_W_M2, 0.8),
            SatelliteOperationalMode::NominalEclipse => (0.0, 0.0),
            SatelliteOperationalMode::SafeMode => (SOLAR_FLUX_LEO_AVG_W_M2, 0.5),
        };

        for panel in self.solar_panels.iter_mut() {
            panel.update_power_output(sun_intensity, angle_mod);
        }
    }

    fn get_total_generated_power_w(&self) -> f64 {
        self.solar_panels.iter().map(|p| p.get_power_output_w()).sum()
    }


    /// Manages the power balance of the CubeSat (power generation, demand from loads, and battery usage) for a given time step.
    /// References:
    /// - Battery Charging with Surplus: page 7  and page 21
    /// - Battery Discharging During Deficit: page 21, Section 3.1
    /// - Load Management / Load Shedding: page 12, Section 1.3
    /// - Transition to Safe Mode: page 12, Section 1.3
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

        let net_power_w = generated_power_w - demanded_power_w;

        if net_power_w >= 0.0 {
            if net_power_w > 0.0 {
                println!("Surplus power {:.2} W available for charging.", net_power_w);
                self.battery.charge(net_power_w, time_step_h);
            } else {
                 println!("Power balanced by solar generation.");
                 if self.battery.state == BatteryState::Charging || self.battery.state == BatteryState::Discharging {
                    if self.battery.current_charge_wh >= self.battery.get_effective_capacity_wh() * 0.999 {
                        self.battery.state = BatteryState::Full;
                    } else if self.battery.current_charge_wh <= self.battery.get_effective_capacity_wh() * 0.001 {
                        self.battery.state = BatteryState::Empty;
                    } else {
                        self.battery.state = BatteryState::Idle;
                    }
                 }
            }
        } else {
            let deficit_w = -net_power_w;
            println!("Power deficit of {:.2} W. Attempting to use battery.", deficit_w);
            let power_from_battery_w = self.battery.discharge(deficit_w, time_step_h);

            if power_from_battery_w < deficit_w * 0.99 {
                println!(
                    "Battery supplied {:.2} W, but {:.2} W was needed. Load shedding may be required.",
                    power_from_battery_w, deficit_w
                );
                if self.battery.get_status() == &BatteryState::Empty || self.battery.get_soc_percentage() < 10.0 {
                     println!("Battery empty or critically low. Attempting to shed non-critical loads.");
                     self.pdu.shed_non_critical_loads();
                     demanded_power_w = self.pdu.get_total_demand_w();
                     let new_net_power_w = generated_power_w - demanded_power_w;
                     if new_net_power_w < 0.0 {
                        let new_deficit_w = -new_net_power_w;
                        let _ = self.battery.discharge(new_deficit_w, time_step_h); // Try again
                        if self.battery.get_status() == &BatteryState::Empty { // Still can't meet critical demand
                            println!("Critical power situation even after shedding. Demanded: {:.2} W. Entering Safe Mode.", demanded_power_w);
                            self.set_satellite_mode(SatelliteOperationalMode::SafeMode); // Call the mode setting function
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
    }


    /// Sets the satellite's operational mode and adjusts component power states accordingly.
    /// References
    /// - Concept of Operational Power Modes: page 12, Section 1.3
    /// - Controlling Loads Based on Mode: page 12, Section 1.3
    /// - Mode Configuration: page 12, Section 1.3
    fn set_satellite_mode(&mut self, mode: SatelliteOperationalMode) {
        if self.current_mode == mode { // No change if already in the target mode
            return;
        }
        println!("Changing satellite mode from {:?} to {:?}", self.current_mode, mode);
        self.current_mode = mode;

        // Default all non-critical to OFF unless explicitly turned ON by the mode
        for load in self.pdu.loads.iter_mut() {
            if !load.is_critical {
                load.turn_off();
            } else {
                load.turn_on(); // Ensure critical loads are ON by default
            }
        }

        match self.current_mode {
            SatelliteOperationalMode::SafeMode => {
                // Critical loads are already set to ON by default logic above.
                // Ensure any specific safe mode loads (if any beyond critical) are handled.
                println!("SafeMode: Ensuring only critical loads are active.");
            }
            SatelliteOperationalMode::PayloadOperation => {
                // In addition to critical loads, turn on specific payloads
                let _ = self.pdu.switch_load("PayloadCam", true);
                let _ = self.pdu.switch_load("PayloadTx", true);
                println!("PayloadOperation: Activating payload camera and transmitter.");
            }
            SatelliteOperationalMode::NominalSunlit | SatelliteOperationalMode::NominalEclipse => {
                // Ensure COM_RX is on, COM_TX might be off unless commanded. Payloads off.
                let _ = self.pdu.switch_load("COM_RX", true); // Ensure COM_RX is on
                let _ = self.pdu.switch_load("COM_TX", false); // Example: TX off by default in nominal
                let _ = self.pdu.switch_load("Heaters", true); // Heaters might be on in nominal modes depending on thermal
                println!("Nominal Mode ({:?}): Baseline operations.", self.current_mode);
            }
        }
    }
}