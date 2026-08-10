#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::Pwm;
use embassy_rp::pwm::{self, SetDutyCycle};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use fixed::FixedU16;
use fixed::types::extra::U4;
use log::info;
use panic_probe as _;

use crate::angle::IntAngle;

mod angle;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

// Testing AS5048A SPI magnetic encoder
// Testing BLDC motor

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // let clk = p.PIN_2;
    // let mosi = p.PIN_3;
    // let miso = p.PIN_4;
    // let mut cs = Output::new(p.PIN_5, Level::High);

    // let mut spi_config = spi::Config::default();
    // spi_config.frequency = 10_000_000; // max 10 MHz (10_000_000)
    // spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;

    // let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_config);

    let mut en = Output::new(p.PIN_9, Level::Low);

    let mut pwm_config: pwm::Config = Default::default();
    // freq = 60kHz // 2kHz
    pwm_config.phase_correct = true;
    const TOP: u16 = 1250; // 31578
    pwm_config.top = TOP;
    pwm_config.divider = FixedU16::<U4>::from_num(1.0); // 1.1875

    let mut pwm_ab = Pwm::new_output_ab(p.PWM_SLICE3, p.PIN_6, p.PIN_7, pwm_config.clone());
    let mut pwm_c = Pwm::new_output_a(p.PWM_SLICE4, p.PIN_8, pwm_config);

    let (pwm_a, pwm_b) = pwm_ab.split_by_ref();
    let mut pwm_a = pwm_a.unwrap();
    let mut pwm_b = pwm_b.unwrap();

    pwm_a.set_duty_cycle(0).unwrap();
    pwm_b.set_duty_cycle(0).unwrap();
    pwm_c.set_duty_cycle(0).unwrap();

    let mut angle = IntAngle::<12>::new(0);
    const SPEED_DEG_S: i32 = 5000; // max = 12000
    const PERIOD_US: u64 = 50;

    let delta_angle =
        IntAngle::<12>::from_raw(((SPEED_DEG_S * (PERIOD_US as i32) * (1 << 12)) / 360) / 1000000);
    // IntAngle::<12>::from_raw(1);

    info!("Hello World! {}", delta_angle);

    en.set_high();

    loop {
        let (a, b, c) = angle.bldc_3pwm((TOP / 2) as i32);
        pwm_a.set_duty_cycle(a as u16).unwrap();
        pwm_b.set_duty_cycle(b as u16).unwrap();
        pwm_c.set_duty_cycle(c as u16).unwrap();

        // info!("angle {angle} --- a {a} b {b} c {c}");

        angle += delta_angle;
        angle.normalize();

        Timer::after_micros(PERIOD_US).await;
    }

    //     Timer::after_secs(1).await;

    //     info!("Hello World!");

    //     let mut counter = 0;
    //     loop {
    //         counter += 1;
    //         let buf_tx = [0xffu8; 2];
    //         let mut buf_rx = [0xffu8; 2];

    //         cs.set_low();
    //         spi.transfer(&mut buf_rx, &buf_tx).await.unwrap();
    //         cs.set_high();

    //         let parity = (buf_rx[0] & 0x80) >> 7;
    //         let error = (buf_rx[0] & 0x40) >> 6;
    //         let angle_raw = ((buf_rx[0] & 0x3f) as u16) << 8 | (buf_rx[1] as u16);
    //         let angle: f32 = angle_raw as f32 / 16383.0 * 360.0;

    //         log::info!(
    //             "tick {} [PAR {} ERR {}] raw {} angle {}",
    //             counter,
    //             parity,
    //             error,
    //             angle_raw,
    //             angle
    //         );

    //         Timer::after_millis(10).await;
    //     }
}
