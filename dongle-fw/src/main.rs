// #![no_std]
// #![no_main]

// use core::panic::PanicInfo;
// use cortex_m_rt::entry;

// use dongle as dk;
// use dk::ieee802154::{self, Channel, Packet, TxPower};

// use nrf52840_hal::usbd::{UsbPeripheral, Usbd};

// use usb_device::bus::UsbBusAllocator;
// use usbd_serial::SerialPort;

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

// const USB_VID_DEMO: u16 = 0x1209;
// const USB_PID_FOR_SIMPLE_SERIAL: u16 = 0x0309; 


// #[entry]
// fn main() -> ! {

//     let mut board = dk::init().unwrap();
//     let mut leds = board.leds;
//     let mut timer = board.timer;
//     let mut radio = board.radio;
//     // let mut usbd = board.usbd;

//     // Serial comm setup
//     let usb_p = UsbPeripheral::new(board.usbd, board.clocks);
//     let usb_bus_allocator = UsbBusAllocator::new(Usbd::new(usb_p));
//     let usb_bus = usb_bus_allocator;
//     let mut serial = SerialPort::new(&usb_bus);

//     radio.set_channel(Channel::_20);  
//     radio.set_txpower(TxPower::_0dBm);
    
//     leds.ld1.on();
    
//     let mut rx_packet = Packet::new();
//     let mut tx_packet = Packet::new();
    
//     const TRIGGER_MESSAGE: &[u8] = b"START";
//     const RESPONSE_MESSAGE: &[u8] = b"HELLO FROM NRF52840";
    
//     loop {
//         leds.ld2_blue.on();
        
//         // Try to receive a packet with timeout (1 second)
//         match radio.recv_timeout(&mut rx_packet, &mut timer, 1_000_000) {
//             Ok(_) => {
//                 // We received something - check if it's our trigger message
//                 if rx_packet.len() as usize == TRIGGER_MESSAGE.len() && 
//                    &rx_packet[..] == TRIGGER_MESSAGE {
                    
//                     // Blink green LED to indicate received trigger
//                     leds.ld2_blue.off();
//                     leds.ld2_green.on();
                    
//                     tx_packet.copy_from_slice(RESPONSE_MESSAGE);
//                     // Small delay to make sure receiver is ready
//                     timer.wait(core::time::Duration::from_millis(10));
                    
//                     radio.send(&mut tx_packet);
                    
//                     // LED feedback
//                     leds.ld2_green.off();
//                     leds.ld2_red.on();
//                     timer.wait(core::time::Duration::from_millis(100));
//                     leds.ld2_red.off();
//                 }
//             },
//             Err(ieee802154::Error::Timeout) => {
//                 leds.ld2_blue.off();
//             },
//             Err(_) => {
//                 // // Error in reception (e.g., CRC error)
//                 // leds.ld2_red.on();
//                 // timer.wait(core::time::Duration::from_millis(50));
//                 // leds.ld2_red.off();
//             }
//         }
        
//         timer.wait(core::time::Duration::from_millis(10));
//     }
// }

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use usbd_serial::embedded_io::Write;
use cortex_m_rt::entry;

use dongle as dk;
use dk::ieee802154::{self, Channel, Packet, TxPower};

use nrf52840_hal::usbd::{UsbPeripheral, Usbd};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const USB_VID_DEMO: u16 = 0x1209;
const USB_PID_FOR_SIMPLE_SERIAL: u16 = 0x0309; 

#[entry]
fn main() -> ! {
    let mut board = dk::init().unwrap();
    let mut leds = board.leds;
    let mut timer = board.timer;
    let mut radio = board.radio;

    // Initialize USB for serial debugging
    let usb_p = UsbPeripheral::new(board.usbd, board.clocks);
    let usb_bus = UsbBusAllocator::new(Usbd::new(usb_p));
    
    let mut serial = SerialPort::new(&usb_bus);
    
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(USB_PID_FOR_SIMPLE_SERIAL, USB_VID_DEMO))
        .device_class(USB_CLASS_CDC)
        .max_packet_size_0(64)
        .unwrap()
        .build();

    // Radio setup
    radio.set_channel(Channel::_20);  
    radio.set_txpower(TxPower::_0dBm);
    
    leds.ld1.on();
    
    // Log startup message
    let mut usb_initialized = false;
    
    let mut rx_packet = Packet::new();
    let mut tx_packet = Packet::new();
    
    const TRIGGER_MESSAGE: &[u8] = b"START";
    const RESPONSE_MESSAGE: &[u8] = b"HELLO FROM NRF52840";
    
    let mut counter = 0;
    
    loop {
        // USB device polling - necessary for USB to work
        if usb_dev.poll(&mut [&mut serial]) {
            if !usb_initialized {
                usb_initialized = true;
                let _ = serial.write(b"USB Serial initialized!\r\n");
                
                // Create a static buffer for the channel message
                let mut buffer = [0u8; 64];
                let mut i = 0;
                
                // Manually create "Radio channel: 20, TX power: 0dBm" message
                for b in b"Radio channel: " {
                    buffer[i] = *b;
                    i += 1;
                }
                
                // Add "20"
                buffer[i] = b'2'; i += 1;
                buffer[i] = b'0'; i += 1;
                
                // Add the rest
                for b in b", TX power: 0dBm\r\n" {
                    buffer[i] = *b;
                    i += 1;
                }
                
                let _ = serial.write(&buffer[0..i]);
            }
        }
        
        leds.ld2_blue.on();
        
        // Try to receive a packet with timeout (1 second)
        match radio.recv_timeout(&mut rx_packet, &mut timer, 1_000_000) {
            Ok(_) => {
                // Log packet reception
                if usb_dev.poll(&mut [&mut serial]) {
                    let _ = serial.write(b"Received packet, length: ");
                    
                    // Convert packet length to ASCII digits
                    let len = rx_packet.len();
                    let mut digits = [0u8; 3];
                    let mut num_digits = 0;
                    
                    if len == 0 {
                        digits[0] = b'0';
                        num_digits = 1;
                    } else {
                        let mut n = len;
                        while n > 0 {
                            digits[num_digits] = b'0' + (n % 10) as u8;
                            n /= 10;
                            num_digits += 1;
                        }
                        // Reverse the digits
                        for i in 0..num_digits/2 {
                            let temp = digits[i];
                            digits[i] = digits[num_digits - 1 - i];
                            digits[num_digits - 1 - i] = temp;
                        }
                    }
                    
                    let _ = serial.write(&digits[0..num_digits]);
                    let _ = serial.write(b"\r\n");
                }
                
                // We received something - check if it's our trigger message
                if rx_packet.len() as usize == TRIGGER_MESSAGE.len() && 
                   &rx_packet[..] == TRIGGER_MESSAGE {
                    
                    // Blink green LED to indicate received trigger
                    leds.ld2_blue.off();
                    leds.ld2_green.on();
                    
                    // Log trigger detection
                    if usb_dev.poll(&mut [&mut serial]) {
                        let _ = serial.write(b"Trigger message detected!\r\n");
                    }
                    
                    tx_packet.copy_from_slice(RESPONSE_MESSAGE);
                    // Small delay to make sure receiver is ready
                    timer.wait(core::time::Duration::from_millis(10));
                    
                    // Log before sending response
                    if usb_dev.poll(&mut [&mut serial]) {
                        let _ = serial.write(b"Sending response: ");
                        let _ = serial.write(RESPONSE_MESSAGE);
                        let _ = serial.write(b"\r\n");
                    }
                    
                    radio.send(&mut tx_packet);
                    
                    // LED feedback
                    leds.ld2_green.off();
                    leds.ld2_red.on();
                    timer.wait(core::time::Duration::from_millis(100));
                    leds.ld2_red.off();
                }
            },
            Err(ieee802154::Error::Timeout) => {
                leds.ld2_blue.off();
                
                // Log periodic heartbeat to show the device is alive
                counter += 1;
                if counter % 10 == 0 && usb_dev.poll(&mut [&mut serial]) {
                    let _ = serial.write(b"Waiting for packets... (");
                    
                    // Convert counter to ASCII digits
                    let count = counter / 10;
                    let mut digits = [0u8; 10]; // Enough for u32 max value
                    let mut num_digits = 0;
                    
                    if count == 0 {
                        digits[0] = b'0';
                        num_digits = 1;
                    } else {
                        let mut n = count;
                        while n > 0 {
                            digits[num_digits] = b'0' + (n % 10) as u8;
                            n /= 10;
                            num_digits += 1;
                        }
                        // Reverse the digits
                        for i in 0..num_digits/2 {
                            let temp = digits[i];
                            digits[i] = digits[num_digits - 1 - i];
                            digits[num_digits - 1 - i] = temp;
                        }
                    }
                    
                    let _ = serial.write(&digits[0..num_digits]);
                    let _ = serial.write(b")\r\n");
                }
            },
            Err(e) => {
                // Error in reception (e.g., CRC error)
                if usb_dev.poll(&mut [&mut serial]) {
                    let _ = writeln!(serial, "Reception error: {:?}", e);
                }
                
                leds.ld2_red.on();
                timer.wait(core::time::Duration::from_millis(50));
                leds.ld2_red.off();
            }
        }
        
        timer.wait(core::time::Duration::from_millis(10));
    }
}