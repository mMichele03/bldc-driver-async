#![no_std]

mod angle;

pub use angle::IntAngle;
use embassy_time::Duration;
use heapless::Vec;

/// Data returned by a single encoder reading
#[derive(Debug, Clone, Copy)]
pub struct EncoderData<const BITS: usize> {
    pub angle: IntAngle<BITS>,
    pub timestamp: u64,
}

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
    /// The frequency at which the encoder can sample data
    const ENCODER_FREQUENCY_HZ: u32;

    /// The time period at which the encoder can sample data
    const ENCODER_PERIOD_US: u64 =
        Duration::from_secs(1).as_micros() / Self::ENCODER_FREQUENCY_HZ as u64;

    /// Reads a single value from the encoder
    /// Used in the implementation of the encoder task loop
    fn read_value_blocking(&mut self) -> EncoderData<BITS>;
}

/// BLDC motor controlled by 3-phase PWM
pub trait BldcMotor<const BITS: usize> {
    /// The maximum value the hardware timer reaches before wrapping
    const PWM_TOP: u32;

    /// The desired frequency of the motor control
    const PWM_FREQ: u32;

    /// The lag in microseconds between when `wake_to_set_pwm` returns
    /// and when the pwm will actually take effect (let's say the middle of the next waveform).
    /// Therefore, with 1-cycle delay, this lag defaults at 1.5 cycles.
    /// This is used internally by the core to estimate the angle at which the motor will be in that exact moment in the future.
    /// Note: we suggest implementing the pwm in phase-corrected mode.
    const PWM_CONTROL_LAG_US: u64 =
        (Duration::from_secs(1).as_micros() * 3) / (Self::PWM_FREQ as u64 * 2);

    /// Wakes up when the control loop has to configure the pwm for the next period,
    /// that is at the start or in the middle of the previous one.  
    /// The easiest implementation is to be triggered by the pwm reset, which introduces necessarily a full 1-cycle delay.
    /// If you are able to make an implementation that reduces this lag (or changes it anyway),
    /// please override the `PWM_CONTROL_LAG_US` constant to make the control loop work as intended.
    fn wake_to_set_pwm(&self) -> impl Future<Output = ()> + Send;

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
pub trait TelemetryFlash<Data, const BUFFER_LEN: usize> {
    /// Write data vec to flash
    fn write_data_vec(&mut self, data: Vec<Data, BUFFER_LEN>) -> impl Future<Output = ()> + Send;
}
