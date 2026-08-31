use bldc_driver_core::{
    controller::controller_cycle,
    pll::{KinematicEst, PllObserver},
};
use bldc_driver_hal::IntAngle;
use std::time::Duration;

use crate::{
    angle_iter::constant_speed::ConstantSpeedAngleIter, csv::write_to_csv, motor::TestMotor,
};

mod angle_iter;
mod csv;
mod motor;

const BITS: usize = 14;
const ENCODER_FREQUENCY_HZ: u32 = 100_000;
const ENCODER_PERIOD_US: u64 =
    (Duration::from_secs(1).as_micros() as u64) / (ENCODER_FREQUENCY_HZ as u64);
type Angle = IntAngle<BITS>;

struct TestData {
    pub timestamp: u64,
    pub angle: Angle,
    pub velocity: i32,
    pub est_angle: Angle,
    pub est_velocity: i32,
    pub pwm_a: u32,
    pub pwm_b: u32,
    pub pwm_c: u32,
}

fn main() {
    env_logger::init();
    log::info!("Logger started.");

    const TEST_SPEED: i32 = 4 * Angle::A360.raw_value() / 1;

    let iter =
        ConstantSpeedAngleIter::new(Angle::from_raw(10), TEST_SPEED, ENCODER_PERIOD_US, 10_000);

    let mut pll = PllObserver::<BITS, 2200>::new(ENCODER_PERIOD_US as i32, 1000);

    let data: Vec<TestData> = iter
        .map(|item| {
            let (est_angle, est_velocity) = pll.update(item.angle);
            let kin_data = KinematicEst {
                angle: est_angle,
                velocity: est_velocity,
                timestamp: item.ts,
            };
            let (pwm_a, pwm_b, pwm_c) = controller_cycle::<BITS, TestMotor>(kin_data, 10);

            TestData {
                timestamp: item.ts,
                angle: item.angle,
                velocity: item.velocity,
                est_angle,
                est_velocity,
                pwm_a,
                pwm_b,
                pwm_c,
            }
        })
        .collect();

    let len = data.len();

    match write_to_csv(data, "output.csv") {
        Ok(_) => println!("Successfully wrote {} rows to output.csv", len),
        Err(e) => eprintln!("Failed to write CSV: {}", e),
    }
}
