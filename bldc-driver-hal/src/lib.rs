#![no_std]

mod angle;

pub use angle::IntAngle;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::{Receiver, Sender, Watch};
use embassy_time::Instant;
use heapless::Vec;

/// Data returned by a single encoder reading
#[derive(Debug, Clone, Copy)]
pub struct EncoderData<const BITS: usize> {
    pub angle: IntAngle<BITS>,
    pub timestamp: Instant,
    pub counter: usize,
}

/// Consumer for the Encoder Data coming from the sensor's data stream
pub type EncoderReceiver<const BITS: usize> =
    Receiver<'static, CriticalSectionRawMutex, EncoderData<BITS>, 1>;
pub type EncoderSender<const BITS: usize> =
    Sender<'static, CriticalSectionRawMutex, EncoderData<BITS>, 1>;
pub type EncoderWatch<const BITS: usize> = Watch<CriticalSectionRawMutex, EncoderData<BITS>, 1>;

/// Encoder with an arbitrary number of the angle precision BITS
///
/// # Implementation example
///
/// ```
/// struct MyEncoder {}
///
/// // precision fixed to 14 bits
/// impl Encoder<14> for MyEncoder {
///     fn read_stream() -> (u32, ...) {
///         todo!()
///     }
/// }
/// ```
pub trait Encoder<const BITS: usize> {
    /// The frequency at which the encoder samples data
    const ENCODER_FREQUENCY_HZ: u32;

    /// Starts a stream of readings, returns immediately the read frequency and the watch receiver
    fn read_stream(self, spawner: Spawner) -> EncoderReceiver<BITS>;
}

/// BLDC motor controlled by 3-phase PWM
pub trait BldcMotor<const BITS: usize> {
    /// The maximum value the hardware timer reaches before wrapping
    const PWM_TOP: u32;

    /// The desired frequency of the motor control
    const PWM_FREQ: u32;

    /// Sets in hardware the values of the three PWM channels (scaled on TOP)
    fn set_pwm(&mut self, a: u32, b: u32, c: u32);

    /// Set the motor pwm according to the given angle, scaling the sinusoids magnitude (max is PWM_TOP)
    fn set_magnetic_field(&mut self, angle: IntAngle<BITS>, magnitude: u32) {
        let half_magnitude = magnitude.min(Self::PWM_TOP) / 2;

        let a = angle.sin_positive().scaled(half_magnitude);
        let b = (angle + IntAngle::A120)
            .sin_positive()
            .scaled(half_magnitude);
        let c = (angle + IntAngle::A240)
            .sin_positive()
            .scaled(half_magnitude);

        self.set_pwm(a, b, c);
    }
}

/// Flash intended to write telemetry data
/// I put the page size here so that the flash method automatically handles the alignment in multiples of the flash page
/// when the divisor is not a divider of the page size
pub trait TelemetryFlash<Data, const BUFFER_SIZE: usize> {
    /// Write data vec to flash
    async fn write_data_vec(&mut self, data: Vec<Data, BUFFER_SIZE>);
}
