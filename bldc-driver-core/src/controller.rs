use bldc_driver_hal::{BldcMotor, IntAngle};

use crate::{KinematicEstReceiver, TorqueReceiver, pll::KinematicEst};

#[inline(always)]
pub fn estimate_control_angle<const BITS: usize>(
    angle: IntAngle<BITS>,
    velocity: i32,
    control_lag_us: u64,
) -> IntAngle<BITS> {
    // Using i64 prevents overflow during the multiplication.
    let delta_angle = (velocity as i64 * control_lag_us as i64) / 1_000_000;

    IntAngle::from_raw(angle.raw_value().wrapping_add(delta_angle as i32))
}

#[inline(always)]
pub const fn frac_mul_velocity_to_rad_s<const BITS: usize>(
    velocity: i32,
    num: i64,
    den: i64,
) -> i64 {
    const TAU_INT_NUM: i64 = 6_283;
    const TAU_INT_DEN: i64 = 1_000;

    (num * velocity as i64 * TAU_INT_NUM)
        / (den * IntAngle::<BITS>::A360.raw_value() as i64 * TAU_INT_DEN)
}

#[inline(always)]
pub fn foc_algorithm<const BITS: usize, M: BldcMotor<BITS>>(
    q_axis_current_ma: i32,
    d_axis_current_ma: i32,
    velocity: i32,
) -> (i32, i32) {
    // Feed-forwards
    const I_Q_FF: i32 = 0;
    const I_D_FF: i32 = 0;
    const U_Q_FF: i32 = 0;
    const U_D_FF: i32 = 0;

    // ==========================================
    // Q-Axis Path
    // ==========================================
    // 1. Current limit & Feed-forward addition
    let i_q_limited = q_axis_current_ma.clamp(-M::MAX_CURRENT, M::MAX_CURRENT);
    let i_q_total = i_q_limited + I_Q_FF;

    // 2. Phase resistance multiplication
    let u_q_res = i_q_total * M::PHASE_RESISTANCE / 1_000;

    // // 3. Estimated BEMF voltage (Ke * filtered_velocity)
    // let u_bemf =
    //     frac_mul_velocity_to_rad_s::<BITS>(velocity, M::BACK_EMF_COEFFICIENT as i64, 1) as i32;

    // // 4. Sum resistance drop and BEMF
    // let u_q_interp = u_q_res + u_bemf;
    let u_q_interp = u_q_res;

    // 5. Voltage limit & Feed-forward addition
    let u_q_limited = u_q_interp.clamp(-M::MAX_VOLTAGE, M::MAX_VOLTAGE);
    let u_q = u_q_limited + U_Q_FF;

    // ==========================================
    // D-Axis Path & Cross-Coupling / Lag Voltage
    // ==========================================
    // 1. Current limit & Feed-forward addition
    let i_d_limited = d_axis_current_ma.clamp(-M::MAX_CURRENT, M::MAX_CURRENT);
    let i_d_total = i_d_limited + I_D_FF;

    // 2. Phase resistance multiplication & Voltage limit
    let u_d_res = i_d_total * M::PHASE_RESISTANCE / 1_000;
    let u_d_limited = u_d_res.clamp(-M::MAX_VOLTAGE, M::MAX_VOLTAGE);

    // // 3. Estimated lag voltage: u_lag = (i_q_total * L_q) * (filtered_velocity * n_pp)
    // let electrical_velocity = velocity * M::POLE_PAIRS;
    // let flux_linkage_q = i_q_total * M::Q_AXIS_INDUCTANCE / 1_000_000;
    // let u_lag =
    //     frac_mul_velocity_to_rad_s::<BITS>(electrical_velocity, flux_linkage_q as i64, 1) as i32;

    // // 4. Subtract lag voltage and add d-axis voltage feed-forward
    // let u_d = u_d_limited - u_lag + U_D_FF;
    let u_d = u_d_limited + U_D_FF;

    (u_q, u_d)
}

#[inline(always)]
pub fn inverse_park_clarke<const BITS: usize>(
    electrical_angle: IntAngle<BITS>,
    q_axis_voltage_mv: i32,
    d_axis_voltage_mv: i32,
    pwm_top: u32,
    max_voltage_uv: i32,
) -> (u32, u32, u32) {
    // 1. Compute trigonometry with lookup table on integers
    let sin_theta = electrical_angle.sin().scaled(255) as f32 / 255.0;
    let cos_theta = electrical_angle.cos().scaled(255) as f32 / 255.0;

    let v_q = q_axis_voltage_mv as f32;
    let v_d = d_axis_voltage_mv as f32;

    // 2. Inverse Park Transform (Rotating to Stationary frame)
    let v_alpha = v_d * cos_theta - v_q * sin_theta;
    let v_beta = v_d * sin_theta + v_q * cos_theta;

    let sqrt_3_over_2 = 0.8660254; // sqrt(3)/2

    // 3. Inverse Clarke Transform (Stationary to 3-Phase frame)
    let v_a = v_alpha;
    let v_b = -0.5 * v_alpha + sqrt_3_over_2 * v_beta;
    let v_c = -0.5 * v_alpha - sqrt_3_over_2 * v_beta;

    // 4. SVPWM Min-Max Injection
    // Centers the vectors to fully utilize the available PWM headroom.
    let v_min = v_a.min(v_b).min(v_c);
    let v_max = v_a.max(v_b).max(v_c);
    let v_offset = -(v_min + v_max) / 2.0;

    // 5. Scale voltages to PWM duty cycle
    // 50% duty cycle yields 0V differential.
    // pwm_top yields +max_voltage.
    // 0 yields -max_voltage.
    let half_pwm = (pwm_top as f32) / 2.0;
    let volts_to_pwm_ratio = (pwm_top as f32) / (max_voltage_uv as f32);

    // Apply offset, scale by voltage-to-PWM ratio, shift to absolute PWM range, and clamp
    let pwm_a =
        ((v_a + v_offset) * volts_to_pwm_ratio + half_pwm).clamp(0.0, pwm_top as f32) as u32;
    let pwm_b =
        ((v_b + v_offset) * volts_to_pwm_ratio + half_pwm).clamp(0.0, pwm_top as f32) as u32;
    let pwm_c =
        ((v_c + v_offset) * volts_to_pwm_ratio + half_pwm).clamp(0.0, pwm_top as f32) as u32;

    (pwm_a, pwm_b, pwm_c)
}

#[inline(always)]
pub fn controller_cycle<const BITS: usize, M: BldcMotor<BITS>>(
    kin_data: KinematicEst<BITS>,
    target_torque: i32,
) -> (u32, u32, u32) {
    let control_angle =
        estimate_control_angle(kin_data.angle, kin_data.velocity, M::PWM_CONTROL_LAG_US);

    let target_q_axis_current_ma = target_torque / M::TORQUE_COEFFICIENT;

    let (q_axis_voltage_mv, d_axis_voltage_mv) =
        foc_algorithm::<BITS, M>(target_q_axis_current_ma, 0, kin_data.velocity);

    inverse_park_clarke(
        control_angle * M::POLE_PAIRS,
        q_axis_voltage_mv,
        d_axis_voltage_mv,
        M::PWM_TOP,
        M::MAX_VOLTAGE,
    )
}

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
    mut motor: M,
    mut kin_est_rx: KinematicEstReceiver<BITS>,
    mut torque_rx: TorqueReceiver<BITS>,
) -> ! {
    loop {
        motor.wake_to_set_pwm().await;

        if let Some(kin_data) = kin_est_rx.try_get()
            && let Some(target_torque) = torque_rx.try_get()
        {
            let (pwm_a, pwm_b, pwm_c) = controller_cycle::<BITS, M>(kin_data, target_torque);

            motor.set_pwm(pwm_a, pwm_b, pwm_c);
        }
    }
}
