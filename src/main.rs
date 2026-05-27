#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // Onboard LED is typically on Pin 25 on the original Raspberry Pi Pico
    let mut led = Output::new(p.PIN_25, Level::High);

    loop {
        led.set_level(Level::High);
        Timer::after_millis(500).await;

        led.set_level(Level::Low);
        Timer::after_millis(500).await;
    }
}
