use crate::{Angle, angle_iter::AngleIterItem};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct RampSpeedAngleIter {
    ts: u64,
    period_us: u64,
    duration_us: u64,
    angle: f32,
    velocity_offset: f32,
    velocity_amplitude: f32,
    oscillation_period_us: u64,
}

impl RampSpeedAngleIter {
    pub fn new(
        start: Angle,
        velocity_offset: i32,
        velocity_amplitude: i32,
        oscillation_period_us: u64,
        period_us: u64,
        duration_us: u64,
    ) -> Self {
        Self {
            ts: 0,
            period_us,
            duration_us,
            angle: start.raw_value() as f32,
            velocity_offset: velocity_offset as f32,
            velocity_amplitude: velocity_amplitude as f32,
            oscillation_period_us,
        }
    }

    pub fn _current(&self) -> Angle {
        Angle::from_raw(self.angle as i32)
    }
}

impl Iterator for RampSpeedAngleIter {
    type Item = AngleIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.ts;

        if ts >= self.duration_us {
            return None;
        }

        // Calculate current velocity using a triangle wave
        // (piecewise constant acceleration oscillating between positive and negative)
        let current_velocity = if self.oscillation_period_us == 0 {
            self.velocity_offset
        } else {
            let t_mod = ts % self.oscillation_period_us;
            let phase = t_mod as f32 / self.oscillation_period_us as f32; // 0.0 to 1.0

            // Triangle wave logic starting at 0 offset, peaking at +amplitude (phase 0.25),
            // and troughing at -amplitude (phase 0.75)
            let v_wave = if phase < 0.25 {
                phase * 4.0 * self.velocity_amplitude
            } else if phase < 0.75 {
                self.velocity_amplitude - (phase - 0.25) * 4.0 * self.velocity_amplitude
            } else {
                -self.velocity_amplitude + (phase - 0.75) * 4.0 * self.velocity_amplitude
            };

            self.velocity_offset + v_wave
        };

        // Cache exact values for this iteration before advancing state
        let angle_out = self.angle;
        let velocity_out = current_velocity as i32;

        // Calculate step using Euler integration based on the instantaneous velocity
        let step = (current_velocity * self.period_us as f32)
            / (Duration::from_secs(1).as_micros() as f32);

        // Advance state
        self.angle += step;
        self.ts += self.period_us;

        Some(AngleIterItem {
            ts,
            angle: Angle::from_raw(angle_out as i32),
            velocity: velocity_out,
        })
    }
}
