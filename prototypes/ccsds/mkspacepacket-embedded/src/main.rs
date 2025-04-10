#![no_std]
#![no_main]
use rtt_target::{rtt_init_print, rprintln};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use spacepacket::{GroupingFlag, PacketType, SpacePacket};

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    rtt_init_print!();
    rprintln!("Hoi!");

    let payload = b"secret payload".to_vec();
    let packet = SpacePacket::new(
        0,
        PacketType::Command,
        0x012,
        GroupingFlag::Unsegm,
        3,
        true,
        payload,
    );
    
    let bytestream = packet.encode();
    let recovered_packet = SpacePacket::decode(&mut bytestream.as_slice()).unwrap();
    assert_eq!(packet, recovered_packet)
    loop {

    }
}