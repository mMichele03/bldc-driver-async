use bldc_driver_core::{
    controller::{controller_cycle, estimate_control_angle, foc_algorithm, inverse_park_clarke},
    pll::{KinematicEst, PllObserver},
};
use bldc_driver_hal::{BldcMotor, IntAngle};
use std::time::Duration;

use crate::{
    angle_iter::{
        constant_speed::ConstantSpeedAngleIter, sinusoidal_speed::SinusoidalSpeedAngleIter,
    },
    csv::write_to_csv,
    motor::TestMotor,
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
    pub control_angle: Angle,
    pub q_axis_voltage_uv: i32,
    pub d_axis_voltage_uv: i32,
    pub control_angle_sin: f32,
    pub v_a: f32,
    pub v_b: f32,
    pub v_c: f32,
}

impl Default for TestData {
    fn default() -> Self {
        Self {
            timestamp: Default::default(),
            angle: Default::default(),
            velocity: Default::default(),
            est_angle: Default::default(),
            est_velocity: Default::default(),
            pwm_a: Default::default(),
            pwm_b: Default::default(),
            pwm_c: Default::default(),
            control_angle: Default::default(),
            q_axis_voltage_uv: Default::default(),
            d_axis_voltage_uv: Default::default(),
            control_angle_sin: Default::default(),
            v_a: Default::default(),
            v_b: Default::default(),
            v_c: Default::default(),
        }
    }
}

fn main() {
    env_logger::init();
    log::info!("Logger started.");

    const TEST_SPEED: i32 = 10 * Angle::A360.raw_value() / 1;

    const TEST_TARGET_TORQUE: i32 = 1_000_000; // µNm

    // let iter = SinusoidalSpeedAngleIter::new(
    //     Angle::from_raw(10),
    //     0,
    //     TEST_SPEED,
    //     5_000,
    //     ENCODER_PERIOD_US,
    //     10_000,
    // );
    let iter =
        ConstantSpeedAngleIter::new(Angle::from_raw(10), TEST_SPEED, ENCODER_PERIOD_US, 10_000);

    let mut pll =
        PllObserver::<BITS, 2200>::new(Angle::from_raw(10), ENCODER_PERIOD_US as i32, 1000);

    let data: Vec<TestData> = iter
        .map(|item| {
            let (est_angle, est_velocity) = pll.update(item.angle);
            let kin_data = KinematicEst {
                angle: est_angle,
                velocity: est_velocity,
                timestamp: item.ts,
            };
            let (pwm_a, pwm_b, pwm_c) =
                controller_cycle::<BITS, TestMotor>(kin_data, TEST_TARGET_TORQUE);

            // // // control loop start
            // let control_angle = estimate_control_angle(
            //     kin_data.angle,
            //     kin_data.velocity,
            //     TestMotor::PWM_CONTROL_LAG_US,
            // );

            // let target_q_axis_current_ua =
            //     (TEST_TARGET_TORQUE * 1_000) / TestMotor::TORQUE_COEFFICIENT;

            // let (q_axis_voltage_uv, d_axis_voltage_uv) =
            //     foc_algorithm::<BITS, TestMotor>(target_q_axis_current_ua, 0, kin_data.velocity);

            // let electrical_angle = control_angle * TestMotor::POLE_PAIRS;

            // let (pwm_a, pwm_b, pwm_c) = inverse_park_clarke(
            //     control_angle * TestMotor::POLE_PAIRS,
            //     q_axis_voltage_uv,
            //     d_axis_voltage_uv,
            //     TestMotor::PWM_TOP,
            //     TestMotor::MAX_VOLTAGE,
            // );

            // // // inverse_park_clarke start
            // // let pwm_top = TestMotor::PWM_TOP;
            // // let max_voltage_uv = TestMotor::MAX_VOLTAGE;

            // // // 1. Compute trigonometry with lookup table on integers
            // // let sin_theta = electrical_angle.sin().scaled(255) as f32 / 255.0;
            // // let cos_theta = electrical_angle.cos().scaled(255) as f32 / 255.0;

            // // let v_q = q_axis_voltage_uv as f32;
            // // let v_d = d_axis_voltage_uv as f32;

            // // // 2. Inverse Park Transform (Rotating to Stationary frame)
            // // let v_alpha = v_d * cos_theta - v_q * sin_theta;
            // // let v_beta = v_d * sin_theta + v_q * cos_theta;

            // // let sqrt_3_over_2 = 0.8660254; // sqrt(3)/2

            // // // 3. Inverse Clarke Transform (Stationary to 3-Phase frame)
            // // let v_a = v_alpha;
            // // let v_b = -0.5 * v_alpha + sqrt_3_over_2 * v_beta;
            // // let v_c = -0.5 * v_alpha - sqrt_3_over_2 * v_beta;

            // // // 4. SVPWM Min-Max Injection
            // // // Centers the vectors to fully utilize the available PWM headroom.
            // // let v_min = v_a.min(v_b).min(v_c);
            // // let v_max = v_a.max(v_b).max(v_c);
            // // let v_offset = -(v_min + v_max) / 2.0;

            // // // 5. Scale voltages to PWM duty cycle
            // // // 50% duty cycle yields 0V differential.
            // // // pwm_top yields +max_voltage.
            // // // 0 yields -max_voltage.
            // // let half_pwm = (pwm_top as f32) / 2.0;
            // // let volts_to_pwm_ratio = (pwm_top as f32) / (max_voltage_uv as f32);

            // // // Apply offset, scale by voltage-to-PWM ratio, shift to absolute PWM range, and clamp
            // // let pwm_a = ((v_a + v_offset) * volts_to_pwm_ratio + half_pwm)
            // //     .clamp(0.0, pwm_top as f32) as u32;
            // // let pwm_b = ((v_b + v_offset) * volts_to_pwm_ratio + half_pwm)
            // //     .clamp(0.0, pwm_top as f32) as u32;
            // // let pwm_c = ((v_c + v_offset) * volts_to_pwm_ratio + half_pwm)
            // //     .clamp(0.0, pwm_top as f32) as u32;

            // // // inverse_park_clarke end
            // // control loop end

            TestData {
                timestamp: item.ts,
                angle: item.angle,
                velocity: item.velocity,
                est_angle,
                est_velocity,
                pwm_a,
                pwm_b,
                pwm_c,
                // control_angle: electrical_angle,
                // q_axis_voltage_uv,
                // d_axis_voltage_uv,
                // v_a,
                // v_b,
                // v_c,
                ..Default::default()
            }
        })
        .collect();

    let len = data.len();

    match write_to_csv(data, "output.csv") {
        Ok(_) => println!("Successfully wrote {} rows to output.csv", len),
        Err(e) => eprintln!("Failed to write CSV: {}", e),
    }
}
