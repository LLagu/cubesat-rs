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
            health_percentage: 1.0,     // Starts at 100%
            charge_efficiency: 0.9,     // Default 90%
            discharge_efficiency: 0.9,  // Default 90%
            cycles: 0,
        }
    }

    fn get_effective_capacity_wh(&self) -> f64 {
        self.capacity_wh * (self.health_percentage / 100.0)
    }

    fn charge(&mut self, mut power_w: f64, duration_h: f64) {
        // TODO: handle different types of faults
        // if self.state == BatteryState::Fault("Fault-type".to_string()) {
        //     println!("Battery {} is faulty, cannot charge.", self.id);
        //     return;
        // }

        power_w = power_w.min(self.max_charge_rate_w * self.health_percentage);
        let energy_to_add_wh = power_w * duration_h * self.charge_efficiency;
        let effective_capacity = self.get_effective_capacity_wh();

        if self.current_charge_wh >= effective_capacity {
            self.state = BatteryState::Full;
            self.current_charge_wh = effective_capacity;
            return;
        }

        self.state = BatteryState::Charging;
        self.current_charge_wh += energy_to_add_wh;

        if self.current_charge_wh >= effective_capacity {
            self.current_charge_wh = effective_capacity;
            self.state = BatteryState::Full;
            // TODO: increment cycles here as well?
        }
    }

    // Returns actual power supplied 
    fn discharge(&mut self, mut power_demand_w: f64, duration_h: f64) -> f64 {
        // TODO: handle different types of faults here as well
        // if self.state == BatteryState::Fault("Fault-type".to_string()) { 
        //     println!("Battery {} is faulty, cannot discharge.", self.id);
        //     return 0.0;
        // }

        power_demand_w = power_demand_w.min(self.max_discharge_rate_w * self.health_percentage);
        let energy_needed_wh = power_demand_w * duration_h / self.discharge_efficiency;

        if self.current_charge_wh <= 0.0 {
            self.state = BatteryState::Empty;
            self.current_charge_wh = 0.0;
            return 0.0; 
        }

        self.state = BatteryState::Discharging;
        let energy_can_supply_wh = self.current_charge_wh.min(energy_needed_wh);
        self.current_charge_wh -= energy_can_supply_wh;

        if self.current_charge_wh <= 0.0 {
            self.current_charge_wh = 0.0;
            self.state = BatteryState::Empty;
            self.cycles += 1;
        }
    
        energy_can_supply_wh * self.discharge_efficiency / duration_h
    }

    fn get_soc_percentage(&self) -> f64 {
        (self.current_charge_wh / self.get_effective_capacity_wh()) * 100.0
    }

    fn get_status(&self) -> &BatteryState {
        &self.state
    }

    // percentage_decrease range between 0.0 and 1.0
    fn apply_health_degradation(&mut self, percentage_decrease: f64) {
        self.health_percentage = (self.health_percentage - percentage_decrease).max(0.0);
        if self.health_percentage < 0.2 && self.state != BatteryState::Fault("Degraded".to_string()) {
             self.state = BatteryState::Fault("Severely Degraded".to_string());
        }
    }
}