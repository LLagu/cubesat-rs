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
    // let mut usb_device = board.usbd;
    

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