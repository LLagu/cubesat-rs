use std::env;
use color_eyre::eyre::Result;
mod tasks;


fn main() -> Result<()> {
    color_eyre::install()?; 

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <serial|usb>", args[0]);
        return Err(color_eyre::eyre::eyre!("Invalid arguments"));
    }

    match args[1].as_str() {
        "serial" => {
            println!("Running serial terminal...");
            tasks::serial_term()
        }
        "usb" => {
            println!("Running USB list...");
            tasks::send_command_test_loop()
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Available commands: serial, usb");
            Err(color_eyre::eyre::eyre!("Unknown command"))
        }
    }
}