use std::time::Duration;

use crate::{Angle, angle_iter::AngleIterItem};

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
    pub fn new(start: Angle, velocity: i32, period_us: u64, duration_us: u64) -> Self {
        let step =
            (velocity * period_us as i32) as f32 / (Duration::from_secs(1).as_micros() as f32);

        Self {
            ts: 0,
            period_us,
            duration_us,
            angle: start.raw_value() as f32,
            step,
            velocity,
        }
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
