#![no_std]

mod angle;

use angle::IntAngle;
// use embassy_sync::watch;

pub trait Encoder {
    const BITS: usize;

    fn read_value() -> IntAngle<14>;

    fn read_stream() -> (u32,);
}
