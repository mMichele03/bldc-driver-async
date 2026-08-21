use bldc_driver_hal::{self, EncoderData};
use embassy_time::Timer;
use heapless::Vec;

pub struct TelemetryData {}

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
async fn telemetry_run<const ENCODER_BITS: usize, const BUFFER_SIZE: usize>(
    period_us: u64,
    mut rx: bldc_driver_hal::EncoderReceiver<ENCODER_BITS>,
) {
    let mut buffer: Vec<EncoderData<ENCODER_BITS>, BUFFER_SIZE> = Vec::new();

    loop {
        if let Some(reading) = rx.try_get() {
            if buffer.push(reading).is_err() {
                break;
            }
        }
        Timer::after_micros(period_us).await;
    }
}
