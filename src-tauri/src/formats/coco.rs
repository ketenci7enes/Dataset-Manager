use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, BBox, DatasetInfo};
use crate::types::DetectedFormat;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::BufReader;
use image;

#[derive(Debug, Deserialize)]
struct CocoDataset {
    images: Vec<CocoImage>,
    annotations: Vec<CocoAnnotation>,
    categories: Vec<CocoCategory>,
}

#[derive(Debug, Deserialize)]
struct CocoImage {
    id: u64,
    file_name: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct CocoAnnotation {
    id: u64,
    image_id: u64,
    category_id: u64,
    bbox: [f32; 4],
    segmentation: Option<Vec<Vec<f32>>>,
    area: f32,
    iscrowd: bool,
}

#[derive(Debug, Deserialize)]
struct CocoCategory {
    id: u64,
    name: String,
    supercategory: Option<String>,
}

pub fn parse_coco_dataset(root: &Path, annotation_file: &str, _emit_progress: bool) -> Result<UnifiedDataset> {
    let annotation_path = PathBuf::from(annotation_file);
    let file = std::fs::File::open(&annotation_path)?;
    let reader = BufReader::new(file);
    
    // Stream parse the JSON to avoid loading entire file into memory
    let mut deserializer = Deserializer::from_reader(reader).into_iter::<Value>();
    
    let root_value = deserializer.next()
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("Empty JSON file"))?;
    
    let coco: CocoDataset = serde_json::from_value(root_value)?;

    let images_dir = find_images_dir(root, &annotation_path);

    // Build category map
    let mut category_map: HashMap<u64, Category> = HashMap::new();
    for cat in &coco.categories {
        category_map.insert(cat.id, Category {
            id: cat.id,
            name: cat.name.clone(),
            supercategory: cat.supercategory.clone(),
            color: generate_color(cat.id as usize),
        });
    }

    // Build image map
    let mut image_map: HashMap<u64, UnifiedImage> = HashMap::new();
    for img in &coco.images {
        let image_path = images_dir.join(&img.file_name);
        let (width, height) = if image_path.exists() {
            get_image_dimensions(&image_path)?
        } else {
            (img.width, img.height)
        };

        image_map.insert(img.id, UnifiedImage {
            id: img.id,
            file_name: img.file_name.clone(),
            width,
            height,
            annotations: Vec::new(),
            source_format: DetectedFormat::COCO { annotation_file: annotation_file.to_string() },
            source_path: image_path.to_string_lossy().to_string(),
        });
    }

    // Add annotations to images
    for ann in &coco.annotations {
        if let Some(image) = image_map.get_mut(&ann.image_id) {
            let bbox = BBox::from_coco(ann.bbox);
            image.annotations.push(UnifiedAnnotation {
                id: ann.id,
                image_id: ann.image_id,
                category_id: ann.category_id,
                bbox,
                segmentation: ann.segmentation.clone(),
                area: ann.area,
                iscrowd: ann.iscrowd,
            });
        }
    }

    let images: Vec<UnifiedImage> = image_map.into_values().collect();
    let categories: Vec<Category> = category_map.into_values().collect();

    Ok(UnifiedDataset {
        info: DatasetInfo::default(),
        images,
        categories,
    })
}

#[derive(Debug, Serialize)]
struct CocoDatasetOut {
    images: Vec<CocoImageOut>,
    annotations: Vec<CocoAnnotationOut>,
    categories: Vec<CocoCategoryOut>,
}

#[derive(Debug, Serialize)]
struct CocoImageOut {
    id: u64,
    file_name: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
struct CocoAnnotationOut {
    id: u64,
    image_id: u64,
    category_id: u64,
    bbox: [f32; 4],
    segmentation: Option<Vec<Vec<f32>>>,
    area: f32,
    iscrowd: bool,
}

#[derive(Debug, Serialize)]
struct CocoCategoryOut {
    id: u64,
    name: String,
    supercategory: Option<String>,
}

pub fn write_coco_dataset(dataset: &UnifiedDataset, output_dir: &Path) -> Result<()> {
    let images_dir = output_dir.join("images");
    let annotations_dir = output_dir.join("annotations");
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&annotations_dir)?;

    let mut coco_images = Vec::new();
    let mut coco_annotations = Vec::new();
    let mut annotation_id = 1u64;

    for image in &dataset.images {
        // Copy image
        let src_path = Path::new(&image.source_path);
        if src_path.exists() {
            let dst_path = images_dir.join(&image.file_name);
            fs::copy(src_path, &dst_path)?;
        }

        coco_images.push(CocoImageOut {
            id: image.id,
            file_name: image.file_name.clone(),
            width: image.width,
            height: image.height,
        });

        for ann in &image.annotations {
            coco_annotations.push(CocoAnnotationOut {
                id: annotation_id,
                image_id: image.id,
                category_id: ann.category_id,
                bbox: ann.bbox.to_coco(),
                segmentation: ann.segmentation.clone(),
                area: ann.area,
                iscrowd: ann.iscrowd,
            });
            annotation_id += 1;
        }
    }

    let coco_categories: Vec<CocoCategoryOut> = dataset.categories.iter().map(|c| {
        CocoCategoryOut {
            id: c.id,
            name: c.name.clone(),
            supercategory: c.supercategory.clone(),
        }
    }).collect();

    let coco = CocoDatasetOut {
        images: coco_images,
        annotations: coco_annotations,
        categories: coco_categories,
    };

    fs::write(
        annotations_dir.join("instances.json"),
        serde_json::to_string_pretty(&coco)?
    )?;

    Ok(())
}

fn find_images_dir(root: &Path, annotation_path: &Path) -> PathBuf {
    let candidates = [
        root.join("images"),
        root.join("train"),
        root.join("val"),
        root.join("test"),
        annotation_path.parent().unwrap().join("images"),
        annotation_path.parent().unwrap().join("..").join("images"),
        root.to_path_buf(),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.to_path_buf();
        }
    }
    root.to_path_buf()
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    let dimensions = reader.into_dimensions()?;
    Ok((dimensions.0, dimensions.1))
}

fn generate_color(index: usize) -> String {
    let colors = [
        "#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6",
        "#EC4899", "#06B6D4", "#84CC16", "#F97316", "#6366F1",
    ];
    colors[index % colors.len()].to_string()
}