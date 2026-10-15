use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, DatasetInfo};
use crate::types::{MixSource, MixConfig, DetectedFormat, ScanOptions};
use crate::workspace::scanner;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct MixResult {
    pub output_path: String,
    pub affected_files: usize,
}

pub fn mix_datasets(sources: Vec<MixSource>, config: MixConfig) -> Result<MixResult> {
    let mut combined = UnifiedDataset {
        info: DatasetInfo {
            name: String::new(),
            path: String::new(),
            total_images: 0,
            total_annotations: 0,
            class_distribution: HashMap::new(),
        },
        images: Vec::new(),
        categories: Vec::new(),
    };
    let mut image_id_counter = 0u64;
    let mut annotation_id_counter = 0u64;
    let mut category_map: HashMap<String, Category> = HashMap::new();
    let mut category_id_counter = 0u64;

    // First pass: collect all unique categories
    for source in &sources {
        let dataset = scanner::scan_workspace(Path::new(&source.path), Some(ScanOptions::default()))?;
        for cat in &dataset.categories {
            category_map.entry(cat.name.clone()).or_insert_with(|| {
                let mut new_cat = cat.clone();
                new_cat.id = category_id_counter;
                category_id_counter += 1;
                new_cat
            });
        }
    }

    // Build unified category list
    combined.categories = category_map.values().cloned().collect();

    // Second pass: merge images with remapped category IDs
    for source in &sources {
        let dataset = scanner::scan_workspace(Path::new(&source.path), Some(ScanOptions::default()))?;

        // Build category ID mapping for this source
        let mut source_cat_map: HashMap<u64, u64> = HashMap::new();
        for cat in &dataset.categories {
            if let Some(unified_cat) = category_map.get(&cat.name) {
                source_cat_map.insert(cat.id, unified_cat.id);
            }
        }

        // Process images
        for mut image in dataset.images {
            // Remap annotation category IDs
            for ann in &mut image.annotations {
                if let Some(&new_cat_id) = source_cat_map.get(&ann.category_id) {
                    ann.category_id = new_cat_id;
                }
                ann.image_id = image_id_counter;
                ann.id = annotation_id_counter;
                annotation_id_counter += 1;
            }

            image.id = image_id_counter;
            image_id_counter += 1;

            combined.images.push(image);
        }
    }

    // Apply mixing strategy
    combined.images = apply_mix_strategy(combined.images, &config)?;

    // Generate output
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = Path::new(&sources[0].path).join(format!("mixed_{}", timestamp));
    std::fs::create_dir_all(&output_dir)?;

    let affected_count = combined.images.len();

    // Write in target format
    let convert_options = crate::types::ConvertOptions {
        class_mapping: None,
        normalize_coordinates: None,
        output_dir: None,
    };
    crate::converter::pipeline::convert_dataset(combined, config.output_format, output_dir.to_string_lossy().to_string(), convert_options)?;

    Ok(MixResult {
        output_path: output_dir.to_string_lossy().to_string(),
        affected_files: affected_count,
    })
}

fn apply_mix_strategy(mut images: Vec<UnifiedImage>, config: &MixConfig) -> Result<Vec<UnifiedImage>> {
    match config.strategy.as_str() {
        "Concatenate" => Ok(images),
        "Stratified" => apply_stratified_sampling(images, config),
        "Balanced" => apply_balanced_sampling(images, config),
        _ => Ok(images),
    }
}

fn apply_stratified_sampling(images: Vec<UnifiedImage>, _config: &MixConfig) -> Result<Vec<UnifiedImage>> {
    Ok(images)
}

fn apply_balanced_sampling(images: Vec<UnifiedImage>, _config: &MixConfig) -> Result<Vec<UnifiedImage>> {
    Ok(images)
}