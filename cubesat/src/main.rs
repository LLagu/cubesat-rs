#![no_std]
#![no_main]
use rtt_target::{rtt_init_print, rprintln};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use cortex_m::asm;
use cortex_m_rt::entry;

// use spacepackets::{ecss::{tc::PusTcSecondaryHeader, tm::PusTmCreator}, time::cds::CdsTime, SpHeader};
use spacepackets::{ecss::tm::PusTmCreator, *};


const APID: u16 =  0x123;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting cubesat telemetry routine...");

    let voltage_mock: f64  = 42.0;

    loop {
        let ccsds_primary_header = SpHeader::new_from_apid(APID);
        let data: [u8; 8] = voltage_mock.to_be_bytes();
        let pus_tm_secondary_header = ecss::tm::PusTmSecondaryHeader::new_simple_no_timestamp(3, 5);
        let pus_voltage_tm = PusTmCreator::new(ccsds_primary_header, pus_tm_secondary_header, &data, false);
    }
}