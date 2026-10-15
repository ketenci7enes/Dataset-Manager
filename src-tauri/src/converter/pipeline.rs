use crate::formats::{coco, label_studio, tfrecord, voc, yolo};
use crate::formats::unified::UnifiedDataset;
use crate::types::{DetectedFormat, ConvertOptions, LSExportConfig};
use anyhow::{Context, Result};
use std::path::Path;
use std::fs;

#[derive(Debug)]
pub struct ConvertResult {
    pub output_path: String,
    pub format: DetectedFormat,
    pub affected_files: usize,
}

pub fn convert_dataset(
    dataset: UnifiedDataset,
    target_format: DetectedFormat,
    output_dir: String,
    options: ConvertOptions,
) -> Result<ConvertResult> {
    let output_path = Path::new(&output_dir);
    fs::create_dir_all(output_path)?;

    let output_dir_clone = output_dir.clone();
    let image_count = dataset.images.len();
    
    let result = match &target_format {
        DetectedFormat::YOLO { .. } => {
            yolo::write_yolo_dataset(&dataset, output_path, options.clone())?;
            ConvertResult { output_path: output_dir_clone, format: target_format.clone(), affected_files: image_count }
        }
        DetectedFormat::COCO { .. } => {
            coco::write_coco_dataset(&dataset, output_path)?;
            ConvertResult { output_path: output_dir_clone, format: target_format.clone(), affected_files: image_count }
        }
        DetectedFormat::VOC { .. } => {
            voc::write_voc_dataset(&dataset, output_path)?;
            ConvertResult { output_path: output_dir_clone, format: target_format.clone(), affected_files: image_count }
        }
        DetectedFormat::LabelStudio { .. } => {
            let ls_config = LSExportConfig {
                from_name: "label".to_string(),
                to_name: "image".to_string(),
                image_source: "local-files".to_string(),
                output_format: "Tasks".to_string(),
            };
            let output_file = label_studio::export_label_studio(dataset.clone(), ls_config)?;
            ConvertResult { output_path: output_file, format: target_format.clone(), affected_files: image_count }
        }
        DetectedFormat::TFRecord { .. } => {
            tfrecord::write_tfrecord(dataset.clone(), output_path)?;
            ConvertResult { output_path: output_dir_clone, format: target_format.clone(), affected_files: image_count }
        }
        _ => return Err(anyhow::anyhow!("Unsupported target format")),
    };

    Ok(result)
}