use bldc_driver_hal::{
    Encoder, EncoderData, EncoderReceiver, EncoderSender, EncoderWatch, IntAngle,
};

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_2, PIN_3, PIN_4, PIN_5, SPI0};
use embassy_rp::{Peri, dma, interrupt};

use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::watch::Watch;
use embassy_time::{Instant, Timer};
use log::info;

/// AS5048A Spi Magnetic encoder
pub struct SpiEncoder {
    cs: Output<'static>,
    spi: Spi<'static, SPI0, Async>,
}

impl SpiEncoder {
    const ENCODER_PERIOD_US: u64 = 10000; // Duration::from_secs(1).as_micros() / Self::ENCODER_FREQUENCY_HZ as u64;

    pub fn new(
        pin_2: Peri<'static, PIN_2>,
        pin_3: Peri<'static, PIN_3>,
        pin_4: Peri<'static, PIN_4>,
        pin_5: Peri<'static, PIN_5>,
        spi_0: Peri<'static, SPI0>,
        dma_ch0: Peri<'static, DMA_CH0>,
        dma_ch1: Peri<'static, DMA_CH1>,
        irq: impl interrupt::typelevel::Binding<
            interrupt::typelevel::DMA_IRQ_0,
            dma::InterruptHandler<DMA_CH0>,
        > + interrupt::typelevel::Binding<
            interrupt::typelevel::DMA_IRQ_0,
            dma::InterruptHandler<DMA_CH1>,
        > + 'static,
    ) -> Self {
        let clk = pin_2;
        let mosi = pin_3;
        let miso = pin_4;
        let cs = Output::new(pin_5, Level::High);

        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000; // max 10 MHz (10_000_000)
        spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;

        let spi = Spi::new(spi_0, clk, mosi, miso, dma_ch0, dma_ch1, irq, spi_config);

        Self { cs, spi }
    }

    pub async fn read_value(&mut self) -> EncoderData<ENCODER_BITS> {
        let buf_tx = [0xffu8; 2];
        let mut buf_rx = [0xffu8; 2];

        self.cs.set_low();
        if let Err(e) = self.spi.transfer(&mut buf_rx, &buf_tx).await {
            loop {
                info!("ERROR: {:?}", e);
                Timer::after_millis(10).await;
            }
        }

        self.cs.set_high();

        let _parity = (buf_rx[0] & 0x80) >> 7;
        let _error = (buf_rx[0] & 0x40) >> 6;

        let angle_raw = ((buf_rx[0] & 0x3f) as u16) << 8 | (buf_rx[1] as u16);
        // let angle: f32 = angle_raw as f32 / 16383.0 * 360.0;

        EncoderData {
            angle: IntAngle::from_raw(angle_raw as i32),
            timestamp: Instant::now(),
            counter: 0,
        }
    }
}

// encoder precision fixed to 14 bits
pub const ENCODER_BITS: usize = 14;

pub type EncoderAngle = IntAngle<ENCODER_BITS>;

pub static WATCH: EncoderWatch<ENCODER_BITS> = Watch::new();

#[embassy_executor::task]
async fn encoder_task(mut encoder: SpiEncoder, sender: EncoderSender<ENCODER_BITS>) {
    // let mut i = 0;
    loop {
        let start_time = Instant::now();
        let data = encoder.read_value().await;
        sender.send(data);
        // sender.send(EncoderData {
        //     angle: EncoderAngle::from_raw(0),
        //     timestamp: Instant::now(),
        //     counter: 0,
        // });

        let dt = (Instant::now() - start_time).as_micros();

        if dt < SpiEncoder::ENCODER_PERIOD_US {
            Timer::after_micros(SpiEncoder::ENCODER_PERIOD_US - dt).await;
        } else {
            loop {
                info!("FALSE");
            }
        }

        // if i == 0 {
        //     info!("DT {}", dt);
        // } else {
        //     i += 1;
        // }
        // i %= 1000;

        Timer::after_micros(25).await;
    }
}

impl Encoder<ENCODER_BITS> for SpiEncoder {
    const ENCODER_FREQUENCY_HZ: u32 = 10000;

    fn read_stream(self, spawner: Spawner) -> EncoderReceiver<ENCODER_BITS> {
        spawner.spawn(encoder_task(self, WATCH.sender()).expect("Failed to allocate encoder task"));

        WATCH.receiver().unwrap()
    }
}
