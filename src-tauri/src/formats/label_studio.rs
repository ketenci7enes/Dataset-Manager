use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, BBox, DatasetInfo};
use crate::types::DetectedFormat;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use image;

#[derive(Debug, Deserialize)]
struct LSTask {
    data: LSData,
    annotations: Option<Vec<LSAnnotation>>,
}

#[derive(Debug, Deserialize)]
struct LSData {
    image: String,
}

#[derive(Debug, Deserialize)]
struct LSAnnotation {
    result: Vec<LSResult>,
    ground_truth: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LSResult {
    id: String,
    #[serde(rename = "type")]
    result_type: String,
    value: LSValue,
    from_name: String,
    to_name: String,
    original_width: u32,
    original_height: u32,
}

#[derive(Debug, Deserialize)]
struct LSValue {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    rectanglelabels: Vec<String>,
}

pub fn parse_label_studio_dataset(root: &Path, json_file: &str, _emit_progress: bool) -> Result<UnifiedDataset> {
    let json_path = PathBuf::from(json_file);
    let content = std::fs::read_to_string(&json_path)?;
    let tasks: Vec<LSTask> = serde_json::from_str(&content)?;

    let mut category_map: HashMap<String, Category> = HashMap::new();
    let mut category_id_counter = 0u64;

    let mut images = Vec::new();
    let mut annotation_id = 0u64;

    for task in tasks {
        let image_url = &task.data.image;
        let (image_path, image_name) = resolve_image_path(root, &json_path, image_url);

        let (width, height) = if image_path.exists() {
            get_image_dimensions(&image_path)?
        } else {
            (1280, 720) // default fallback
        };

        let mut annotations = Vec::new();

        if let Some(anns) = task.annotations {
            for ann in anns {
                for result in ann.result {
                    if result.result_type != "rectanglelabels" {
                        continue;
                    }

                    for label in &result.value.rectanglelabels {
                        let category = category_map.entry(label.clone()).or_insert_with(|| {
                            let id = category_id_counter;
                            category_id_counter += 1;
                            Category {
                                id,
                                name: label.clone(),
                                supercategory: None,
                                color: generate_color(id as usize),
                            }
                        }).clone();

                        let bbox = BBox::from_label_studio(
                            result.value.x,
                            result.value.y,
                            result.value.width,
                            result.value.height,
                            result.original_width,
                            result.original_height,
                        );

                        annotations.push(UnifiedAnnotation {
                            id: annotation_id,
                            image_id: images.len() as u64,
                            category_id: category.id,
                            bbox,
                            segmentation: None,
                            area: bbox.width * bbox.height,
                            iscrowd: false,
                        });
                        annotation_id += 1;
                    }
                }
            }
        }

        images.push(UnifiedImage {
            id: images.len() as u64,
            file_name: image_name,
            width,
            height,
            annotations,
            source_format: DetectedFormat::LabelStudio { json_file: json_file.to_string() },
            source_path: image_path.to_string_lossy().to_string(),
        });
    }

    let categories: Vec<Category> = category_map.into_values().collect();

    Ok(UnifiedDataset {
        info: DatasetInfo::default(),
        images,
        categories,
    })
}

fn resolve_image_path(root: &Path, json_path: &Path, image_url: &str) -> (PathBuf, String) {
    // Handle local-files:// protocol
    if image_url.starts_with("local-files://") {
        let path_str = image_url.strip_prefix("local-files://").unwrap();
        // Try to decode URL encoding
        let path_str = urlencoding::decode(path_str).unwrap_or_else(|_| path_str.into());
        let path = PathBuf::from(path_str.as_ref());
        if path.exists() {
            return (path.clone(), path.file_name().unwrap().to_string_lossy().to_string());
        }
    }

    // Handle data URIs (skip for now)
    if image_url.starts_with("data:") {
        return (PathBuf::new(), "data_uri".to_string());
    }

    // Try relative to JSON file
    let relative_path = json_path.parent().unwrap().join(image_url);
    if relative_path.exists() {
        return (relative_path.clone(), relative_path.file_name().unwrap().to_string_lossy().to_string());
    }

    // Try relative to root
    let root_relative = root.join(image_url);
    if root_relative.exists() {
        return (root_relative.clone(), root_relative.file_name().unwrap().to_string_lossy().to_string());
    }

    // Try images subdirectory
    let images_dir = root.join("images").join(image_url);
    if images_dir.exists() {
        return (images_dir.clone(), images_dir.file_name().unwrap().to_string_lossy().to_string());
    }

    // Fallback
    (PathBuf::new(), image_url.to_string())
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let img = image::open(path)?;
    Ok((img.width(), img.height()))
}

fn generate_color(index: usize) -> String {
    let colors = [
        "#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6",
        "#EC4899", "#06B6D4", "#84CC16", "#F97316", "#6366F1",
    ];
    colors[index % colors.len()].to_string()
}

pub fn export_label_studio(dataset: UnifiedDataset, config: crate::types::LSExportConfig) -> Result<String> {
    use chrono::Utc;
    use std::fs;
    use uuid::Uuid;

    let output_dir = format!("{}/exports", dataset.info.path);
    fs::create_dir_all(&output_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let output_file = format!("{}/label_studio_{}.json", output_dir, timestamp);

    let mut tasks = Vec::new();

    for image in &dataset.images {
        let mut task = serde_json::json!({
            "data": {
                "image": format!("local-files://{}", image.source_path)
            }
        });

        if config.output_format == "Tasks" || config.output_format == "Both" {
            task["annotations"] = serde_json::json!([]);
        }

        if let Some(annotations) = task["annotations"].as_array_mut() {
            for ann in &image.annotations {
                if let Some(category) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
                    let (x, y, w, h) = ann.bbox.to_label_studio(image.width, image.height);

                    let result = serde_json::json!({
                        "id": Uuid::new_v4().to_string(),
                        "type": "rectanglelabels",
                        "value": {
                            "x": x,
                            "y": y,
                            "width": w,
                            "height": h,
                            "rotation": 0,
                            "rectanglelabels": [category.name]
                        },
                        "to_name": config.to_name,
                        "from_name": config.from_name,
                        "image_rotation": 0,
                        "original_width": image.width,
                        "original_height": image.height
                    });

                    annotations.push(serde_json::json!({
                        "result": [result],
                        "ground_truth": false
                    }));
                }
            }
        }

        tasks.push(task);
    }

    fs::write(&output_file, serde_json::to_string_pretty(&tasks)?)?;

    Ok(output_file)
}