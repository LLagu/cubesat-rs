#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m_rt::entry;

use dongle as dk;
use dk::ieee802154::{self, Channel, Packet, TxPower};


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {

    let board = dk::init().unwrap();
    let mut leds = board.leds;
    let mut timer = board.timer;
    let mut radio = board.radio;
    

    radio.set_channel(Channel::_20);  
    radio.set_txpower(TxPower::_0dBm);
    
    leds.ld1.on();
    
    let mut rx_packet = Packet::new();
    let mut tx_packet = Packet::new();
    
    const TRIGGER_MESSAGE: &[u8] = b"START";
    const RESPONSE_MESSAGE: &[u8] = b"HELLO FROM NRF52840";
    
    loop {
        leds.ld2_blue.on();
        
        // Try to receive a packet with timeout (1 second)
        match radio.recv_timeout(&mut rx_packet, &mut timer, 1_000_000) {
            Ok(_) => {
                // We received something - check if it's our trigger message
                if rx_packet.len() as usize == TRIGGER_MESSAGE.len() && 
                   &rx_packet[..] == TRIGGER_MESSAGE {
                    
                    // Blink green LED to indicate received trigger
                    leds.ld2_blue.off();
                    leds.ld2_green.on();
                    
                    tx_packet.copy_from_slice(RESPONSE_MESSAGE);
                    // Small delay to make sure receiver is ready
                    timer.wait(core::time::Duration::from_millis(10));
                    
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
            },
            Err(_) => {
                // // Error in reception (e.g., CRC error)
                // leds.ld2_red.on();
                // timer.wait(core::time::Duration::from_millis(50));
                // leds.ld2_red.off();
            }
        }
        
        timer.wait(core::time::Duration::from_millis(10));
    }
}

#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};
use cortex_m_rt::entry;
use defmt::info;

// Import board support package and radio module
use dongle as dk;
use dk::ieee802154::{self, Channel, Packet, TxPower};

// LED control for visual feedback
use dk::Leds;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    defmt::error!("Panic occurred: {:?}", defmt::Debug2Format(info));
    loop {}
}

// #[entry]
// fn main() -> ! {
//     // Initialize the board
//     let board = dk::init().unwrap();
    
//     // Print welcome message to RTT console
//     info!("nRF52840 Radio Firmware Started");
//     info!("Listening for \"START\" message on channel 20");
    
//     // Split components
//     let mut leds = board.leds;
//     let mut timer = board.timer;
//     let mut radio = board.radio;
    
//     // Configure radio
//     radio.set_channel(Channel::_20);  // Use channel 20 (2_450 MHz)
//     radio.set_txpower(TxPower::_0dBm);
    
//     // Turn on LD1 to indicate we're ready
//     leds.ld1.on();
    
//     // Create buffers for receiving and sending
//     let mut rx_packet = Packet::new();
//     let mut tx_packet = Packet::new();
    
//     // Message we're looking for
//     const TRIGGER_MESSAGE: &[u8] = b"START";
//     // Response message
//     const RESPONSE_MESSAGE: &[u8] = b"HELLO FROM NRF52840";
    
//     // Main loop
//     loop {
//         // Visual indicator that we're listening
//         leds.ld2_blue.on();
        
//         // Try to receive a packet with timeout (1 second)
//         match radio.recv_timeout(&mut rx_packet, &mut timer, 1_000_000) {
//             Ok(crc) => {
//                 // Print received message details
//                 info!("Received packet: length={}, LQI={}, CRC=0x{:04x}", 
//                       rx_packet.len(), rx_packet.lqi(), crc);
                
//                 // Convert to string for printing (safely handling non-UTF8)
//                 if rx_packet.len() > 0 {
//                     info!("Message content: {:?}", &rx_packet[..]);
                    
//                     // Try to convert to string if it's valid UTF-8
//                     if let Ok(text) = core::str::from_utf8(&rx_packet[..]) {
//                         info!("Text: \"{}\"", text);
//                     }
//                 }
                
//                 // Check if it's our trigger message
//                 if rx_packet.len() as usize == TRIGGER_MESSAGE.len() && 
//                    &rx_packet[..] == TRIGGER_MESSAGE {
                    
//                     info!("Trigger message received! Sending response...");
                    
//                     // Blink green LED to indicate received trigger
//                     leds.ld2_blue.off();
//                     leds.ld2_green.on();
                    
//                     // Prepare response packet
//                     tx_packet.copy_from_slice(RESPONSE_MESSAGE);
                    
//                     // Add a small delay to make sure receiver is ready
//                     timer.wait(core::time::Duration::from_millis(10));
                    
//                     // Send response
//                     info!("Sending response: {:?}", RESPONSE_MESSAGE);
//                     radio.send(&mut tx_packet);
//                     info!("Response sent!");
                    
//                     // Visual indicator that we sent the response
//                     leds.ld2_green.off();
//                     leds.ld2_red.on();
//                     timer.wait(core::time::Duration::from_millis(100));
//                     leds.ld2_red.off();
//                 }
//             },
//             Err(ieee802154::Error::Timeout) => {
//                 // No packet received in timeout period, just continue
//                 leds.ld2_blue.off();
//             },
//             Err(ieee802154::Error::Crc(crc)) => {
//                 // CRC error in reception
//                 info!("CRC error detected (0x{:04x})", crc);
//                 leds.ld2_red.on();
//                 timer.wait(core::time::Duration::from_millis(50));
//                 leds.ld2_red.off();
//             },
//             Err(err) => {
//                 // Other errors
//                 info!("Error in reception: {:?}", defmt::Debug2Format(&err));
//                 leds.ld2_red.on();
//                 timer.wait(core::time::Duration::from_millis(50));
//                 leds.ld2_red.off();
//             }
//         }
        
//         // Small delay to avoid hammering the CPU
//         timer.wait(core::time::Duration::from_millis(10));
//     }
// }