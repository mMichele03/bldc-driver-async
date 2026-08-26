use bldc_driver_hal::IntAngle;
use bldc_driver_hal::TelemetryFlash;
use core::fmt::Write;
use core::slice;
use embassy_time::{Duration, Instant, Ticker};
use heapless::String;
use heapless::Vec;
use zerocopy::FromBytes;
use zerocopy::KnownLayout;

#[repr(C)]
#[derive(Clone, Copy, KnownLayout)]
pub struct LocalIntAngle<const BITS: usize>(IntAngle<BITS>);

/// Used u64 and not Instant for the timestamp because u64 implements the zerocopy traits.
/// Total struct size: 24 bytes
#[repr(C)]
#[derive(Clone, Copy, FromBytes, KnownLayout)]
pub struct TelemetryData<const BITS: usize> {
    timestamp: u64,                  // 8 bytes
    estimated_angle: IntAngle<BITS>, // 4 bytes
    measured_angle: IntAngle<BITS>,  // 4 bytes
    speed_rpm: i32,                  // 4 bytes
    enc_dt1: u32,                    // 4 bytes
    enc_dt2: u32,                    // 4 bytes
    _pad: u32,                       // 4 bytes -> Total 32 bytes
}

const CSV_ROW_BYTES: usize = 64; // number of bytes that can be used to represent a csv row of telemetry data 

impl<const BITS: usize> TelemetryData<BITS> {
    pub fn into_csv_row(&self) -> String<{ CSV_ROW_BYTES }> {
        self.into()
    }
}

impl<const BITS: usize> TelemetryData<BITS> {
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = (self as *const Self) as *const u8;
        let len = core::mem::size_of::<Self>(); // Struct dimension, 24 bytes
        unsafe { slice::from_raw_parts(ptr, len) } // We create a slice of 24 u8 (bytes)
    }
}

impl<const BITS: usize> From<&TelemetryData<BITS>> for String<{ CSV_ROW_BYTES }> {
    fn from(data: &TelemetryData<BITS>) -> Self {
        let mut s = String::<{ CSV_ROW_BYTES }>::new();
        let _ = write!(
            &mut s,
            "{},{},{},{}",
            data.timestamp, data.estimated_angle, data.measured_angle, data.speed_rpm
        );
        s
    }
}

/// Telemetry task loop, intended to be run in an embassy task
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn telemetry_task() {
///     telemetry_run<{ENCODER_BITS}, {BUFFER_SIZE}>(/* ... */)
///     // handle telemetry end...
/// }
/// ```
pub async fn telemetry_run<
    const ENCODER_BITS: usize,
    const BUFFER_SIZE: usize,
    const FLASH_SIZE: usize,
    const PAGE_SIZE: usize,
>(
    period_us: u64,
    mut rx: bldc_driver_hal::EncoderReceiver<ENCODER_BITS>,
    mut flash: impl TelemetryFlash<TelemetryData<ENCODER_BITS>, BUFFER_SIZE>,
) {
    let mut buffer: Vec<TelemetryData<ENCODER_BITS>, BUFFER_SIZE> = Vec::new();
    let mut ticker = Ticker::every(Duration::from_micros(period_us));

    loop {
        if let Some(reading) = rx.try_get() {
            let data = TelemetryData {
                estimated_angle: IntAngle::new(0),
                measured_angle: reading.angle,
                timestamp: Instant::now().as_micros(),
                speed_rpm: 0,
                enc_dt1: reading.timestamp as u32,
                enc_dt2: 0,
                _pad: 0,
            };
            if buffer.push(data).is_err() {
                break;
            }
        }

        ticker.next().await;
    }

    flash.write_data_vec(buffer).await;
}
