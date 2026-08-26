use super::encoder::ENCODER_BITS;
use bldc_driver_hal::BldcMotor;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_6, PIN_7, PIN_8, PIN_9, PWM_SLICE3, PWM_SLICE4};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use fixed::FixedU16;
use fixed::types::extra::U4;

pub struct RpBldcMotor {
    pwm_ab: Pwm<'static>,
    pwm_c: Pwm<'static>,
    _en: Output<'static>,
}

impl RpBldcMotor {
    pub const PWM_TOP: u32 = 1250;

    pub fn new(
        pin_6: Peri<'static, PIN_6>,
        pin_7: Peri<'static, PIN_7>,
        pin_8: Peri<'static, PIN_8>,
        pin_9: Peri<'static, PIN_9>,
        slice_3: Peri<'static, PWM_SLICE3>,
        slice_4: Peri<'static, PWM_SLICE4>,
    ) -> Self {
        let _en = Output::new(pin_9, Level::High);

        let mut config = Config::default();
        config.phase_correct = true;
        config.top = Self::PWM_TOP as u16;
        config.divider = FixedU16::<U4>::from_num(1.0);

        let pwm_ab = Pwm::new_output_ab(slice_3, pin_6, pin_7, config.clone());
        let pwm_c = Pwm::new_output_a(slice_4, pin_8, config);

        Self { pwm_ab, pwm_c, _en }
    }
}

impl BldcMotor<ENCODER_BITS> for RpBldcMotor {
    const PWM_TOP: u32 = RpBldcMotor::PWM_TOP;

    const PWM_FREQ: u32 = 60000;

    fn set_pwm(&mut self, a: u32, b: u32, c: u32) {
        let (pwm_a, pwm_b) = self.pwm_ab.split_by_ref();
        let mut pwm_a = pwm_a.unwrap();
        let mut pwm_b = pwm_b.unwrap();

        pwm_a
            .set_duty_cycle(a.clamp(0, Self::PWM_TOP) as u16)
            .unwrap();
        pwm_b
            .set_duty_cycle(b.clamp(0, Self::PWM_TOP) as u16)
            .unwrap();
        self.pwm_c
            .set_duty_cycle(c.clamp(0, Self::PWM_TOP) as u16)
            .unwrap();

        log::debug!("a b c : {a} {b} {c}");
    }
}
