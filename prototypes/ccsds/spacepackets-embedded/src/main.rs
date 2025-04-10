#![no_std]
#![no_main]
use rtt_target::{rtt_init_print, rprintln};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use spacepackets::SpHeader;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    rtt_init_print!();
    rprintln!("Hoi!");

    let sp_header = SpHeader::new_for_unseg_tc_checked(0x42, 12, 1).expect("error creating CCSDS TC header");
    rprintln!("{:?}", sp_header);
    let mut ccsds_buf: [u8; 32] = [0; 32];
    sp_header.write_to_be_bytes(&mut ccsds_buf).expect("Writing CCSDS TC header failed");
    rprintln!("{:x?}", &ccsds_buf[0..6]);
    loop {

    }
}