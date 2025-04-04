#![no_std]
#![no_main]
use rtt_target::{rtt_init_print, rprintln};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    rtt_init_print!();
    rprintln!("Hoi!");
    loop {

    }
}