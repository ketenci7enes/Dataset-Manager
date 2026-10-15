use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation, Category, BBox, DatasetInfo};
use crate::types::DetectedFormat;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use image;
use quick_xml::de::from_str;

#[derive(Debug, Deserialize)]
struct VocAnnotation {
    #[serde(rename = "filename")]
    filename: String,
    #[serde(rename = "size")]
    size: VocSize,
    #[serde(rename = "object")]
    objects: Vec<VocObject>,
}

#[derive(Debug, Deserialize)]
struct VocSize {
    width: u32,
    height: u32,
    depth: u32,
}

#[derive(Debug, Deserialize)]
struct VocObject {
    name: String,
    #[serde(rename = "bndbox")]
    bndbox: VocBndBox,
    difficult: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct VocBndBox {
    xmin: f32,
    ymin: f32,
    xmax: f32,
    ymax: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename = "annotation")]
struct VocAnnotationOut {
    folder: String,
    filename: String,
    path: String,
    source: VocSource,
    size: VocSizeOut,
    segmented: u8,
    object: Vec<VocObjectOut>,
}

#[derive(Debug, Serialize)]
struct VocSource {
    database: String,
}

#[derive(Debug, Serialize)]
struct VocSizeOut {
    width: u32,
    height: u32,
    depth: u32,
}

#[derive(Debug, Serialize)]
struct VocObjectOut {
    name: String,
    pose: String,
    truncated: u8,
    difficult: u8,
    bndbox: VocBndBoxOut,
}

#[derive(Debug, Serialize)]
struct VocBndBoxOut {
    xmin: u32,
    ymin: u32,
    xmax: u32,
    ymax: u32,
}

pub fn parse_voc_dataset(root: &Path, annotations_dir: &str, _emit_progress: bool) -> Result<UnifiedDataset> {
    let annotations_path = PathBuf::from(annotations_dir);
    let images_dir = find_images_dir(root, &annotations_path);

    let mut category_map: HashMap<String, Category> = HashMap::new();
    let mut category_id_counter = 0u64;

    let mut images = Vec::new();
    let mut annotation_id = 0u64;

    for entry in WalkDir::new(&annotations_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("xml") {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let voc: VocAnnotation = from_str(&content)?;

        let image_path = images_dir.join(&voc.filename);
        let (width, height) = if image_path.exists() {
            get_image_dimensions(&image_path)?
        } else {
            (voc.size.width, voc.size.height)
        };

        let mut annotations = Vec::new();
        for obj in &voc.objects {
            let category = category_map.entry(obj.name.clone()).or_insert_with(|| {
                let id = category_id_counter;
                category_id_counter += 1;
                Category {
                    id,
                    name: obj.name.clone(),
                    supercategory: None,
                    color: generate_color(id as usize),
                }
            }).clone();

            let bbox = BBox::from_voc(obj.bndbox.xmin, obj.bndbox.ymin, obj.bndbox.xmax, obj.bndbox.ymax);

            annotations.push(UnifiedAnnotation {
                id: annotation_id,
                image_id: images.len() as u64,
                category_id: category.id,
                bbox,
                segmentation: None,
                area: bbox.width * bbox.height,
                iscrowd: obj.difficult.unwrap_or(0) == 1,
            });
            annotation_id += 1;
        }

        images.push(UnifiedImage {
            id: images.len() as u64,
            file_name: voc.filename,
            width,
            height,
            annotations,
            source_format: DetectedFormat::VOC { annotations_dir: annotations_dir.to_string() },
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

pub fn write_voc_dataset(dataset: &UnifiedDataset, output_dir: &Path) -> Result<()> {
    let images_dir = output_dir.join("JPEGImages");
    let annotations_dir = output_dir.join("Annotations");
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&annotations_dir)?;

    for image in &dataset.images {
        // Copy image
        let src_path = Path::new(&image.source_path);
        if src_path.exists() {
            let dst_path = images_dir.join(&image.file_name);
            fs::copy(src_path, &dst_path)?;
        }

        // Create XML annotation
        let mut objects = Vec::new();
        for ann in &image.annotations {
            if let Some(category) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
                let (xmin, ymin, xmax, ymax) = ann.bbox.to_voc();
                objects.push(VocObjectOut {
                    name: category.name.clone(),
                    pose: "Unspecified".to_string(),
                    truncated: 0,
                    difficult: if ann.iscrowd { 1 } else { 0 },
                    bndbox: VocBndBoxOut {
                        xmin: xmin.max(0.0) as u32,
                        ymin: ymin.max(0.0) as u32,
                        xmax: xmax.min(image.width as f32) as u32,
                        ymax: ymax.min(image.height as f32) as u32,
                    },
                });
            }
        }

        let annotation = VocAnnotationOut {
            folder: "JPEGImages".to_string(),
            filename: image.file_name.clone(),
            path: images_dir.join(&image.file_name).to_string_lossy().to_string(),
            source: VocSource { database: "Unknown".to_string() },
            size: VocSizeOut {
                width: image.width,
                height: image.height,
                depth: 3,
            },
            segmented: 0,
            object: objects,
        };

        let xml_content = quick_xml::se::to_string(&annotation)?;
        let xml_name = Path::new(&image.file_name)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string() + ".xml";
        fs::write(annotations_dir.join(xml_name), xml_content)?;
    }

    Ok(())
}

fn find_images_dir(root: &Path, annotations_path: &Path) -> PathBuf {
    let candidates = [
        root.join("JPEGImages"),
        root.join("images"),
        annotations_path.parent().unwrap().join("..").join("JPEGImages"),
        annotations_path.parent().unwrap().join("..").join("images"),
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