use crate::BITS;
use bldc_driver_hal::BldcMotor;

pub struct TestMotor {}

impl BldcMotor<BITS> for TestMotor {
    const PHASE_RESISTANCE: i32 = 5600;
    const Q_AXIS_INDUCTANCE: i32 = 4600;
    const POLE_PAIRS: i32 = 7;
    const BACK_EMF_COEFFICIENT: i32 = 47_000_000;
    const TORQUE_COEFFICIENT: i32 = 70;
    const MAX_VOLTAGE: i32 = 12_000;
    const MAX_CURRENT: i32 = 2_000;

    const PWM_TOP: u32 = 1250;
    const PWM_FREQ: u32 = 60000;

    fn wake_to_set_pwm(&self) -> impl Future<Output = ()> + Send {
        async { unimplemented!() }
    }

    fn set_pwm(&mut self, _a: u32, _b: u32, _c: u32) {
        unimplemented!()
    }
}
