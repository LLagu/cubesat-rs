use core::fmt::Error;

use hal::{
    clocks::{self, Clocks},
    gpio::{Level, Output, Pin, Port, PushPull, p0},
    rtc::{Rtc, RtcInterrupt},
    timer::OneShot,
};
use nrf52840_hal::{self as hal, pac::CLOCK};
use grounded::uninit::GroundedCell;
use hal::ieee802154;

struct ClockSyncWrapper<H, L, LSTAT> {
    clocks: Clocks<H, L, LSTAT>,
}
unsafe impl<H, L, LSTAT> Sync for ClockSyncWrapper<H, L, LSTAT> {}

pub fn init(radio: nrf52840_hal::pac::RADIO, clock: nrf52840_hal::pac::CLOCK) -> Result<ieee802154::Radio<'static>, Error> {
    // let p: nrf52840_hal::pac::Peripherals = hal::pac::Peripherals::take().unwrap();
    // We need the wrapper to make this type Sync, as it contains raw pointers
    static CLOCKS: GroundedCell<
        ClockSyncWrapper<
            clocks::ExternalOscillator,
            clocks::ExternalOscillator,
            clocks::LfOscStarted,
        >,
    > = GroundedCell::uninit();

    // let clocks = Clocks::new(p.CLOCK);
    let clocks = Clocks::new(clock);
    let clocks = clocks.enable_ext_hfosc();
    let clocks = clocks.set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass);
    let clocks = clocks.start_lfclk();
    let _clocks = clocks.enable_ext_hfosc();

    let clocks = unsafe {
        let clocks_ptr = CLOCKS.get();
        clocks_ptr.write(ClockSyncWrapper { clocks: _clocks });
        // Now it's initialised, we can take a static reference to the clocks
        // object it contains.
        let clock_wrapper: &'static ClockSyncWrapper<_, _, _> = &*clocks_ptr;
        &clock_wrapper.clocks
    };

    let radio = {
        let mut radio = ieee802154::Radio::init(radio, clocks);

        // set TX power to its maximum value
        radio.set_txpower(ieee802154::TxPower::Pos8dBm);
        defmt::debug!("Radio initialized and configured with TX power set to the maximum value");
        radio
    };
    Ok(radio)
}