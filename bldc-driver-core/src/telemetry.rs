use bldc_driver_hal::IntAngle;
use bldc_driver_hal::TelemetryFlash;
use core::slice;
use embassy_time::{Duration, Instant, Ticker};
use heapless::Vec;
use zerocopy::FromBytes;
use zerocopy::KnownLayout;

use crate::ControllerDataReceiver;
use crate::EncoderReceiver;
use crate::KinematicEstReceiver;

/// Used u64 and not Instant for the timestamp because u64 implements the zerocopy traits.
/// Total struct size: 24 bytes
#[repr(C)]
#[derive(Clone, Copy, FromBytes, KnownLayout)]
pub struct TelemetryData<const BITS: usize> {
    timestamp: u64,                  // 8 bytes
    measured_angle: IntAngle<BITS>,  // 4 bytes
    estimated_angle: IntAngle<BITS>, // 4 bytes
    estimated_velocity: i32,         // 4 bytes
    pwm_a: u32,
    pwm_b: u32,
    pwm_c: u32,
    // _pad: u32,                       // 4 bytes -> Total 32 bytes
}

impl<const BITS: usize> TelemetryData<BITS> {
    pub fn as_bytes(&self) -> &[u8] {
        let ptr = (self as *const Self) as *const u8;
        let len = core::mem::size_of::<Self>(); // Struct dimension, 24 bytes
        unsafe { slice::from_raw_parts(ptr, len) } // We create a slice of 24 u8 (bytes)
    }
}

impl<const BITS: usize> TelemetryData<BITS> {
    fn new() -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            ..Default::default()
        }
    }
}

impl<const BITS: usize> Default for TelemetryData<BITS> {
    fn default() -> Self {
        Self {
            timestamp: Default::default(),
            measured_angle: Default::default(),
            estimated_angle: Default::default(),
            estimated_velocity: Default::default(),
            pwm_a: Default::default(),
            pwm_b: Default::default(),
            pwm_c: Default::default(),
            // _pad: Default::default(),
        }
    }
}

/// Telemetry task loop, intended to be run in an embassy task
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn telemetry_task() {
///     telemetry_run<{BITS}, {BUFFER_LEN}>(/* ... */)
///     // handle telemetry end...
/// }
/// ```
pub async fn telemetry_run<const BITS: usize, const BUFFER_LEN: usize>(
    frequency: u32,
    duration_us: u64,
    mut encoder_rx: EncoderReceiver<BITS>,
    mut kin_est_rx: KinematicEstReceiver<BITS>,
    mut controller_rx: ControllerDataReceiver<BITS>,
    mut flash: impl TelemetryFlash<TelemetryData<BITS>, BUFFER_LEN>,
) {
    let mut buffer: Vec<TelemetryData<BITS>, BUFFER_LEN> = Vec::new();
    let mut ticker = Ticker::every(Duration::from_micros(1_000_000 / (frequency as u64)));

    let buffer_use_len = ((frequency as u64) * duration_us / 1_000_000).min(BUFFER_LEN as u64);

    for _ in 0..buffer_use_len {
        let mut data = TelemetryData::new();

        if let Some(encoder_data) = encoder_rx.try_get() {
            data.measured_angle = encoder_data.angle;
            // data.encoder_timestamp = encoder_data.timestamp as u32;
        }

        if let Some(kin_est_data) = kin_est_rx.try_get() {
            data.estimated_angle = kin_est_data.angle;
            data.estimated_velocity = kin_est_data.velocity;
            // data.estimation_timestamp = kin_est_data.timestamp as u32;
        }

        if let Some(controller_data) = controller_rx.try_get() {
            // data.estimation_timestamp = controller_data.dt as u32;
            // data.encoder_timestamp = controller_data.reg;
            data.pwm_a = controller_data.pwm_a;
            data.pwm_b = controller_data.pwm_b;
            data.pwm_c = controller_data.pwm_c;
        }

        if buffer.push(data).is_err() {
            break;
        }

        ticker.next().await;
    }

    flash.write_data_vec(buffer).await;
}
