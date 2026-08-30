use std::time::Duration;

use crate::Angle;

#[derive(Debug, Clone, Copy)]
pub struct AngleIterItem {
    pub ts: u64,
    pub angle: Angle,
    pub velocity: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct ConstantSpeedAngleIter {
    ts: u64,
    period_us: u64,
    duration_us: u64,
    angle: f32,
    step: f32,
    velocity: i32,
}

impl ConstantSpeedAngleIter {
    pub fn from_raw_step(
        start: Angle,
        step: f32,
        period_us: u64,
        duration_us: u64,
        velocity: i32,
    ) -> Self {
        Self {
            ts: 0,
            period_us,
            duration_us,
            angle: start.raw_value() as f32,
            step,
            velocity,
        }
    }

    pub fn from_int_angle_per_second(
        start: Angle,
        velocity: i32,
        period_us: u64,
        duration_us: u64,
    ) -> Self {
        let step =
            (velocity * period_us as i32) as f32 / (Duration::from_secs(1).as_micros() as f32);

        println!(
            "velocity = {}, period_us = {}, duration = {}, step = {}",
            velocity,
            period_us,
            Duration::from_secs(1).as_micros() as i32,
            step
        );

        Self::from_raw_step(start, step, period_us, duration_us, velocity)
    }

    pub fn _current(&self) -> Angle {
        Angle::from_raw(self.angle as i32)
    }
}

impl Iterator for ConstantSpeedAngleIter {
    type Item = AngleIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts;
        let angle = self.angle;

        self.angle = self.angle + self.step;
        self.ts = self.ts + self.period_us;

        if ts >= self.duration_us {
            None
        } else {
            Some(AngleIterItem {
                ts,
                angle: Angle::from_raw(angle as i32),
                velocity: self.velocity,
            })
        }
    }
}
