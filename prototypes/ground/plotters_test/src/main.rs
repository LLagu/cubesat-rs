use plotters::prelude::*;
use std::error::Error;

fn calculate_satellite_position(time_s: f64) -> (f64, f64) {
    let radius = 1.5; 
    let period = 10.0;
    let angular_velocity = 2.0 * std::f64::consts::PI / period;
    let angle = angular_velocity * time_s;
    (radius * angle.cos(), radius * angle.sin())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root_folder = "orbit_frames";
    std::fs::create_dir_all(root_folder)?;

    let time_start = 0.0;
    let time_end = 20.0;
    let time_step = 0.1; 
    let frame_count = ((time_end - time_start) / time_step) as usize;

    let mut current_time = time_start;

    println!("Generating {} frames...", frame_count);

    for i in 0..frame_count {
        let file_path = format!("{}/frame_{:04}.png", root_folder, i);
        let root = BitMapBackend::new(&file_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let (sat_x, sat_y) = calculate_satellite_position(current_time);

        // Define chart area
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Satellite Orbit - Time: {:.2}s", current_time), ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(-2.0..2.0, -2.0..2.0)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(std::iter::once(Circle::new((0.0, 0.0), 10, BLUE.filled())))?; //Earth

        chart.draw_series(std::iter::once(Circle::new((sat_x, sat_y), 5, RED.filled())))?; //Sat

        root.present()?;
        println!("Generated {}", file_path);

        current_time += time_step;
    }

    println!("Finished generating frames.");
    println!("You can now combine them using ffmpeg, e.g.:");
    println!("ffmpeg -framerate 10 -i orbit_frames/frame_%04d.png -c:v libx264 -pix_fmt yuv420p orbit_animation.mp4");

    Ok(())
}