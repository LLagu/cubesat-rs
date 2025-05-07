// Referencing Introduction to CubeSat Power Control System.pdf from KiboCUBE Academy Webinars

impl SolarPanel {
    fn new(id: String, area_m2: f64, efficiency: f64) -> Self {
        SolarPanel {
            id,
            area_m2, // Page 14, Section 2.1
            efficiency, // Page 14, Section 2.1
            current_power_output_w: 0.0,
            is_deployed: false,
            degradation_factor: 1.0, // Page 15, Section 2.2
        }
    }

    fn deploy(&mut self) {
        self.is_deployed = true;
        println!("Solar panel {} deployed.", self.id);
    }

    // Page 12, Section 1.3
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