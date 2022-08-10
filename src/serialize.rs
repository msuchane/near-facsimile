use color_eyre::Result;

use crate::Cli;
use crate::Comparison;

/// Serialize the resulting comparisons as a CSV table.
pub fn serialize(mut comparisons: Vec<Comparison>, options: &Cli) -> Result<()> {
    // Prepare to write to the CSV file.
    let mut wtr = csv::Writer::from_path(&options.csv_path)?;

    // The CSV header:
    wtr.write_record(&["% similar", "File 1", "File 2"])?;

    // Sort from highest to lowest. You can't sort f64 values, so convert them to u32
    // with a precision of percentage with a single decimal place, then subtract from 1000.
    comparisons
        .sort_by_key(|comparison| 1000 - (comparison.similarity_pct.0 * 10.0).round() as i32);

    // Each comparison entry writes a row in the CSV table.
    for comparison in &comparisons {
        wtr.write_record(&[
            format!("{:.1}", comparison.similarity_pct.0),
            comparison.path1.display().to_string(),
            comparison.path2.display().to_string(),
        ])?;
    }

    // Flush the CSV writer buffer.
    wtr.flush()?;

    Ok(())
}
