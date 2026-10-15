use crate::formats::{coco, label_studio, voc, yolo};
use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, DatasetInfo, BBox};
use crate::types::{DetectedFormat, ScanOptions};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;
use uuid::Uuid;

static SCAN_CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub stage: String,
}

pub fn set_scan_cancelled(cancelled: bool) {
    SCAN_CANCELLED.store(cancelled, Ordering::SeqCst);
}

pub fn is_scan_cancelled() -> bool {
    SCAN_CANCELLED.load(Ordering::SeqCst)
}

pub fn scan_workspace(root: &Path, options: Option<ScanOptions>) -> Result<UnifiedDataset> {
    SCAN_CANCELLED.store(false, Ordering::SeqCst);
    let opts = options.unwrap_or_default();
    let emit_progress = opts.emit_progress.unwrap_or(false);
    let offset = opts.offset.unwrap_or(0);
    let limit = opts.limit.unwrap_or(usize::MAX);

    let format = crate::workspace::detector::detect_format(root)?;

    let mut dataset = match &format {
        DetectedFormat::YOLO { classes_file } => {
            yolo::parse_yolo_dataset(root, classes_file.as_deref(), emit_progress)?
        }
        DetectedFormat::COCO { annotation_file } => {
            coco::parse_coco_dataset(root, annotation_file, emit_progress)?
        }
        DetectedFormat::VOC { annotations_dir } => {
            voc::parse_voc_dataset(root, annotations_dir, emit_progress)?
        }
        DetectedFormat::LabelStudio { json_file } => {
            label_studio::parse_label_studio_dataset(root, json_file, emit_progress)?
        }
        DetectedFormat::TFRecord { .. } => {
            return Err(anyhow::anyhow!("TFRecord reading not implemented yet"));
        }
        DetectedFormat::Mixed { formats } => {
            let mut combined = UnifiedDataset::default();
            for fmt in formats {
                let single = scan_single_format(root, fmt, emit_progress)?;
                combined.images.extend(single.images);
                for cat in single.categories {
                    if !combined.categories.iter().any(|c| c.id == cat.id) {
                        combined.categories.push(cat);
                    }
                }
            }
            combined
        }
        DetectedFormat::Unknown => {
            return Err(anyhow::anyhow!("Unknown dataset format"));
        }
    };

    // Check for cancellation
    if is_scan_cancelled() {
        return Err(anyhow::anyhow!("Scan cancelled by user"));
    }

    // Apply pagination
    if offset > 0 || limit < usize::MAX {
        let len = dataset.images.len();
        let skip = offset.min(len);
        let take = (offset.saturating_add(limit)).min(len).saturating_sub(skip);
        dataset.images = dataset.images.into_iter().skip(skip).take(take).collect();
        // Re-index IDs
        for (i, img) in dataset.images.iter_mut().enumerate() {
            img.id = i as u64;
            for ann in &mut img.annotations {
                ann.image_id = i as u64;
            }
        }
    }

    // Generate dataset info (based on full dataset before pagination)
    let info = generate_dataset_info(&dataset, root);
    Ok(UnifiedDataset { info, ..dataset })
}

fn scan_single_format(root: &Path, format: &DetectedFormat, emit_progress: bool) -> Result<UnifiedDataset> {
    match format {
        DetectedFormat::YOLO { classes_file } => yolo::parse_yolo_dataset(root, classes_file.as_deref(), emit_progress),
        DetectedFormat::COCO { annotation_file } => coco::parse_coco_dataset(root, annotation_file, emit_progress),
        DetectedFormat::VOC { annotations_dir } => voc::parse_voc_dataset(root, annotations_dir, emit_progress),
        DetectedFormat::LabelStudio { json_file } => label_studio::parse_label_studio_dataset(root, json_file, emit_progress),
        _ => Err(anyhow::anyhow!("Unsupported format for single scan")),
    }
}

fn generate_dataset_info(dataset: &UnifiedDataset, root: &Path) -> DatasetInfo {
    let mut class_distribution: HashMap<String, usize> = HashMap::new();
    let total_annotations: usize = dataset.images.iter().map(|img| img.annotations.len()).sum();

    for img in &dataset.images {
        for ann in &img.annotations {
            if let Some(cat) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
                *class_distribution.entry(cat.name.clone()).or_insert(0) += 1;
            }
        }
    }

    DatasetInfo {
        name: root.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string(),
        path: root.to_string_lossy().to_string(),
        total_images: dataset.images.len(),
        total_annotations,
        class_distribution,
    }
}

impl Default for UnifiedDataset {
    fn default() -> Self {
        Self {
            info: DatasetInfo {
                name: String::new(),
                path: String::new(),
                total_images: 0,
                total_annotations: 0,
                class_distribution: HashMap::new(),
            },
            images: Vec::new(),
            categories: Vec::new(),
        }
    }
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    let dimensions = reader.into_dimensions()?;
    Ok((dimensions.0, dimensions.1))
}