use bldc_driver_hal::{Encoder, EncoderData, IntAngle};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_2, PIN_3, PIN_4, PIN_5, SPI0};
use embassy_rp::{Peri, dma, interrupt};

use embassy_rp::spi::{Async, Config, Spi};
use embassy_time::Instant;

/// AS5048A Spi Magnetic encoder
pub struct SpiEncoder {
    cs: Output<'static>,
    spi: Spi<'static, SPI0, Async>,
}

impl SpiEncoder {
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
}

// encoder precision fixed to 14 bits
pub const ENCODER_BITS: usize = 14;

const ENCODER_OFFSET: IntAngle<ENCODER_BITS> = IntAngle::from_raw(7781);

impl Encoder<ENCODER_BITS> for SpiEncoder {
    const ENCODER_FREQUENCY_HZ: u32 = 100_000;

    fn read_value_blocking(&mut self) -> EncoderData<ENCODER_BITS> {
        // let start = Instant::now();

        let buf_tx = [0xffu8; 2];
        let mut buf_rx = [0xffu8; 2];
        // electrical_angle += angle_step;
        self.cs.set_low();
        let _ = self.spi.blocking_transfer(&mut buf_rx, &buf_tx);
        self.cs.set_high();

        let _parity = (buf_rx[0] & 0x80) >> 7;
        let _error = (buf_rx[0] & 0x40) >> 6;

        let angle_raw = ((buf_rx[0] & 0x3f) as u16) << 8 | (buf_rx[1] as u16);
        // let angle: f32 = angle_raw as f32 / 16383.0 * 360.0;

        // let end = Instant::now();

        EncoderData {
            angle: -IntAngle::from_raw(angle_raw as i32) - ENCODER_OFFSET,
            timestamp: Instant::now().as_micros(),
        }
    }
}
