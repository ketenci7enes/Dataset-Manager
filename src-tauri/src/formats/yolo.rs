use crate::formats::unified::{BBox, Category, DatasetInfo, UnifiedAnnotation, UnifiedDataset, UnifiedImage};
use crate::types::DetectedFormat;
use crate::workspace::detector;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "bmp", "gif", "tif", "tiff", "webp"];

/// Parses a YOLO dataset in place. Supports both:
///   dataset/images + dataset/labels
/// and split layouts:
///   dataset/train/images + dataset/train/labels, dataset/valid/... etc.
pub fn parse_yolo_dataset(
    root: &Path,
    classes_file: Option<&str>,
    emit_progress: bool,
) -> Result<UnifiedDataset> {
    let class_names = load_class_names(root, classes_file);

    // Discover all labels directories (flat or nested per-split).
    let label_dirs = {
        let mut dirs = detector::find_label_dirs(root);
        if dirs.is_empty() {
            // Nothing found with the "labels" naming; fall back to root itself.
            dirs.push(root.to_path_buf());
        }
        dirs
    };

    let mut images: Vec<UnifiedImage> = Vec::new();
    let mut max_class_id: u64 = 0;
    let mut next_id: u64 = 0;

    for labels_dir in &label_dirs {
        let images_dir = detector::sibling_images_dir(labels_dir);
        let image_index = build_image_index(&images_dir);

        let entries: Vec<PathBuf> = WalkDir::new(labels_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("txt"))
                    .unwrap_or(false)
            })
            .filter(|e| !is_meta_label_file(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect();

        if emit_progress {
            // Progress is surfaced by the scanner via events; nothing extra here.
        }

        let mut parsed: Vec<UnifiedImage> = entries
            .par_iter()
            .filter_map(|label_path| {
                let stem = label_path.file_stem().and_then(|s| s.to_str())?;

                // Find the matching image: exact stem in the index, or search
                // recursively for any image with the same stem.
                let image_path = image_index
                    .get(stem)
                    .cloned()
                    .or_else(|| find_image_file(&images_dir, stem))
                    .or_else(|| find_image_file(labels_dir, stem))?;

                if !image_path.exists() {
                    return None;
                }

                let (width, height) = get_image_dimensions(&image_path).ok()?;
                let mut annotation_id = 0u64;
                let annotations = parse_yolo_labels(label_path, width, height, &mut annotation_id).ok()?;

                let file_name = image_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| stem.to_string());

                Some(UnifiedImage {
                    id: 0,
                    file_name,
                    width,
                    height,
                    annotations,
                    source_format: DetectedFormat::YOLO {
                        classes_file: classes_file.map(|s| s.to_string()),
                    },
                    source_path: normalize_path(&image_path),
                })
            })
            .collect();

        max_class_id = max_class_id.max(
            parsed
                .iter()
                .flat_map(|img| img.annotations.iter())
                .map(|a| a.category_id)
                .max()
                .map(|m| m)
                .unwrap_or(0),
        );

        // Deduplicate by source_path across splits (in case of overlapping dirs).
        for img in parsed.drain(..) {
            if images.iter().any(|existing| existing.source_path == img.source_path) {
                continue;
            }
            images.push(img);
        }
    }

    // Assign sequential IDs and fix annotation references.
    for (i, img) in images.iter_mut().enumerate() {
        img.id = i as u64;
        for ann in &mut img.annotations {
            ann.image_id = i as u64;
        }
    }
    next_id = images.len() as u64;
    let _ = next_id;

    // Build categories covering every class id that appears in the labels.
    let category_count = ((max_class_id + 1) as usize).max(class_names.len()).max(1);
    let categories: Vec<Category> = (0..category_count)
        .map(|i| Category {
            id: i as u64,
            name: class_names
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("class_{}", i)),
            supercategory: None,
            color: generate_color(i),
        })
        .collect();

    Ok(UnifiedDataset {
        info: DatasetInfo::default(),
        images,
        categories,
    })
}

pub fn write_yolo_dataset(
    dataset: &UnifiedDataset,
    output_dir: &Path,
    _options: crate::types::ConvertOptions,
) -> Result<()> {
    let images_dir = output_dir.join("images");
    let labels_dir = output_dir.join("labels");
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&labels_dir)?;

    let classes_file = output_dir.join("classes.txt");
    let class_names: Vec<String> = dataset.categories.iter().map(|c| c.name.clone()).collect();
    fs::write(&classes_file, class_names.join("\n"))?;

    for image in &dataset.images {
        let src_path = Path::new(&image.source_path);
        if src_path.exists() {
            let dst_path = unique_path(&images_dir, &image.file_name);
            fs::copy(src_path, &dst_path)?;
        }

        let label_name = Path::new(&image.file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| image.id.to_string())
            + ".txt";
        let label_path = labels_dir.join(label_name);

        let mut label_content = String::new();
        for ann in &image.annotations {
            if let Some(category) = dataset.categories.iter().find(|c| c.id == ann.category_id) {
                let (x_center, y_center, w, h) = ann.bbox.to_yolo(image.width, image.height);
                label_content.push_str(&format!(
                    "{} {:.6} {:.6} {:.6} {:.6}\n",
                    category.id, x_center, y_center, w, h
                ));
            }
        }
        fs::write(&label_path, label_content)?;
    }

    let yaml_content = format!(
        "path: {}\ntrain: images\nval: images\n\nnames:\n{}",
        output_dir.to_string_lossy(),
        class_names
            .iter()
            .enumerate()
            .map(|(i, name)| format!("  {}: {}", i, name))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(output_dir.join("data.yaml"), yaml_content)?;

    Ok(())
}

fn build_image_index(images_dir: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    if !images_dir.is_dir() {
        return index;
    }
    for entry in WalkDir::new(images_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let is_image = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| IMAGE_EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext)))
            .unwrap_or(false);
        if !is_image {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            index.entry(stem.to_string()).or_insert_with(|| path.to_path_buf());
        }
    }
    index
}

fn load_class_names(root: &Path, classes_file: Option<&str>) -> Vec<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(custom) = classes_file {
        candidates.push(PathBuf::from(custom));
    }
    candidates.push(root.join("classes.txt"));
    candidates.push(root.join("obj.names"));
    candidates.push(root.join("classes.names"));
    candidates.push(root.join("data.yaml"));
    candidates.push(root.join("data.yml"));

    // Nested class-name files (split layouts).
    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
            let lower = fname.to_ascii_lowercase();
            if lower == "classes.txt"
                || lower == "obj.names"
                || lower == "classes.names"
                || lower == "data.yaml"
                || lower == "data.yml"
            {
                candidates.push(path.to_path_buf());
            }
        }
    }

    for path in candidates {
        if !path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        let names = if path.extension().and_then(|s| s.to_str()).map(|e| {
            e.eq_ignore_ascii_case("yaml") || e.eq_ignore_ascii_case("yml")
        }) == Some(true)
        {
            parse_names_from_yaml(&content)
        } else {
            content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        if !names.is_empty() {
            return names;
        }
    }

    Vec::new()
}

/// Extracts class names from a YOLO `data.yaml` file.
/// Supports:
///   names: [cat, dog]
///   names: ['cat', 'dog']
///   names:
///     0: cat
///     1: dog
fn parse_names_from_yaml(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut names: Vec<(Option<usize>, String)> = Vec::new();
    let mut in_names = false;
    let mut order: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("names:") {
            in_names = true;
            let rest = rest.trim();
            if rest.starts_with('[') {
                // Inline list
                let inner = rest.trim_start_matches('[').trim_end_matches(']');
                for item in inner.split(',') {
                    let name = item.trim().trim_matches(|c| c == '\'' || c == '"').trim();
                    if !name.is_empty() {
                        order.push(name.to_string());
                    }
                }
                if !order.is_empty() {
                    return order;
                }
            }
            continue;
        }

        if in_names {
            // Stop when indentation ends (new top-level key).
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            let entry = trimmed;
            if let Some((key, value)) = entry.split_once(':') {
                let name = value.trim().trim_matches(|c| c == '\'' || c == '"').trim();
                if name.is_empty() {
                    continue;
                }
                if let Ok(idx) = key.trim().parse::<usize>() {
                    names.push((Some(idx), name.to_string()));
                } else {
                    order.push(name.to_string());
                }
            } else {
                // "- cat" list style
                let name = entry.trim_start_matches('-').trim().trim_matches(|c| c == '\'' || c == '"');
                if !name.is_empty() {
                    order.push(name.to_string());
                }
            }
        }
    }

    if !names.is_empty() {
        let max_idx = names.iter().filter_map(|(i, _)| *i).max().unwrap_or(0);
        let mut result = vec![String::new(); max_idx + 1];
        for (idx, name) in names {
            if let Some(i) = idx {
                if i < result.len() {
                    result[i] = name;
                }
            }
        }
        if result.iter().any(|s| !s.is_empty()) {
            return result;
        }
    }

    order
}

fn is_meta_label_file(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|stem| {
            let lower = stem.to_ascii_lowercase();
            lower == "classes"
                || lower == "obj"
                || lower == "readme"
                || lower == "notes"
                || lower.starts_with("_")
        })
        .unwrap_or(false)
}

fn find_image_file(images_dir: &Path, stem: &str) -> Option<PathBuf> {
    // Direct match first (cheap).
    for ext in IMAGE_EXTS {
        let candidate = images_dir.join(format!("{}.{}", stem, ext));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    // Case-insensitive extension + recursive fallback.
    let target = stem.to_ascii_lowercase();
    WalkDir::new(images_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .find(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(&target))
                .unwrap_or(false)
                && e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| IMAGE_EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext)))
                    .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
}

fn parse_yolo_labels(
    path: &Path,
    img_w: u32,
    img_h: u32,
    annotation_id: &mut u64,
) -> Result<Vec<UnifiedAnnotation>> {
    let content = std::fs::read_to_string(path)?;
    let mut annotations = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let Ok(class_id) = parts[0].parse::<u64>() else {
            continue;
        };
        let x_center: f32 = parts[1].parse().unwrap_or(0.0);
        let y_center: f32 = parts[2].parse().unwrap_or(0.0);
        let w: f32 = parts[3].parse().unwrap_or(0.0);
        let h: f32 = parts[4].parse().unwrap_or(0.0);

        let bbox = BBox::from_yolo(class_id, x_center, y_center, w, h, img_w, img_h);

        annotations.push(UnifiedAnnotation {
            id: *annotation_id,
            image_id: 0,
            category_id: class_id,
            bbox,
            segmentation: None,
            area: bbox.width * bbox.height,
            iscrowd: false,
        });
        *annotation_id += 1;
    }

    Ok(annotations)
}

fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let reader = image::io::Reader::open(path)?.with_guessed_format()?;
    let dimensions = reader.into_dimensions()?;
    Ok((dimensions.0, dimensions.1))
}

fn normalize_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn unique_path(dir: &Path, file_name: &str) -> PathBuf {
    let mut candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    let ext = Path::new(file_name)
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut i = 1;
    loop {
        let name = if ext.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, ext)
        };
        candidate = dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

fn generate_color(index: usize) -> String {
    let colors = [
        "#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
        "#F97316", "#6366F1",
    ];
    colors[index % colors.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_split_layout() {
        let dir = std::env::temp_dir().join(format!("dm_yolo_split_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for split in ["train", "valid"] {
            fs::create_dir_all(dir.join(split).join("labels")).unwrap();
            fs::create_dir_all(dir.join(split).join("images")).unwrap();
        }
        fs::write(dir.join("classes.txt"), "cat\ndog").unwrap();
        fs::write(dir.join("train/labels/a.txt"), "0 0.5 0.5 0.2 0.2\n1 0.3 0.3 0.1 0.1").unwrap();
        fs::write(dir.join("valid/labels/b.txt"), "1 0.6 0.6 0.3 0.3").unwrap();
        write_test_image(&dir.join("train/images/a.jpg"));
        write_test_image(&dir.join("valid/images/b.jpg"));

        let ds = parse_yolo_dataset(&dir, None, false).unwrap();
        assert_eq!(ds.images.len(), 2, "should find images across splits");
        assert_eq!(ds.categories.len(), 2);
        assert_eq!(ds.categories[0].name, "cat");
        assert_eq!(ds.categories[1].name, "dog");
        let total_anns: usize = ds.images.iter().map(|i| i.annotations.len()).sum();
        assert_eq!(total_anns, 3);

        let _ = fs::remove_dir_all(&dir);
    }

    fn write_test_image(path: &Path) {
        let img = image::RgbImage::from_pixel(100, 80, image::Rgb([128, 128, 128]));
        img.save(path).unwrap();
    }

    #[test]
    fn parses_data_yaml_names() {
        let yaml = "path: /x\ntrain: images\nnames:\n  0: person\n  1: car\n";
        let names = parse_names_from_yaml(yaml);
        assert_eq!(names, vec!["person".to_string(), "car".to_string()]);

        let inline = "names: ['a', 'b', 'c']";
        assert_eq!(parse_names_from_yaml(inline), vec!["a", "b", "c"]);
    }
}
