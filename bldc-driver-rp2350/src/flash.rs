use bldc_driver_core::telemetry::TelemetryData;
use bldc_driver_hal::TelemetryFlash;
use embassy_rp::{
    Peri, dma,
    flash::{Async, Flash},
    interrupt,
    peripherals::{DMA_CH2, FLASH},
};
use embedded_storage_async::nor_flash::NorFlash;
use heapless::Vec;

use crate::ENCODER_BITS;

pub struct RpFlash {
    flash: Flash<'static, FLASH, Async, { Self::FLASH_SIZE }>,
    current_addr: u32,
}

impl RpFlash {
    const DATA_LEN: usize = core::mem::size_of::<TelemetryData<ENCODER_BITS>>();

    pub const BUFFER_SIZE_BYTES_MAX: usize = 5000;
    pub const BUFFER_SIZE: usize = Self::BUFFER_SIZE_BYTES_MAX / Self::DATA_LEN;

    pub const ADDR_OFFSET: u32 = 0x100000;
    pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
    pub const PAGE_SIZE: usize = 256;
    pub const SECTOR_SIZE: usize = 4096;

    pub fn new(
        flash: Peri<'static, FLASH>,
        dma_ch2: Peri<'static, DMA_CH2>,
        irq: impl interrupt::typelevel::Binding<
            interrupt::typelevel::DMA_IRQ_0,
            dma::InterruptHandler<DMA_CH2>,
        > + 'static,
    ) -> Self {
        let flash =
            embassy_rp::flash::Flash::<_, Async, { Self::FLASH_SIZE }>::new(flash, dma_ch2, irq);
        let current_addr: u32 = Self::ADDR_OFFSET;

        // Initialize the current flash address for multiple writings in flash

        RpFlash {
            flash,
            current_addr,
        }
    }
}

impl TelemetryFlash<TelemetryData<ENCODER_BITS>, { RpFlash::BUFFER_SIZE }> for RpFlash {
    async fn write_data_vec(
        &mut self,
        data: Vec<TelemetryData<ENCODER_BITS>, { RpFlash::BUFFER_SIZE }>,
    ) {
        const PAGE_SIZE: usize = RpFlash::PAGE_SIZE;
        const SECTOR_SIZE: usize = RpFlash::SECTOR_SIZE;

        let mut page_buffer = [0u8; PAGE_SIZE];
        let mut page_offset = 0;

        for entry in data.into_iter() {
            // let csv_row = entry.into_csv_row();
            // let csv_row_bytes = csv_row.as_bytes();
            let entry_bytes = entry.as_bytes();

            // Case in which the element exceeds the current 256 bytes page (we write and then reset the buffer_offset to write in a new one)
            if page_offset + Self::DATA_LEN > PAGE_SIZE {
                // If the page is at the beginning of a new sector, erase it and then write, else we can just write
                // normally since we are in an erased sector already.
                if (self.current_addr) as usize % SECTOR_SIZE == 0 {
                    let offset = self.current_addr;
                    self.erase_sector(offset).await;
                }

                // Write the complete 256 bytes page
                self.flash
                    .write(self.current_addr, &page_buffer)
                    .await
                    .unwrap();

                // Advance the flash address, reset the local buffer and reset the offest to 0
                self.current_addr += PAGE_SIZE as u32;
                page_buffer.fill(0);
                page_offset = 0;
            }

            // Copy the 24 bytes of the struct in the local page buffer
            page_buffer[page_offset..page_offset + Self::DATA_LEN].copy_from_slice(entry_bytes);
            page_offset += Self::DATA_LEN;
        }

        // Now, we handle cases in which the number of bytes to write is not a multiple of the page size
        if page_offset > 0 {
            // Fill the buffer with 0s next to the actual data to be able to write a page
            page_buffer[page_offset..].fill(0);

            // Same erase and write logic as before
            if (self.current_addr) as usize % SECTOR_SIZE == 0 {
                let offset = self.current_addr - Self::ADDR_OFFSET;
                self.erase_sector(offset).await;
            }

            self.flash
                .write(self.current_addr, &page_buffer)
                .await
                .unwrap();

            // Advance the address for the next function call
            self.current_addr += PAGE_SIZE as u32;
        }
    }
}

impl RpFlash {
    async fn erase_sector(&mut self, address: u32) {
        // Make sure that the address is aligned at 4 KB, so it marks the beginning of a sector
        debug_assert_eq!(
            address % Self::SECTOR_SIZE as u32,
            0,
            "The address is not aligned at 4 KB"
        );

        // Asynchronous erase of the flash sector
        self.flash
            .erase(address, address + Self::SECTOR_SIZE as u32)
            .await
            .unwrap();

        // NorFlash::erase(&mut self.flash, addr, addr + Self::SECTOR_SI)
        //     .await
        //     .unwrap();
    }
}
