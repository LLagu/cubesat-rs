use std::{
    io::{self, Write as _},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use color_eyre::eyre::{anyhow, bail};
use hidapi::HidApi;
use serialport::SerialPortType;
use std::thread; // If needed for delays

pub fn change_channel(channel: &str) -> color_eyre::Result<()> {
    fn check_pid(pid: u16) -> bool {
        pid == consts::USB_PID_DONGLE_LOOPBACK || pid == consts::USB_PID_DONGLE_PUZZLE
    }

    let api = HidApi::new()?;
    let dev = api
        .device_list()
        .filter(|dev| dev.vendor_id() == consts::USB_VID_DEMO && check_pid(dev.product_id()))
        .next()
        .ok_or_else(|| anyhow!("device not found"))?
        .open_device(&api)?;

    let chan = channel.parse::<u8>()?;
    if chan < 11 || chan > 26 {
        bail!("channel is out of range (`11..=26`)")
    }
    const REPORT_ID: u8 = 0;
    dev.write(&[REPORT_ID, chan])?;
    println!("requested channel change to channel {}", chan);

    Ok(())
}

pub fn serial_term() -> color_eyre::Result<()> {
    let mut once = true;
    let dongle = loop {
        if let Some(dongle) = serialport::available_ports()?
            .into_iter()
            .filter(|info| match &info.port_type {
                SerialPortType::UsbPort(usb) => usb.vid == consts::USB_VID_DEMO,
                _ => false,
            })
            .next()
        {
            break dongle;
        } else if once {
            once = false;

            eprintln!("(waiting for the Dongle to be connected)");
        }
    };

    let mut port = serialport::new(&dongle.port_name, 115200).open()?;
    port.set_timeout(Duration::from_millis(10))?;

    static CONTINUE: AtomicBool = AtomicBool::new(true);

    // properly close the serial device on Ctrl-C
    ctrlc::set_handler(|| CONTINUE.store(false, Ordering::Relaxed))?;

    let stdout = io::stdout();
    while CONTINUE.load(Ordering::Relaxed) {
        let mut read_buf = [0u8; 8];
        match port.read(&mut read_buf) {
            Ok(n) => {
                let mut stdout = stdout.lock();
                stdout.write_all(&read_buf[..n])?;
                stdout.flush()?;        
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Go around
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }

    eprintln!("(closing the serial port)");
    Ok(())
}

pub fn send_hid_command(command_byte: u8, payload: &[u8]) -> color_eyre::Result<()> {
    // if payload.len() >= consts::HID_REPORT_SIZE {
    //     // Payload must fit alongside the command byte
    //     bail!("Payload too large ({} bytes), max is {}", payload.len(), consts::HID_REPORT_SIZE - 1);
    // }

    // println!(
    //     "Searching for HID device VID={:#06x} PID={:#06x}...",
    //     consts::USB_VID_DEMO,
    //     consts::USB_PID_DONGLE_LOOPBACK
    // );

    let api = HidApi::new().expect("Failed to create HidApi instance.");

    // Find the device
    let device_info = api
        .device_list()
        .find(|d| d.vendor_id() == consts::USB_VID_DEMO && d.product_id() == consts::USB_PID_DONGLE_LOOPBACK)
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "HID device VID={:#06x} PID={:#06x} not found",
                consts::USB_VID_DEMO,
                consts::USB_PID_DONGLE_LOOPBACK
            )
        })?;

    // println!("Found HID device: {:?}", device_info.product_string());

    let device = device_info.open_device(&api).expect("Failed to open HID device");

    // Prepare the HID report buffer (must be exactly HID_REPORT_SIZE bytes for raw output)
    // Some OSes might require prepending a 'Report ID' byte (often 0x00) if your HID descriptor uses them.
    // If your descriptor *doesn't* use report IDs (common for simple vendor-defined devices),
    // then the buffer is just the raw data. Let's assume no Report ID for now.
    let mut report = [0u8; 64];
    report[0] = command_byte;
    report[1..][..payload.len()].copy_from_slice(payload);

    // println!("Sending HID report ({} bytes): command={:#04x}, payload_len={}, report[0..8]={:02X?}",
    //     report.len(), command_byte, payload.len(), &report[0..core::cmp::min(8, 1 + payload.len())]);

    // Send the report via write (which usually corresponds to HID Output reports)
    // Note: Some HID implementations might use feature reports instead. Check your descriptor.
    let bytes_written = device.write(&report)
        .expect("Failed to write HID report");

    if bytes_written != report.len() {
         bail!("Incomplete HID write: wrote {} bytes, expected {}", bytes_written, report.len());
    }

    // println!("HID report sent successfully.");

    thread::sleep(Duration::from_millis(5000));

    Ok(())
}

pub fn send_command_test_loop() -> color_eyre::Result<()>{
    loop {
        thread::sleep(Duration::from_millis(50));
        let payload_to_send = b"Hello";
        println!("Sending payload: '{:?}'", std::str::from_utf8(payload_to_send));
        send_hid_command(consts::CMD_SEND_RADIO, payload_to_send);
    }
}

pub fn usb_list() -> color_eyre::Result<()> {
    for dev in rusb::devices()?.iter() {
        let desc = dev.device_descriptor()?;
        let suffix = match (desc.vendor_id(), desc.product_id()) {
            (0x1366, pid) if (pid >> 8) == 0x10 || (pid >> 8) == 0x01 => " <- J-Link on the nRF52840 Development Kit",
            (0x1915, 0x521f) => " <- nRF52840 Dongle (in bootloader mode)",
            (consts::USB_VID_DEMO, consts::USB_PID_DONGLE_LOOPBACK) => " <- nRF52840 Dongle (loopback-fw)",
            (consts::USB_VID_DEMO, consts::USB_PID_DONGLE_PUZZLE) => " <- nRF52840 Dongle (puzzle-fw)",
            (consts::USB_VID_DEMO, consts::USB_PID_RTIC_DEMO) => " <- nRF52840 on the nRF52840 Development Kit",
            _ => "",
        };

        println!("{:?}{}", dev, suffix);
    }

    Ok(())
}

pub fn usb_descriptors() -> color_eyre::Result<()> {
    for dev in rusb::devices()?.iter() {
        let dev_desc = dev.device_descriptor()?;
        if dev_desc.vendor_id() == consts::USB_VID_DEMO && dev_desc.product_id() == consts::USB_PID_RTIC_DEMO {
            println!("{:#?}", dev_desc);
            println!("address: {}", dev.address());
            for i in 0..dev_desc.num_configurations() {
                let conf_desc = dev.config_descriptor(i)?;
                println!("config{}: {:#?}", i, conf_desc);

                for iface in conf_desc.interfaces() {
                    println!(
                        "iface{}: {:#?}",
                        iface.number(),
                        iface.descriptors().collect::<Vec<_>>()
                    );
                }
            }

            // TODO open the device; this will force the OS to configure it
            // let mut handle = dev.open()?;

            return Ok(());
        }
    }

    bail!("nRF52840 USB device not found")
}