use crate::engine::state_store::TransferRecord;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Write all completed transfer records to a CSV file at `output_path`.
///
/// The output directory is created if it does not already exist.
///
/// # Arguments
///
/// * `records` - Slice of [`TransferRecord`]s to serialize (one row per record).
/// * `output_path` - Filesystem path for the CSV output file.
///
/// # Returns
///
/// `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if the output directory cannot be created, the file cannot be
/// opened for writing, or an I/O error occurs while writing.
pub fn generate_csv_report(records: &[TransferRecord], output_path: &str) -> Result<()> {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory for report: {:?}", parent))?;
        }
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create report file at: {}", output_path))?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "Source Repo,Path,Target Repo,SHA256,Size (Bytes),Completed At"
    )?;

    for record in records {
        writeln!(
            writer,
            "{},{},{},{},{},{}",
            record.source_repo,
            record.path,
            record.target_repo,
            record.sha256,
            record.size,
            record.completed_at
        )?;
    }

    writer.flush()?;
    Ok(())
}
