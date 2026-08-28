use bldc_driver_hal::IntAngle;
use bldc_driver_hal::TelemetryFlash;
use core::slice;
use embassy_time::{Duration, Instant, Ticker};
use heapless::Vec;
use zerocopy::FromBytes;
use zerocopy::KnownLayout;

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

impl<const BITS: usize> TelemetryData<BITS> {
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = (self as *const Self) as *const u8;
        let len = core::mem::size_of::<Self>(); // Struct dimension, 24 bytes
        unsafe { slice::from_raw_parts(ptr, len) } // We create a slice of 24 u8 (bytes)
    }
}

/// Telemetry task loop, intended to be run in an embassy task
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn telemetry_task() {
///     telemetry_run<{ENCODER_BITS}, {BUFFER_LEN}>(/* ... */)
///     // handle telemetry end...
/// }
/// ```
pub async fn telemetry_run<const ENCODER_BITS: usize, const BUFFER_LEN: usize>(
    frequency: u32,
    duration_us: u64,
    mut rx: bldc_driver_hal::EncoderReceiver<ENCODER_BITS>,
    mut flash: impl TelemetryFlash<TelemetryData<ENCODER_BITS>, BUFFER_LEN>,
) {
    let mut buffer: Vec<TelemetryData<ENCODER_BITS>, BUFFER_LEN> = Vec::new();
    let mut ticker = Ticker::every(Duration::from_micros(1_000_000 / (frequency as u64)));

    let buffer_use_len = ((frequency as u64) * duration_us / 1_000_000).min(BUFFER_LEN as u64);

    for _ in 0..buffer_use_len {
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
