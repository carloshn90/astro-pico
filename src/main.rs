//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod stepper;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp235x_hal::clocks::init_clocks_and_plls;
use rp235x_hal::{self as hal, entry};
use rp235x_hal::{Clock, pac};

use crate::stepper::{Direction, StepperMotor, ULN2003};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use some_bsp;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // stepper
    let pin2 = pins.gpio2.into_push_pull_output();
    let pin3 = pins.gpio3.into_push_pull_output();
    let pin4 = pins.gpio4.into_push_pull_output();
    let pin5 = pins.gpio5.into_push_pull_output();
    let mut motor = ULN2003::new(pin2, pin3, pin4, pin5);

    let mut count = 0;
    let mut direction = Direction::Normal;
    loop {
        if count < 5000 {
            motor.step().unwrap();
            delay.delay_ms(5);
            count += 1;
        } else {
            motor.stop().unwrap();
            if direction == Direction::Normal {
                direction = Direction::Reverse;
                motor.set_direction(Direction::Reverse);
            } else {
                direction = Direction::Normal;
                motor.set_direction(Direction::Normal);
            }
            count = 0;
        }
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [rp235x_hal::binary_info::EntryAddr; 5] = [
    rp235x_hal::binary_info::rp_cargo_bin_name!(),
    rp235x_hal::binary_info::rp_cargo_version!(),
    rp235x_hal::binary_info::rp_program_description!(c"RP2350 Template"),
    rp235x_hal::binary_info::rp_cargo_homepage_url!(),
    rp235x_hal::binary_info::rp_program_build_attribute!(),
];
