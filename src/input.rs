//! This module handles parsing of input clustering data.

use std::path::Path;

use csv::StringRecord;

use crate::data::{CellSample, ResolutionData};

/// Tries to parse the specified CSV file as [`ResolutionData`]s.
///
/// # Parameters
///
/// * `csv_path` - the path to the CSV file
pub fn parse_input_csv<T: AsRef<Path>>(
    csv_path: T,
) -> Result<Vec<ResolutionData>, Box<dyn std::error::Error>> {
    let mut csv_reader = csv::ReaderBuilder::default()
        .delimiter(b',')
        .flexible(false)
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path(csv_path.as_ref())?;

    let mut resolutions = Vec::new();
    for record_result in csv_reader.records() {
        let resolution_data = row_to_resolution_data(record_result?)?;
        resolutions.push(resolution_data);
    }
    Ok(resolutions)
}

/// Parses a CSV data row as [`ResolutionData`].
///
/// # Parameters
///
/// * `row` - the row to parse
fn row_to_resolution_data(row: StringRecord) -> Result<ResolutionData, String> {
    // Parses resolution.
    let resolution: f64 = row
        .get(0)
        .ok_or("The first column must contain resolution data, but is empty.".to_string())?
        .parse()
        .map_err(|error| {
            format!("Parsing the resolution data in the first row failed with error: {}", error)
        })?;
    // Parses cell clustering data.
    let mut cells: Vec<CellSample> = Vec::new();
    for column_index in 1..row.len() {
        let cluster: usize = row[column_index].parse().map_err(|error| {
            format!(
                "Parsing the cell cluster data {} of cell {} failed with error: {}",
                &row[column_index], column_index, error
            )
        })?;
        cells.push(CellSample::new(column_index, cluster));
    }
    // Returns resolution data.
    if cells.is_empty() {
        Err(format!("No cell data present for resolution {}.", resolution))
    } else {
        Ok(ResolutionData::new(resolution, &cells))
    }
}
