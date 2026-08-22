use bldc_driver_hal::IntAngle;
use bldc_driver_hal::TelemetryFlash;
use embassy_time::Instant;
use embassy_time::Timer;
use heapless::Vec;

pub struct TelemetryData<const BITS: usize> {
    estimated_angle: IntAngle<BITS>,
    measured_angle: IntAngle<BITS>,
    timestamp: Instant,
    speed_rpm: i32,
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
pub async fn telemetry_run<const ENCODER_BITS: usize, const BUFFER_SIZE: usize>(
    period_us: u64,
    mut rx: bldc_driver_hal::EncoderReceiver<ENCODER_BITS>,
    mut flash: impl TelemetryFlash<TelemetryData<ENCODER_BITS>, BUFFER_SIZE>,
) {
    let mut buffer: Vec<TelemetryData<ENCODER_BITS>, BUFFER_SIZE> = Vec::new();

    loop {
        if let Some(reading) = rx.try_get() {
            let data = TelemetryData {
                estimated_angle: IntAngle::new(0),
                measured_angle: reading.angle,
                timestamp: Instant::now(),
                speed_rpm: 0,
            };
            if buffer.push(data).is_err() {
                break;
            }
        }

        Timer::after_micros(period_us).await;
    }

    flash.write_data_vec(buffer);
}
