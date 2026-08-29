use std::{
    fs::File,
    io::{BufWriter, Error, Write},
};

use crate::{Angle, TestKinEstData};

/// Writes a slice (or Vec) of TestKinEstData to a CSV file.
pub fn write_to_csv(data: Vec<TestKinEstData>, filename: &str) -> Result<(), Error> {
    let file = File::create(filename)?;
    let mut wtr = BufWriter::new(file);

    // Write the CSV header
    writeln!(
        wtr,
        "timestamp,angle,angle_deg,est_angle,est_angle_deg,est_velocity"
    )?;

    // Write each row
    for row in data {
        writeln!(
            wtr,
            "{},{},{:.2},{},{:.2},{}",
            row.timestamp,
            row.angle.raw_value(),
            row.angle.raw_value() as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.est_angle.raw_value(),
            row.est_angle.raw_value() as f32 * 360.0 / Angle::MAX_VALUE as f32,
            row.est_velocity
        )?;
    }

    // Ensure all data is flushed to disk
    wtr.flush()?;

    Ok(())
}
