use bldc_driver_core::pll::PllObserver;
use bldc_driver_hal::IntAngle;
use std::time::Duration;

use crate::{angle_iter::ConstantSpeedAngleIter, csv::write_to_csv};

mod angle_iter;
mod csv;

const BITS: usize = 14;
const ENCODER_FREQUENCY_HZ: u32 = 100_000;
const ENCODER_PERIOD_US: u64 =
    (Duration::from_secs(1).as_micros() as u64) / (ENCODER_FREQUENCY_HZ as u64);
type Angle = IntAngle<BITS>;

struct TestKinEstData {
    pub timestamp: u64,
    pub angle: Angle,
    pub est_angle: Angle,
    pub est_velocity: i32,
}

fn main() {
    env_logger::init();
    log::info!("Logger started.");

    const TEST_SPEED: i32 = Angle::A360.raw_value() / 1;

    let iter = ConstantSpeedAngleIter::from_int_angle_per_second(
        Angle::from_raw(0),
        TEST_SPEED,
        ENCODER_PERIOD_US,
        7_000,
    );

    let mut pll = PllObserver::<BITS, 2200>::new(ENCODER_PERIOD_US as i32, 1000);

    let data: Vec<TestKinEstData> = iter
        .map(|(timestamp, angle)| {
            let (est_angle, est_velocity) = pll.update(angle);
            TestKinEstData {
                timestamp,
                angle,
                est_angle,
                est_velocity,
            }
        })
        .collect();

    let len = data.len();

    match write_to_csv(data, "output.csv") {
        Ok(_) => println!("Successfully wrote {} rows to output.csv", len),
        Err(e) => eprintln!("Failed to write CSV: {}", e),
    }
}
