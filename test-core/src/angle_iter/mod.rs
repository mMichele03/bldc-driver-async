use crate::Angle;

pub mod constant_speed;
pub mod ramp_speed;

#[derive(Debug, Clone, Copy)]
pub struct AngleIterItem {
    pub ts: u64,
    pub angle: Angle,
    pub velocity: i32,
}
