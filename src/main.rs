#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::spi::{Config, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use log::info;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

// Testing AS5048A SPI magnetic encoder

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    // let mut led = Output::new(p.PIN_25, Level::Low);

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let clk = p.PIN_2;
    let mosi = p.PIN_3;
    let miso = p.PIN_4;
    let mut cs = Output::new(p.PIN_5, Level::High);

    let mut spi_config = Config::default();
    spi_config.frequency = 10_000_000; // max 10 MHz (10_000_000)
    spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;

    let mut spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_config);

    info!("Hello World!");

    let mut counter = 0;
    loop {
        counter += 1;
        let buf_tx = [0xffu8; 2];
        let mut buf_rx = [0xffu8; 2];

        cs.set_low();
        spi.transfer(&mut buf_rx, &buf_tx).await.unwrap();
        cs.set_high();

        let parity = (buf_rx[0] & 0x80) >> 7;
        let error = (buf_rx[0] & 0x40) >> 6;
        let angle_raw = ((buf_rx[0] & 0x3f) as u16) << 8 | (buf_rx[1] as u16);
        let angle: f32 = angle_raw as f32 / 16383.0 * 360.0;

        log::info!(
            "tick {} [PAR {} ERR {}] raw {} angle {}",
            counter,
            parity,
            error,
            angle_raw,
            angle
        );

        Timer::after_millis(10).await;
    }
}
