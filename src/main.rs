#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut counter = 0;
    loop {
        counter += 1;

        led.set_level(if counter % 2 == 0 {
            Level::High
        } else {
            Level::Low
        });
        Timer::after_millis(1000).await;
    }
}
