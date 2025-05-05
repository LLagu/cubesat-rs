#![no_std]
#![no_main]


use core::str;

use rtt_target::{rprintln, rtt_init_print};

use cortex_m::asm;
use cortex_m_rt::entry;
use nrf52840_hal::{self as hal, ieee802154::Error};
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

// use spacepackets::{ecss::{tc::PusTcSecondaryHeader, tm::PusTmCreator}, time::cds::CdsTime, SpHeader};
use spacepackets::{ecss::tm::PusTmCreator, *};
use nrf52840_hal::ieee802154::Packet;
pub mod radio_setup;

const APID: u16 = 0x123;
const ONE_SEC_IN_CYCLES: u32 = 1000000;
const TEN_MS: u32 = 10_000;


#[entry]
fn main() -> ! {
    rtt_init_print!();
    let p = hal::pac::Peripherals::take().unwrap();
    let mut timer = hal::Timer::new(p.TIMER0);

    // let radio = p.RADIO;
    // let mut radio = hal::ieee802154::Radio::init(p.RADIO, clocks);
    // radio.mode.write(|w| w.mode().ieee802154_250kbit());
    // radio.txpower.write(|w| w.txpower().pos8d_bm());
    let mut radio = radio_setup::init(p.RADIO, p.CLOCK).unwrap();
    radio.set_channel(nrf52840_hal::ieee802154::Channel::_20);
    radio.set_txpower(nrf52840_hal::ieee802154::TxPower::Pos8dBm);
    
    rprintln!("Starting cubesat telemetry routine...");
    let voltage_mock: f64 = 42.0;
    let mut tm_buffer: [u8; 40] = [0; 40];

    loop {
        let mut packet = Packet::new();
        let res = radio.recv_timeout(&mut packet, &mut timer, TEN_MS);

        match res {
           Ok(crc) => {
            let ccsds_primary_header = SpHeader::new_from_apid(APID);
            let data: [u8; 8] = voltage_mock.to_be_bytes();
            let pus_tm_secondary_header = ecss::tm::PusTmSecondaryHeader::new_simple_no_timestamp(3, 5);
            let pus_voltage_tm =
                PusTmCreator::new(ccsds_primary_header, pus_tm_secondary_header, &data, false);
            pus_voltage_tm
                .write_to_bytes(&mut tm_buffer)
                .expect("Error serializing TM data.");
    
            let mut packet = Packet::new();
            packet.copy_from_slice(&tm_buffer);
            radio.send(&mut packet);
    
            timer.delay(ONE_SEC_IN_CYCLES);
           }
           _ => {} 
        }


        }     
    }

