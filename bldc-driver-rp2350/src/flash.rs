use bldc_driver_core::telemetry::TelemetryData;
use bldc_driver_hal::TelemetryFlash;
use embassy_rp::{
    Peri, dma,
    flash::{Async, Flash},
    interrupt,
    peripherals::{DMA_CH0, FLASH},
};
use heapless::Vec;

use crate::ENCODER_BITS;

pub struct RpFlash {
    flash: Flash<'static, FLASH, Async, { Self::FLASH_SIZE }>,
}

impl RpFlash {
    pub const BUFFER_SIZE: usize = 50;

    const ADDR_OFFSET: u32 = 0x100000;
    const FLASH_SIZE: usize = 2 * 1024 * 1024;

    pub fn new(
        flash: Peri<'static, FLASH>,
        dma_ch0: Peri<'static, DMA_CH0>,
        irq: impl interrupt::typelevel::Binding<
            interrupt::typelevel::DMA_IRQ_0,
            dma::InterruptHandler<DMA_CH0>,
        > + 'static,
    ) -> Self {
        let flash =
            embassy_rp::flash::Flash::<_, Async, { Self::FLASH_SIZE }>::new(flash, dma_ch0, irq);

        RpFlash { flash }
    }
}

impl TelemetryFlash<TelemetryData<ENCODER_BITS>, { RpFlash::BUFFER_SIZE }> for RpFlash {
    fn write_data_vec(&mut self, data: Vec<TelemetryData<ENCODER_BITS>, { RpFlash::BUFFER_SIZE }>) {
        todo!()
    }
}
