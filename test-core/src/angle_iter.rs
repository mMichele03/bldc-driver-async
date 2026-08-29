use std::time::Duration;

use crate::Angle;

#[derive(Debug, Clone, Copy)]
pub struct ConstantSpeedAngleIter {
    ts: u64,
    period_us: u64,
    duration_us: u64,
    angle: Angle,
    step: i32,
}

impl ConstantSpeedAngleIter {
    pub fn from_raw_step(start: Angle, step: i32, period_us: u64, duration_us: u64) -> Self {
        Self {
            ts: 0,
            period_us,
            duration_us,
            angle: start,
            step,
        }
    }

    pub fn from_int_angle_per_second(
        start: Angle,
        velocity: i32,
        period_us: u64,
        duration_us: u64,
    ) -> Self {
        let step = (velocity * period_us as i32) / (Duration::from_secs(1).as_micros() as i32);
        Self::from_raw_step(start, step, period_us, duration_us)
    }

    pub fn _current(&self) -> Angle {
        self.angle
    }
}

impl Iterator for ConstantSpeedAngleIter {
    type Item = (u64, Angle);

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts;
        let angle = self.angle;
        self.angle = (self.angle + Angle::from_raw(self.step)).normalized();
        self.ts = self.ts + self.period_us;

        if ts >= self.duration_us {
            None
        } else {
            Some((ts, angle))
        }
    }
}
