// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

// A.k.a. subsystems/payloads
#[derive(Debug, Clone)]
struct Load {
    id: String,
    power_consumption_w: f64,
    is_critical: bool,        // Page 33, Section 4.2 (e.g. telecomm, attitude control etc.)
    is_on: bool,              // Page 12, Section 1.3, constant power-ON components and ON/OFF controllable.
}

impl Load {
    fn new(id: String, power_consumption_w: f64, is_critical: bool) -> Self {
        Load {
            id,
            power_consumption_w,
            is_critical,
            is_on: false, 
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

// Actual PDU
#[derive(Debug)]
struct PowerDistributionUnit {
    loads: Vec<Load>,
}

impl PowerDistributionUnit {
    fn new() -> Self {
        PowerDistributionUnit { loads: Vec::new() }
    }

    fn add_load(&mut self, load: Load) {
        self.loads.push(load);
    }

    // Page 12, Section 1.3 and page 34, Section 4.3: ability to toggle subsystems (loads) on/off
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

    // Page 12, Section 1.3: turn off non-critical loads if needed
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