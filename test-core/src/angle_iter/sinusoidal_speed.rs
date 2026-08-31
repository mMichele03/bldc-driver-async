use crate::{Angle, angle_iter::AngleIterItem};
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SinusoidalSpeedAngleIter {
    ts: u64,
    period_us: u64,
    duration_us: u64,
    angle: f32,
    velocity_offset: f32,
    velocity_amplitude: f32,
    sinusoid_period_us: u64,
}

impl SinusoidalSpeedAngleIter {
    pub fn new(
        start: Angle,
        velocity_offset: i32,
        velocity_amplitude: i32,
        sinusoid_period_us: u64,
        period_us: u64,
        duration_us: u64,
    ) -> Self {
        assert!(
            sinusoid_period_us > 0,
            "Sinusoid period must be greater than 0"
        );

        Self {
            ts: 0,
            period_us,
            duration_us,
            angle: start.raw_value() as f32,
            velocity_offset: velocity_offset as f32,
            velocity_amplitude: velocity_amplitude as f32,
            sinusoid_period_us,
        }
    }

    pub fn _current(&self) -> Angle {
        Angle::from_raw(self.angle as i32)
    }
}

impl Iterator for SinusoidalSpeedAngleIter {
    type Item = AngleIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts;
        let angle = self.angle;

        if ts >= self.duration_us {
            return None;
        }

        // 1. Calculate current velocity using: v(t) = offset + amplitude * sin(2 * PI * t / T)
        let phase = (ts as f32) / (self.sinusoid_period_us as f32) * 2.0 * PI;
        let current_velocity_f32 = self.velocity_offset + self.velocity_amplitude * phase.sin();
        let current_velocity = current_velocity_f32.round() as i32;

        // 2. Calculate how much the angle changes during this specific period_us
        let step = (current_velocity_f32 * self.period_us as f32)
            / (Duration::from_secs(1).as_micros() as f32);

        // 3. Update internal state for the next iteration
        self.angle += step;
        self.ts += self.period_us;

        Some(AngleIterItem {
            ts,
            angle: Angle::from_raw(angle as i32),
            velocity: current_velocity,
        })
    }
}
