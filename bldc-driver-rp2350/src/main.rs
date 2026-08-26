#![no_std]
#![no_main]

use bldc_driver_core::telemetry::telemetry_run;
use bldc_driver_hal::{BldcMotor, Encoder};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, USB};
use embassy_rp::usb;
use embassy_time::Timer;
use log::info;
// use panic_probe as _;
use panic_persist as _;

use crate::bldc_motor::RpBldcMotor;
use crate::encoder::{ENCODER_BITS, EncoderAngle, SpiEncoder, WATCH};
use crate::flash::RpFlash;

mod bldc_motor;
mod encoder;
mod flash;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>, embassy_rp::dma::InterruptHandler<DMA_CH2>;
});

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn telemetry_task(flash: RpFlash, period_us: u64, mut led: Output<'static>) {
    led.set_low();

    telemetry_run::<
        { ENCODER_BITS },
        { RpFlash::BUFFER_SIZE },
        { RpFlash::FLASH_SIZE },
        { RpFlash::PAGE_SIZE },
    >(period_us, WATCH.receiver().unwrap(), flash)
    .await;

    // telemetry end
    led.set_high();
    loop {
        Timer::after_secs(1).await;
    }
}

const TELEMETRY_PERIOD_US: u64 = 10000;
const LOOP_PERIOD_US: u64 = 10000;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());

    // Take panic message from RAM before anything else
    let panic_msg = panic_persist::get_panic_message_utf8();

    // Spawn USB logger
    let driver = usb::Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver).expect("Failed to create logger task"));

    // If the previus run crashed, print the panic message in loop
    if let Some(msg) = panic_msg {
        loop {
            log::error!("==== PREVIOUS CRASH RECOVERED ====\n{}", msg);

            // Wait 1 second and print again.
            // Whenever you open your terminal, you will see it.
            embassy_time::Timer::after_secs(1).await;
        }
    }
    let led = Output::new(p.PIN_25, Level::Low);

    let flash = RpFlash::new(p.FLASH, p.DMA_CH2, Irqs);
    spawner.spawn(
        telemetry_task(flash, TELEMETRY_PERIOD_US, led).expect("Failed to create telemetry task"),
    );

    let mut angle = EncoderAngle::new(0);

    let delta_angle = EncoderAngle::from_raw(100);
    // EncoderAngle::from_raw(((SPEED_DEG_S * (PERIOD_US as i32) * (1 << 14)) / 360) / 1000000);

    let mut motor = RpBldcMotor::new(
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

    // let _rec = encoder.read_stream(spawner);

    info!("Hello World! {}", delta_angle);

    Timer::after_secs(1).await;

    loop {
        // while let Some(val) = rec.try_changed() {
        // info!("[{}] value: {}", val.timestamp, val.angle);
        // }

        motor.set_magnetic_field(angle, RpBldcMotor::PWM_TOP / 2);

        angle += delta_angle;
        angle.normalize();

        // let data = encoder.read_value().await;
        // info!("value: {}", data.angle);

        Timer::after_micros(LOOP_PERIOD_US).await;
    }
}
