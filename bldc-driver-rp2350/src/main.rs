#![no_std]
#![no_main]

use bldc_driver_core::generate_bldc_driver_tasks;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, USB};
use embassy_rp::usb;
use embassy_time::Timer;
// use panic_probe as _;
use panic_persist as _;

use crate::bldc_motor::RpBldcMotor;
use crate::encoder::{ENCODER_BITS, SpiEncoder};
use crate::flash::RpFlash;

mod bldc_motor;
mod encoder;
mod flash;

generate_bldc_driver_tasks!(
    crate::encoder::SpiEncoder,
    crate::bldc_motor::RpBldcMotor,
    crate::flash::RpFlash,
    crate::encoder::ENCODER_BITS,
    crate::flash::RpFlash::BUFFER_LEN,
);

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>, embassy_rp::dma::InterruptHandler<DMA_CH2>;
});

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

const TELEMETRY_FREQUENCY: u32 = 1_000;
const TELEMETRY_DURATION_US: u64 = 2_000_000;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Ensure PLL is enabled and configured for 150 MHz
    // let config = config::Config::new(ClockConfig::crystal(100_000_000));
    // let p = embassy_rp::init(config);
    let p = embassy_rp::init(Default::default());

    // Take panic message from RAM before anything else
    let panic_msg = panic_persist::get_panic_message_utf8();

    // Spawn USB logger
    let driver = usb::Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver).expect("Failed to create logger task"));

    let mut led = Output::new(p.PIN_25, Level::Low);
    led.set_high();

    // If the previus run crashed, print the panic message in loop
    if let Some(msg) = panic_msg {
        led.set_high();

        loop {
            log::error!("==== PREVIOUS CRASH RECOVERED ====\n{}", msg);

            // Wait 1 second and print again.
            // Whenever you open your terminal, you will see it.
            embassy_time::Timer::after_secs(1).await;
            led.set_level(if led.is_set_high() {
                Level::Low
            } else {
                Level::High
            });
        }
    }

    let flash = RpFlash::new(p.FLASH, p.DMA_CH2, Irqs);

    let motor = RpBldcMotor::new(
        p.PIN_6,
        p.PIN_7,
        p.PIN_8,
        p.PIN_9,
        p.PWM_SLICE3,
        p.PWM_SLICE4,
    );

    let encoder = SpiEncoder::new(
        p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.SPI0, p.DMA_CH0, p.DMA_CH1, Irqs,
    );

    Timer::after_secs(1).await;

    log::info!("Setup done");

    run_bldc_driver_loop(spawner, motor, encoder);

    led.set_low();
    let telemetry_end = run_telemetry(spawner, flash, TELEMETRY_FREQUENCY, TELEMETRY_DURATION_US);

    log::info!("Everything run");

    telemetry_end.wait().await;
    led.set_high();

    loop {
        log::info!("Telemetry end");

        Timer::after_secs(1).await;
    }
}
