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

    // TODO: finish impl
}