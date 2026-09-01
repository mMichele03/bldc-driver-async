use super::encoder::ENCODER_BITS;
use bldc_driver_hal::BldcMotor;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt::InterruptExt;
use embassy_rp::peripherals::{PIN_6, PIN_7, PIN_8, PIN_9, PWM_SLICE3, PWM_SLICE4};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_rp::{Peri, interrupt, pac};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use fixed::FixedU16;
use fixed::types::extra::U4;

static PWM_SYNC: Signal<CriticalSectionRawMutex, ()> = Signal::new();

// The RP2350 has two PWM IRQs. We are using IRQ 0.
#[interrupt]
#[unsafe(link_section = ".data")]
fn PWM_IRQ_WRAP_0() {
    // let ctr = pac::PWM.ch(3).ctr().read().0;

    // Call set_ch0 to clear the interrupt flag for our specific slice (here Slice 3)
    pac::PWM.intr().write(|w| w.set_ch3(true));

    // Bus flush (Crucial for RP2350 Cortex-M33):
    // Read the register back to stall the CPU until the write propagates.
    // This entirely prevents the spurious "second" interrupt call.
    let _ = pac::PWM.intr().read();

    PWM_SYNC.signal(());
}

pub struct RpBldcMotor {
    pwm_ab: Pwm<'static>,
    pwm_c: Pwm<'static>,
    _en: Output<'static>,
}

impl RpBldcMotor {
    pub const PWM_TOP: u32 = 2999;
    pub const PWM_FREQ: u32 = 150_000_000 / ((Self::PWM_TOP + 1) * 2);

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

        // Enable the hardware interrupt for our specific slice (here Slice 3)
        pac::PWM.irq0_inte().modify(|w| w.set_ch3(true));

        // Set to the highest priority (P3 in embassy-rp for RP-series chips)
        // This ensures your PWM ISR preempts other tasks and critical sections.
        interrupt::PWM_IRQ_WRAP_0.set_priority(interrupt::Priority::P3);

        // Enable the IRQ in the NVIC
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::PWM_IRQ_WRAP_0);
        }

        Self { pwm_ab, pwm_c, _en }
    }
}

impl BldcMotor<ENCODER_BITS> for RpBldcMotor {
    const PHASE_RESISTANCE: i32 = 5_600;
    const Q_AXIS_INDUCTANCE: i32 = 4_600;
    const POLE_PAIRS: i32 = 7;
    const BACK_EMF_COEFFICIENT: i32 = 47;
    const TORQUE_COEFFICIENT: i32 = 70;
    const MAX_VOLTAGE: i32 = 12_000;
    const MAX_CURRENT: i32 = 2_000;

    const PWM_TOP: u32 = RpBldcMotor::PWM_TOP;
    const PWM_FREQ: u32 = RpBldcMotor::PWM_FREQ;

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

    fn wake_to_set_pwm(&self) -> impl Future<Output = ()> + Send {
        PWM_SYNC.wait()
    }
}
