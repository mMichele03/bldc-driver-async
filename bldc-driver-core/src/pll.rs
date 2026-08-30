use bldc_driver_hal::IntAngle;

use crate::{EncoderReceiver, KinematicEstSender};

/// PLL observer (used to reduce voltage phase lag)
#[derive(Debug)]
pub struct PllObserver<const BITS: usize, const MAX_SPEED_RPM: i32> {
    /// Estimated angle
    angle_est: i32,
    /// Estimated angular velocity (in IntAngle/s)
    velocity_est: i32,
    /// Internal state of the PI controller
    integral_term: i64,
    /// Proportional gain (numerator)
    kp_num: i32,
    /// Proportional gain (denominator)
    kp_den: i32,
    /// Integral gain multiplied by sample time (numerator)
    ki_ts_num: i64,
    /// Integral gain multiplied by sample time (denominator)
    ki_ts_den: i64,
    /// Sample period in microseconds
    period_us: i32,
}

impl<const BITS: usize, const MAX_SPEED_RPM: i32> PllObserver<BITS, MAX_SPEED_RPM> {
    /// The fixed internal precision of the PLL observer.
    const PLL_BITS: usize = 24;

    /// Bit shift required to move from encoder BITS to PLL_BITS.
    /// Rust will helpfully throw a compile-time error here if BITS > PLL_BITS
    /// due to usize underflow, acting as a built-in safety check.
    const SHIFT: usize = Self::PLL_BITS - BITS;

    const HALF_ROTATION: i32 = IntAngle::<BITS>::A180.raw_value() << Self::SHIFT;
    const FULL_ROTATION: i32 = IntAngle::<BITS>::A360.raw_value() << Self::SHIFT;

    /// Creates a new PLL Observer
    /// `period_us`: Sample period in microseconds (e.g., 10us for 100kHz)
    /// `bandwidth_hz`: Filter cutoff frequency (e.g., 200.0)
    pub fn new(period_us: i32, bandwidth_hz: i32) -> Self {
        let omega_n = (2.0 * core::f32::consts::PI * bandwidth_hz as f32) as i32;
        let zeta = core::f32::consts::FRAC_1_SQRT_2; // ~0.707
        let overshoot_percent = 105; // with zeta = 0.707

        // The 1_000 scale is used to transfer f32 precision to i32 without truncating to 0
        let kp = (omega_n * (2.0 * zeta * 1_000.0) as i32) / 1_000;

        let ki = omega_n as i64 * omega_n as i64;
        let ki_ts_num = ki * period_us as i64;

        // !!! OVERFLOW SAFETY CHECKS !!!

        // Max error is half a rotation in the PLL domain
        let max_error = Self::HALF_ROTATION as i64;

        // 1. Verify the PI integral accumulator won't panic
        assert!(
            ki_ts_num.checked_mul(max_error).is_some(),
            "i64 overflow: bandwidth or period is too high, ki_ts_num * error will panic"
        );

        // max_integral is the true velocity at MAX_SPEED_RPM in the PLL domain
        let max_integral =
            (MAX_SPEED_RPM as i64 * Self::FULL_ROTATION as i64) * overshoot_percent / (60 * 100);
        let max_proportional = (kp as i64).saturating_mul(max_error);
        let max_loop_output = max_integral.saturating_add(max_proportional);

        // 2. Verify the angle integrator won't panic
        assert!(
            max_loop_output.checked_mul(period_us as i64).is_some(),
            "i64 overflow: loop_output * period_us will panic at max speed and max error"
        );

        let max_velocity = max_integral >> Self::SHIFT;

        // 3. Verify the velocity estimation (i32) won't panic
        assert!(
            MAX_SPEED_RPM >= 0 && max_velocity <= i32::MAX as i64,
            "MAX_SPEED_RPM is too high (or negative, please set it positive)! Velocity in counts/sec exceeds i32 limits. Lower MAX_SPEED_RPM or reduce BITS."
        );

        // Create and return pll
        Self {
            angle_est: 0,
            velocity_est: 0,
            integral_term: 0,
            kp_num: kp,
            kp_den: 1,
            ki_ts_num,
            ki_ts_den: 1_000_000,
            period_us,
        }
    }

    /// Updates the observer with a new raw encoder angle
    #[inline(always)]
    pub fn update(&mut self, angle_read: IntAngle<BITS>) -> (IntAngle<BITS>, i32) {
        // Phase Detector: Calculate estimation error
        let target_angle = angle_read.raw_value() << Self::SHIFT;
        let mut error = target_angle - self.angle_est;

        // Fast wrap using 'if' instead of modulo or while.
        // At 100kHz, error will never exceed a single 2π rotation per tick.
        if error > Self::HALF_ROTATION {
            error -= Self::FULL_ROTATION;
        } else if error < -Self::HALF_ROTATION {
            error += Self::FULL_ROTATION;
        }

        // Loop Filter: PI controller calculates the estimated velocity
        let proportional = self.kp_num * error / self.kp_den;
        self.integral_term += (self.ki_ts_num as i64 * error as i64) / self.ki_ts_den as i64;

        let loop_output = proportional as i64 + self.integral_term;

        // Use arithmetic right shift (>>) which correctly preserves the sign bit in Rust.
        self.velocity_est = (self.integral_term >> Self::SHIFT) as i32;

        // Integrator: Calculate next estimated angle
        let angle_inc = (loop_output * self.period_us as i64) / 1_000_000;
        self.angle_est += angle_inc as i32;

        if self.angle_est > Self::FULL_ROTATION {
            self.angle_est -= Self::FULL_ROTATION;
        } else if self.angle_est < 0 {
            self.angle_est += Self::FULL_ROTATION;
        }

        log::debug!(
            "e = {}, p = {}, i = {} [ angle {} velocity {}]",
            error,
            proportional,
            self.integral_term,
            self.angle_est,
            self.velocity_est
        );

        (self.angle_est(), self.velocity_est)
    }

    pub fn angle_est(&self) -> IntAngle<BITS> {
        IntAngle::from_raw(self.angle_est >> Self::SHIFT)
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
