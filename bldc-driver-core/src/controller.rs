use bldc_driver_hal::BldcMotor;

use crate::KinematicEstReceiver;

/// Controller task loop, intended to be run in an embassy task
///
/// # Usage example
///
/// ```
/// #[embassy_executor::task]
/// async fn controller_task() {
///     controller_run<{BITS}>(/* ... */)
/// }
/// ```
pub async fn controller_run<const BITS: usize, M: BldcMotor<BITS>>(
    motor: M,
    mut receiver: KinematicEstReceiver<BITS>,
    _encoder_period_us: u32,
) -> ! {
    loop {
        motor.wake_to_set_pwm().await;

        let _kin_data = receiver.try_get();
    }
}
