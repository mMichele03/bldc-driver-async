#![no_std]

mod angle;

pub use angle::IntAngle;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Receiver;
use embassy_time::Instant;
use log::debug;

/// Data returned by a single encoder reading
#[derive(Debug, Clone, Copy)]
pub struct EncoderData<const BITS: usize> {
    pub angle: IntAngle<BITS>,
    pub timestamp: Instant,
    pub counter: usize,
}

/// Consumer for the Encoder Data coming from the sensor's data stream
pub type EncoderReceiver<'a, const BITS: usize> =
    Receiver<'a, CriticalSectionRawMutex, EncoderData<BITS>, 1>;

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
    /// Starts a stream of readings, returns immediately the read frequency and the watch receiver
    fn read_stream(&self) -> (u32, EncoderReceiver<'_, BITS>);
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
        debug!("mag u32: {} i32: {}", magnitude, magnitude as i32);

        let half_magnitude = (magnitude.min(Self::PWM_TOP) / 2) as i32;

        debug!("half mag i32: {}", half_magnitude);

        let a = angle.sin().scaled_positive(half_magnitude);

        debug!("pwm a: {}", a);

        let b = (angle + IntAngle::A120)
            .sin()
            .scaled_positive(half_magnitude);

        debug!("pwm b: {}", b);

        let c = (angle + IntAngle::A240)
            .sin()
            .scaled_positive(half_magnitude);

        debug!("pwm c: {}", c);

        self.set_pwm(a as u32, b as u32, c as u32);
    }
}
