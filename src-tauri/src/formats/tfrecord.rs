use crate::formats::unified::UnifiedDataset;
use crate::types::DetectedFormat;
use anyhow::{Context, Result};
use std::path::Path;

pub fn write_tfrecord(dataset: UnifiedDataset, output_dir: &Path) -> Result<()> {
    // TFRecord writing requires TensorFlow protobuf definitions
    // This is a placeholder implementation
    std::fs::create_dir_all(output_dir)?;

    let readme = format!(
        r#"# TFRecord Export

This directory contains TFRecord files exported from Dataset Manager.

Note: Full TFRecord support requires TensorFlow protobuf definitions.
This is a placeholder - implement using tensorflow crate or Python script.

Dataset info:
- Images: {}
- Categories: {}
- Generated: {}
"#,
        dataset.images.len(),
        dataset.categories.len(),
        chrono::Utc::now().to_rfc3339()
    );

    std::fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

pub fn detect_tfrecord_format(root: &Path) -> Result<DetectedFormat> {
    let record_files: Vec<String> = std::fs::read_dir(root)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "tfrecord" || ext == "record")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    if record_files.is_empty() {
        return Ok(DetectedFormat::Unknown);
    }

    Ok(DetectedFormat::TFRecord { record_files })
}