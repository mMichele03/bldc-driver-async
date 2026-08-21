#![no_std]
#![no_main]

use bldc_driver_hal::BldcMotor;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, USB};
use embassy_rp::usb;
use embassy_time::Timer;
use log::info;
use panic_probe as _;

use crate::bldc_motor::RpBldcMotor;
use crate::encoder::{EncoderAngle, SpiEncoder};

mod bldc_motor;
mod encoder;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
async fn logger_task(driver: usb::Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let _led = Output::new(p.PIN_25, Level::Low);

    let driver = usb::Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let mut angle = EncoderAngle::new(0);
    const PERIOD_US: u64 = 5;

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

    let mut encoder = SpiEncoder::new(
        p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.SPI0, p.DMA_CH0, p.DMA_CH1, Irqs,
    );

    // let mut rec = encoder.read_stream(spawner);

    info!("Hello World! {}", delta_angle);

    Timer::after_secs(1).await;

    loop {
        // while let Some(val) = rec.try_changed() {
        // info!("[{}] value: {}", val.timestamp, val.angle);
        // }

        motor.set_magnetic_field(angle, RpBldcMotor::PWM_TOP / 2);

        angle += delta_angle;
        angle.normalize();

        let data = encoder.read_value().await;
        info!("value: {}", data.angle);

        Timer::after_micros(PERIOD_US).await;
    }
}
