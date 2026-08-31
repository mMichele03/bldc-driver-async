use std::{
    fs::File,
    io::{BufWriter, Error, Write},
};

use crate::{Angle, TestData};

fn _wrap_angle_around_0(angle: Angle) -> i32 {
    if angle.raw_value() > Angle::A180.raw_value() {
        angle.raw_value() - Angle::A360.raw_value()
    } else if angle.raw_value() < -Angle::A180.raw_value() {
        angle.raw_value() + Angle::A360.raw_value()
    } else {
        angle.raw_value()
    }
}

/// Writes a slice (or Vec) of TestKinEstData to a CSV file.
pub fn write_to_csv(data: Vec<TestData>, filename: &str) -> Result<(), Error> {
    let file = File::create(filename)?;
    let mut wtr = BufWriter::new(file);

    // Write the CSV header
    writeln!(
        wtr,
        "timestamp,angle,angle_deg,est_angle,est_angle_deg,velocity,velocity_deg_s,est_velocity,est_velocity_deg_s,pwm_a,pwm_b,pwm_c"
    )?;

    // Write each row
    for row in data {
        writeln!(
            wtr,
            "{},{},{:.2},{},{:.2},{},{:.2},{},{:.2},{},{},{}",
            row.timestamp,
            row.angle.raw_value(),
            row.angle.raw_value() as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.est_angle.raw_value(),
            row.est_angle.raw_value() as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.velocity,
            row.velocity as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.est_velocity,
            row.est_velocity as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.pwm_a,
            row.pwm_b,
            row.pwm_c,
        )?;
    }

    // Ensure all data is flushed to disk
    wtr.flush()?;

    Ok(())
}
