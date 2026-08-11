use bldc_driver_hal::{Encoder, EncoderData, EncoderReceiver, IntAngle};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_2, PIN_3, PIN_4, PIN_5, SPI0};

use embassy_rp::spi::{Async, Config, Spi};
use embassy_time::Instant;

/// AS5048A Spi Magnetic encoder
pub struct SpiEncoder<'a> {
    cs: Output<'a>,
    spi: Spi<'a, SPI0, Async>,
}

impl<'a> SpiEncoder<'a> {
    pub fn new(
        pin_2: PIN_2,
        pin_3: PIN_3,
        pin_4: PIN_4,
        pin_5: PIN_5,
        spi_0: SPI0,
        dma_ch0: DMA_CH0,
        dma_ch1: DMA_CH1,
    ) -> Self {
        let clk = pin_2;
        let mosi = pin_3;
        let miso = pin_4;
        let cs = Output::new(pin_5, Level::High);

        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000; // max 10 MHz (10_000_000)
        spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;

        let spi = Spi::new(spi_0, clk, mosi, miso, dma_ch0, dma_ch1, spi_config);

        Self { cs, spi }
    }

    pub async fn read_value(&mut self) -> EncoderData<ENCODER_BITS> {
        let buf_tx = [0xffu8; 2];
        let mut buf_rx = [0xffu8; 2];

        self.cs.set_low();
        self.spi.transfer(&mut buf_rx, &buf_tx).await.unwrap();
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

impl<'a> Encoder<ENCODER_BITS> for SpiEncoder<'a> {
    fn read_stream(&self) -> (u32, EncoderReceiver<'_, ENCODER_BITS>) {
        todo!()
    }
}
