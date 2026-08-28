use bldc_driver_hal::IntAngle;

use crate::{EncoderReceiver, KinematicEstSender};

/// PLL observer (used to reduce voltage phase lag)
struct PllObserver<const BITS: usize, const MAX_SPEED_RPM: i32> {
    /// Estimated angle
    angle_est: IntAngle<BITS>,
    /// Estimated angular velocity (in IntAngle/s)
    velocity_est: i32,
    /// Internal state of the PI controller
    integral_term: i32,
    /// Proportional gain (numerator)
    kp_num: i32,
    /// Proportional gain (denominator)
    kp_den: i32,
    /// Integral gain multiplied by sample time (numerator)
    ki_ts_num: i32,
    /// Integral gain multiplied by sample time (denominator)
    ki_ts_den: i32,
    /// Sample period in microseconds
    period_us: i32,
}

impl<const BITS: usize, const MAX_SPEED_RPM: i32> PllObserver<BITS, MAX_SPEED_RPM> {
    /// Checks if the used configuration of BITS and MAX_SPEED_RPM will fit in the i32 variables
    /// Call this before creating an instance of PllObserver to ensure the assert will not fail!
    pub const fn check_max_speed() -> bool {
        MAX_SPEED_RPM <= i32::MAX / (1 << BITS) * 60
    }

    /// Creates a new PLL Observer
    /// `period_us`: Sample period in microseconds (e.g., 10us for 100kHz)
    /// `bandwidth_hz`: Filter cutoff frequency (e.g., 200.0)
    pub fn new(period_us: i32, bandwidth_hz: i32) -> Self {
        // Fails at runtime if check fails
        assert!(
            Self::check_max_speed(),
            "MAX_SPEED_RPM exceeds the speed that can be set in an i32: \ni32::MAX / (1 << BITS) * 60"
        );

        let omega_n = IntAngle::<BITS>::A360.raw_value() * bandwidth_hz;
        let zeta = core::f32::consts::FRAC_1_SQRT_2; // ~0.707

        let ki = omega_n * omega_n;

        Self {
            angle_est: IntAngle::new(0),
            velocity_est: 0,
            integral_term: 0,
            kp_num: omega_n * ((2.0 * zeta * 1_000.0) as i32),
            kp_den: 1_000,
            ki_ts_num: ki * period_us / 1_000,
            ki_ts_den: 1_000,
            period_us,
        }
    }

    /// Updates the observer with a new raw encoder angle
    #[inline(always)]
    pub fn update(&mut self, angle_read: IntAngle<BITS>) -> (IntAngle<BITS>, i32) {
        // Phase Detector: Calculate estimation error
        let error = (angle_read - self.angle_est).raw_value();

        // Loop Filter: PI controller calculates the estimated velocity
        let proportional = self.kp_num * error / self.kp_den;
        self.integral_term += self.ki_ts_num * error / self.ki_ts_den;

        self.velocity_est = proportional + self.integral_term;

        // Integrator: Calculate next estimated angle
        self.angle_est += IntAngle::from_raw(self.velocity_est * self.period_us / 1_000_000);

        (self.angle_est, self.velocity_est)
    }

    pub fn _angle_est(&self) -> IntAngle<BITS> {
        self.angle_est
    }

    pub fn _velocity_est(&self) -> i32 {
        self.velocity_est
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KinematicEst<const BITS: usize> {
    pub angle: IntAngle<BITS>,
    pub velocity: i32,
    pub timestamp: u64,
}

/// Pll observer task loop, intended to be run in an embassy task
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn pll_observer_task() {
///     pll_observer_run<{BITS}>(/* ... */)
/// }
/// ```
pub async fn pll_observer_run<const BITS: usize, const MAX_SPEED_RPM: i32>(
    mut receiver: EncoderReceiver<BITS>,
    sender: KinematicEstSender<BITS>,
    period_us: i32,
    bandwidth_hz: i32,
) -> ! {
    let mut pll = PllObserver::<BITS, MAX_SPEED_RPM>::new(period_us, bandwidth_hz);

    loop {
        let encoder_data = receiver.changed().await;

        let (angle, velocity) = pll.update(encoder_data.angle);

        sender.send(KinematicEst {
            angle,
            velocity,
            timestamp: encoder_data.timestamp,
        });
    }
}
