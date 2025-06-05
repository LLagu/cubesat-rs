use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::io::{self, Read};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // List available serial ports
    let ports = serialport::available_ports()?;
    if ports.is_empty() {
        println!("No serial ports found");
        return Ok(());
    }
    
    // Find the nRF52840 Dongle
    let mut dongle_port: Option<SerialPortInfo> = None;
    
    println!("Available serial ports:");
    for port in &ports {
        println!("- {}", port.port_name);
        
        // Try to identify the nRF52840 dongle
        // It often appears as a USB CDC device with specific VID/PID or port name
        match &port.port_type {
            SerialPortType::UsbPort(info) => {
                // Check for VID/PID matching our dongle
                // Note: The firmware uses 0x1209/0x0309
                if (info.vid == 0x1209 && info.pid == 0x0309) || 
                   // Often appears as CDC ACM device (Nordic VID)
                   (info.vid == 0x1915) || 
                   // Look for common dongle names
                   port.port_name.contains("ttyACM") ||
                   port.port_name.contains("USB") {
                    dongle_port = Some(port.clone());
                }
            }
            _ => {}
        }
    }
    
    // Use the identified port or ask the user to select one
    let port_name = match dongle_port {
        Some(port) => {
            println!("\nDetected nRF52840 Dongle on port: {}", port.port_name);
            port.port_name
        }
        None => {
            println!("\nCouldn't automatically identify the nRF52840 Dongle.");
            println!("Please enter the port number to use (e.g., 0 for the first port):");
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let port_idx = input.trim().parse::<usize>()?;
            
            if port_idx >= ports.len() {
                return Err("Invalid port selection".into());
            }
            
            ports[port_idx].port_name.clone()
        }
    };
    
    // Open the serial port
    println!("Opening serial port: {}", port_name);
    let mut port = serialport::new(port_name, 115200) // Common baud rate for debugging
        .timeout(Duration::from_millis(10))
        .open()?;
    
    println!("Listening for data from nRF52840 Dongle...");
    println!("Press Ctrl+C to exit");
    
    // Read data from the serial port
    let mut buffer = [0u8; 1024];
    Ok(loop {
        match port.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Convert bytes to string and print
                if let Ok(data) = String::from_utf8(buffer[0..bytes_read].to_vec()) {
                    print!("{}", data);
                } else {
                    // If not valid UTF-8, print the raw bytes
                    print!("Raw bytes: {:?}", &buffer[0..bytes_read]);
                }
            }
            Ok(_) => {
                // No data available, wait a bit
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // Timeout is normal, just continue
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {}", e);
                break;
            }
        }
    })
}