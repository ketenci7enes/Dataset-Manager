use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, DatasetInfo};
use crate::converter::pipeline::convert_dataset;
use crate::types::{SplitConfig, SplitRatios, DetectedFormat};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug)]
pub struct SplitResult {
    pub output_path: String,
    pub affected_files: usize,
    pub split_info: SplitInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct SplitInfo {
    pub train_count: usize,
    pub val_count: usize,
    pub test_count: usize,
    pub train_images: Vec<String>,
    pub val_images: Vec<String>,
    pub test_images: Vec<String>,
    pub class_distribution: HashMap<String, SplitClassDist>,
}

#[derive(Debug, serde::Serialize)]
pub struct SplitClassDist {
    pub train: usize,
    pub val: usize,
    pub test: usize,
}

pub fn split_dataset(dataset: UnifiedDataset, config: SplitConfig) -> Result<SplitResult> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = Path::new(&dataset.info.path).join(format!("splits_{}", timestamp));
    fs::create_dir_all(&output_dir)?;

    let mut rng = StdRng::seed_from_u64(config.seed);

    let (train_images, val_images, test_images) = match config.strategy.as_str() {
        "Stratified" => stratified_split(&dataset, &config.ratios, &mut rng)?,
        "BySource" => split_by_source(&dataset, &config.ratios, &mut rng)?,
        _ => random_split(&dataset, &config.ratios, &mut rng)?,
    };

    // Create split datasets
    let train_dataset = create_split_dataset(&dataset, &train_images, "train");
    let val_dataset = create_split_dataset(&dataset, &val_images, "val");
    let test_dataset = create_split_dataset(&dataset, &test_images, "test");

    // Export in each requested format
    for format in &config.output_formats {
        let split_output = output_dir.join(format!("{:?}", format).to_lowercase());
        fs::create_dir_all(&split_output)?;

        let convert_options = crate::types::ConvertOptions {
            class_mapping: None,
            normalize_coordinates: None,
            output_dir: None,
        };
        convert_dataset(train_dataset.clone(), format.clone(), split_output.join("train").to_string_lossy().to_string(), convert_options.clone())?;
        convert_dataset(val_dataset.clone(), format.clone(), split_output.join("val").to_string_lossy().to_string(), convert_options.clone())?;
        convert_dataset(test_dataset.clone(), format.clone(), split_output.join("test").to_string_lossy().to_string(), convert_options)?;
    }

    // Generate split info
    let split_info = generate_split_info(&dataset, &train_images, &val_images, &test_images);

    // Save split info
    fs::write(output_dir.join("split_info.json"), serde_json::to_string_pretty(&split_info)?)?;

    // Copy classes.txt
    let classes_src = Path::new(&dataset.info.path).join("classes.txt");
    if classes_src.exists() {
        fs::copy(classes_src, output_dir.join("classes.txt"))?;
    }

    Ok(SplitResult {
        output_path: output_dir.to_string_lossy().to_string(),
        affected_files: dataset.images.len(),
        split_info,
    })
}

fn random_split(
    dataset: &UnifiedDataset,
    ratios: &SplitRatios,
    rng: &mut StdRng,
) -> Result<(Vec<usize>, Vec<usize>, Vec<usize>)> {
    let mut indices: Vec<usize> = (0..dataset.images.len()).collect();
    indices.shuffle(rng);

    let train_end = (dataset.images.len() as f32 * ratios.train) as usize;
    let val_end = train_end + (dataset.images.len() as f32 * ratios.val) as usize;

    Ok((
        indices[..train_end].to_vec(),
        indices[train_end..val_end].to_vec(),
        indices[val_end..].to_vec(),
    ))
}

fn stratified_split(
    dataset: &UnifiedDataset,
    ratios: &SplitRatios,
    rng: &mut StdRng,
) -> Result<(Vec<usize>, Vec<usize>, Vec<usize>)> {
    // Group images by their primary class
    let mut class_groups: HashMap<u64, Vec<usize>> = HashMap::new();

    for (idx, image) in dataset.images.iter().enumerate() {
        let primary = find_primary_class(image);
        class_groups.entry(primary).or_default().push(idx);
    }

    let mut train = Vec::new();
    let mut val = Vec::new();
    let mut test = Vec::new();

    for (_, mut indices) in class_groups {
        indices.shuffle(rng);
        let train_end = (indices.len() as f32 * ratios.train) as usize;
        let val_end = train_end + (indices.len() as f32 * ratios.val) as usize;

        train.extend(&indices[..train_end]);
        val.extend(&indices[train_end..val_end]);
        test.extend(&indices[val_end..]);
    }

    train.shuffle(rng);
    val.shuffle(rng);
    test.shuffle(rng);

    Ok((train, val, test))
}

fn split_by_source(
    dataset: &UnifiedDataset,
    ratios: &SplitRatios,
    rng: &mut StdRng,
) -> Result<(Vec<usize>, Vec<usize>, Vec<usize>)> {
    // Group by source path
    let mut source_groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, image) in dataset.images.iter().enumerate() {
        let source = Path::new(&image.source_path)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        source_groups.entry(source).or_default().push(idx);
    }

    let mut train = Vec::new();
    let mut val = Vec::new();
    let mut test = Vec::new();

    for (_, mut indices) in source_groups {
        indices.shuffle(rng);
        let train_end = (indices.len() as f32 * ratios.train) as usize;
        let val_end = train_end + (indices.len() as f32 * ratios.val) as usize;

        train.extend(&indices[..train_end]);
        val.extend(&indices[train_end..val_end]);
        test.extend(&indices[val_end..]);
    }

    train.shuffle(rng);
    val.shuffle(rng);
    test.shuffle(rng);

    Ok((train, val, test))
}

fn find_primary_class(image: &UnifiedImage) -> u64 {
    let mut class_counts: HashMap<u64, usize> = HashMap::new();
    for ann in &image.annotations {
        *class_counts.entry(ann.category_id).or_insert(0) += 1;
    }
    class_counts.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(id, _)| id)
        .unwrap_or(0)
}

fn create_split_dataset(dataset: &UnifiedDataset, indices: &[usize], split_name: &str) -> UnifiedDataset {
    let mut images = Vec::new();
    for &idx in indices {
        let mut img = dataset.images[idx].clone();
        img.id = images.len() as u64;
        for ann in &mut img.annotations {
            ann.image_id = img.id;
        }
        images.push(img);
    }

    UnifiedDataset {
        info: DatasetInfo {
            name: format!("{}_dataset", split_name),
            path: dataset.info.path.clone(),
            total_images: images.len(),
            total_annotations: images.iter().map(|i| i.annotations.len()).sum(),
            class_distribution: HashMap::new(),
        },
        images,
        categories: dataset.categories.clone(),
    }
}

fn generate_split_info(
    dataset: &UnifiedDataset,
    train: &[usize],
    val: &[usize],
    test: &[usize],
) -> SplitInfo {
    let mut class_dist: HashMap<String, SplitClassDist> = HashMap::new();

    for cat in &dataset.categories {
        class_dist.insert(cat.name.clone(), SplitClassDist { train: 0, val: 0, test: 0 });
    }

    fn count_classes(indices: &[usize], dataset: &UnifiedDataset, dist: &mut HashMap<String, SplitClassDist>, split: &str) {
        for &idx in indices {
            for ann in &dataset.images[idx].annotations {
                if let Some(cat) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
                    if let Some(entry) = dist.get_mut(&cat.name) {
                        match split {
                            "train" => entry.train += 1,
                            "val" => entry.val += 1,
                            "test" => entry.test += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    count_classes(train, dataset, &mut class_dist, "train");
    count_classes(val, dataset, &mut class_dist, "val");
    count_classes(test, dataset, &mut class_dist, "test");

    SplitInfo {
        train_count: train.len(),
        val_count: val.len(),
        test_count: test.len(),
        train_images: train.iter().map(|&i| dataset.images[i].file_name.clone()).collect(),
        val_images: val.iter().map(|&i| dataset.images[i].file_name.clone()).collect(),
        test_images: test.iter().map(|&i| dataset.images[i].file_name.clone()).collect(),
        class_distribution: class_dist,
    }
}