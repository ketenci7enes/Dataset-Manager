use crate::formats::unified::{UnifiedDataset, UnifiedImage, UnifiedAnnotation};
use crate::types::{AugmentPipeline, AugmentOp};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::path::Path;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AugmentResult {
    pub output_path: String,
    pub affected_files: usize,
}

pub fn preview_augmentation(image_path: &str, pipeline: AugmentPipeline) -> Result<Vec<String>> {
    let img = image::open(image_path)?;
    let mut results = Vec::new();

    // Original image as base64
    let mut orig_bytes = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut orig_bytes), image::ImageFormat::Png)?;
    results.push(general_purpose::STANDARD.encode(&orig_bytes));

    // Apply pipeline 3 times for preview
    for _ in 0..3 {
        let mut aug_img = img.clone();
        for op in &pipeline.operations {
            aug_img = apply_augment_op(aug_img, op)?;
        }

        let mut bytes = Vec::new();
        aug_img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        results.push(general_purpose::STANDARD.encode(&bytes));
    }

    Ok(results)
}

pub fn run_augmentation(dataset: UnifiedDataset, pipeline: AugmentPipeline, output_dir: String) -> Result<AugmentResult> {
    let output_path = Path::new(&output_dir);
    let images_dir = output_path.join("images");
    let labels_dir = output_path.join("labels");
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&labels_dir)?;

    let mut rng = if let Some(seed) = pipeline.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    };

    let mut augmented_count = 0;

    for image in &dataset.images {
        // Handle potential file:// prefix from frontend
        let clean_path = image.source_path.strip_prefix("file://").unwrap_or(&image.source_path);
        let src_path = Path::new(clean_path);
        if !src_path.exists() {
            continue;
        }

        let mut img = image::open(src_path)?;

        // Apply pipeline
        for op in &pipeline.operations {
            if rng.gen::<f32>() < op.probability {
                img = apply_augment_op(img, op)?;
            }
        }

        // Save augmented image
        let ext = src_path.extension().and_then(|s| s.to_str()).unwrap_or("jpg");
        let new_name = format!("aug_{}.{}", Uuid::new_v4().to_string()[..8].to_string(), ext);
        let new_image_path = images_dir.join(&new_name);
        img.save(&new_image_path)?;

        // Save corresponding labels
        let new_label_path = labels_dir.join(new_name.replace(ext, "txt"));
        let label_content = generate_yolo_labels(&image, &dataset);
        fs::write(&new_label_path, label_content)?;

        augmented_count += 1;
    }

    // Copy classes.txt
    let classes_src = Path::new(&dataset.info.path).join("classes.txt");
    if classes_src.exists() {
        fs::copy(classes_src, output_path.join("classes.txt"))?;
    }

    Ok(AugmentResult {
        output_path: output_dir,
        affected_files: augmented_count,
    })
}

fn apply_augment_op(img: DynamicImage, op: &AugmentOp) -> Result<DynamicImage> {
    let mut rng = rand::thread_rng();

    match op.op_type.as_str() {
        "Rotate" => {
            let degrees = op.params.get("degrees").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32;
            let angle = rng.gen_range(-degrees..degrees);
            // Simple 90-degree rotations only to avoid imageproc version issues
            let angle_90 = (angle / 90.0).round() * 90.0;
            if angle_90 != 0.0 {
                let rotations = (angle_90 / 90.0) as i32;
                let mut result = img;
                for _ in 0..rotations.abs() {
                    if rotations > 0 {
                        result = result.rotate90();
                    } else {
                        result = result.rotate270();
                    }
                }
                Ok(result)
            } else {
                Ok(img)
            }
        }
        "Flip" => {
            let horizontal = op.params.get("horizontal").and_then(|v| v.as_bool()).unwrap_or(true);
            let vertical = op.params.get("vertical").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut result = img;
            if horizontal {
                result = result.fliph();
            }
            if vertical {
                result = result.flipv();
            }
            Ok(result)
        }
        "Brightness" => {
            let factor = op.params.get("factor").and_then(|v| v.as_f64()).unwrap_or(1.2) as f32;
            let factor = rng.gen_range(1.0/factor..factor);
            Ok(img.brighten((factor * 255.0) as i32))
        }
        "Contrast" => {
            let factor = op.params.get("factor").and_then(|v| v.as_f64()).unwrap_or(1.2) as f32;
            let factor = rng.gen_range(1.0/factor..factor);
            Ok(img.adjust_contrast(factor))
        }
        "Noise" => {
            let std = op.params.get("std").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32;
            Ok(add_noise(img, std))
        }
        "Blur" => {
            // Skip blur for now to avoid imageproc version issues
            Ok(img)
        }
        _ => Ok(img),
    }
}

fn add_noise(img: DynamicImage, std: f32) -> DynamicImage {
    let mut rng = rand::thread_rng();
    let mut result = img.to_rgb8();

    for pixel in result.pixels_mut() {
        let mut channels = pixel.0;
        for channel in &mut channels {
            let noise: f32 = rng.gen_range(-std..std);
            let val = *channel as f32 + noise;
            *channel = val.clamp(0.0, 255.0) as u8;
        }
        pixel.0 = channels;
    }

    DynamicImage::ImageRgb8(result)
}

fn generate_yolo_labels(image: &UnifiedImage, dataset: &UnifiedDataset) -> String {
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