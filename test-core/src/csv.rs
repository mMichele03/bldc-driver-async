use std::{
    fs::File,
    io::{BufWriter, Error, Write},
};

use crate::TestKinEstData;

/// Writes a slice (or Vec) of TestKinEstData to a CSV file.
pub fn write_to_csv(data: Vec<TestKinEstData>, filename: &str) -> Result<(), Error> {
    let file = File::create(filename)?;
    let mut wtr = BufWriter::new(file);

    // Write the CSV header
    writeln!(wtr, "timestamp,angle,est_angle,est_velocity")?;

    // Write each row
    for row in data {
        writeln!(
            wtr,
            "{},{},{},{}",
            row.timestamp,
            row.angle.raw_value(),     // Update this to match your HAL
            row.est_angle.raw_value(), // Update this to match your HAL
            row.est_velocity
        )?;
    }

    // Ensure all data is flushed to disk
    wtr.flush()?;

    Ok(())
}
