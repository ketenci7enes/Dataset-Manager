use crate::formats::unified::UnifiedDataset;
use crate::types::OrganizeConfig;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;

#[derive(Debug)]
pub struct OrganizeResult {
    pub output_path: String,
    pub affected_files: usize,
    pub mappings: Vec<FileMapping>,
}

#[derive(Debug, serde::Serialize)]
pub struct FileMapping {
    pub old_path: String,
    pub new_path: String,
    pub old_label_path: Option<String>,
    pub new_label_path: Option<String>,
}

pub fn organize_dataset(dataset: UnifiedDataset, config: OrganizeConfig) -> Result<OrganizeResult> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_dir = Path::new(&dataset.info.path).join(format!("organized_{}", timestamp));
    let images_dir = output_dir.join("images");
    let labels_dir = output_dir.join("labels");

    if config.create_class_folders {
        // Create class subdirectories
        for category in &dataset.categories {
            fs::create_dir_all(images_dir.join(&category.name))?;
            fs::create_dir_all(labels_dir.join(&category.name))?;
        }
    } else {
        fs::create_dir_all(&images_dir)?;
        fs::create_dir_all(&labels_dir)?;
    }

    let mut mappings = Vec::new();
    let mut class_counters: HashMap<u64, usize> = HashMap::new();

    for image in &dataset.images {
        // Determine primary class for this image (most frequent)
        let primary_class = find_primary_class(&image, &dataset);
        let class_name = dataset.categories.iter()
            .find(|c| c.id == primary_class)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Generate new filename
        let counter = class_counters.entry(primary_class).or_insert(0);
        *counter += 1;

        let ext = Path::new(&image.file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");

        let new_name = config.naming_template
            .replace("{class}", &class_name)
            .replace("{index}", &format!("{:06}", counter))
            .replace("{hash}", &Uuid::new_v4().to_string()[..8])
            .replace("{date}", &chrono::Utc::now().format("%Y%m%d").to_string())
            .replace("{original}", &Path::new(&image.file_name).file_stem().unwrap().to_string_lossy())
            .replace("{ext}", &format!(".{}", ext));

        // Determine destination paths
        let (new_image_path, new_label_path) = if config.create_class_folders {
            (
                images_dir.join(&class_name).join(&new_name),
                labels_dir.join(&class_name).join(new_name.replace(&format!(".{}", ext), ".txt")),
            )
        } else {
            (
                images_dir.join(&new_name),
                labels_dir.join(new_name.replace(&format!(".{}", ext), ".txt")),
            )
        };

        // Handle conflicts
        let final_image_path = resolve_conflict(&new_image_path, &config.conflict_strategy)?;
        let final_label_path = resolve_conflict(&new_label_path, &config.conflict_strategy)?;

        // Copy files if not dry run
        if !config.dry_run {
            fs::copy(&image.source_path, &final_image_path)?;

            // Generate label file in new location
            if let Some(first_ann) = image.annotations.first() {
                if let Some(category) = dataset.categories.iter().find(|c| c.id == first_ann.category_id) {
                    let label_content = generate_yolo_labels(&image, &dataset);
                    fs::write(&final_label_path, label_content)?;
                }
            }
        }

        mappings.push(FileMapping {
            old_path: image.source_path.clone(),
            new_path: final_image_path.to_string_lossy().to_string(),
            old_label_path: find_label_file(&image.source_path),
            new_label_path: Some(final_label_path.to_string_lossy().to_string()),
        });
    }

    // Copy classes.txt
    if !config.dry_run {
        let classes_src = Path::new(&dataset.info.path).join("classes.txt");
        if classes_src.exists() {
            fs::copy(classes_src, output_dir.join("classes.txt"))?;
        }
    }

    Ok(OrganizeResult {
        output_path: output_dir.to_string_lossy().to_string(),
        affected_files: mappings.len(),
        mappings,
    })
}

fn find_primary_class(image: &crate::formats::unified::UnifiedImage, _dataset: &UnifiedDataset) -> u64 {
    let mut class_counts: HashMap<u64, usize> = HashMap::new();
    for ann in &image.annotations {
        *class_counts.entry(ann.category_id).or_insert(0) += 1;
    }
    class_counts.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(id, _)| id)
        .unwrap_or(0)
}

fn find_label_file(image_path: &str) -> Option<String> {
    let path = Path::new(image_path);
    let stem = path.file_stem()?.to_string_lossy();
    let label_dir = path.parent()?.parent()?.join("labels");
    let label_path = label_dir.join(format!("{}.txt", stem));
    if label_path.exists() {
        Some(label_path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn generate_yolo_labels(image: &crate::formats::unified::UnifiedImage, dataset: &UnifiedDataset) -> String {
    let mut content = String::new();
    for ann in &image.annotations {
        if let Some(category) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
            let (x_center, y_center, w, h) = ann.bbox.to_yolo(image.width, image.height);
            content.push_str(&format!("{} {:.6} {:.6} {:.6} {:.6}\n",
                category.id, x_center, y_center, w, h));
        }
    }
    content
}

fn resolve_conflict(path: &Path, strategy: &str) -> Result<PathBuf> {
    match strategy {
        "Skip" => {
            if path.exists() {
                return Err(anyhow::anyhow!("File exists and skip strategy selected"));
            }
            Ok(path.to_path_buf())
        }
        "Overwrite" => Ok(path.to_path_buf()),
        "AutoRename" => {
            let mut counter = 1;
            let mut new_path = path.to_path_buf();
            while new_path.exists() {
                let stem = path.file_stem().unwrap().to_string_lossy();
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                let parent = path.parent().unwrap();
                new_path = parent.join(format!("{}_{}.{}", stem, counter, ext));
                counter += 1;
            }
            Ok(new_path)
        }
        _ => Ok(path.to_path_buf()),
    }
}