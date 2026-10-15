use crate::types::DetectedFormat;
use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "bmp", "gif", "tif", "tiff", "webp"];

fn name_eq(path: &Path, name: &str) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case(name))
        .unwrap_or(false)
}

fn dir_has_extension(dir: &Path, exts: &[&str]) -> bool {
    if !dir.is_dir() {
        return false;
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| exts.iter().any(|x| x.eq_ignore_ascii_case(ext)))
                .unwrap_or(false)
        })
}

fn has_yolo_format(root: &Path) -> bool {
    find_label_dirs(root).is_empty() == false
}

/// Finds every directory that looks like a YOLO `labels` folder.
/// Handles both the flat layout:
///   dataset/labels/*.txt + dataset/images/*
/// and the split layout commonly produced by Roboflow:
///   dataset/train/labels/*.txt + dataset/train/images/*
///   dataset/valid/labels/*.txt + dataset/valid/images/*
pub(crate) fn find_label_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    // Direct root/labels
    let direct = root.join("labels");
    if direct.is_dir() && dir_has_extension(&direct, &["txt"]) {
        dirs.push(direct);
    }

    // Nested */labels at up to 4 levels deep (train/, valid/, test/, ...)
    for entry in WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if path == root {
            continue;
        }
        // Case-insensitive "labels"
        let is_labels = name_eq(path, "labels");
        if is_labels && dir_has_extension(path, &["txt"]) && !dirs.contains(&path.to_path_buf()) {
            dirs.push(path.to_path_buf());
        }
    }

    dirs
}

/// Given a labels directory, returns the matching images directory.
pub(crate) fn sibling_images_dir(labels_dir: &Path) -> PathBuf {
    if let Some(parent) = labels_dir.parent() {
        // Look for a sibling that is named images (any case)
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    && name_eq(&entry.path(), "images")
                {
                    return entry.path();
                }
            }
        }
    }
    // Fall back to labels dir itself (labels next to images in same folder)
    labels_dir.to_path_buf()
}

fn find_classes_file(root: &Path) -> Option<String> {
    // Explicit class-name files first (root, then nested for split layouts)
    for name in &["classes.txt", "obj.names", "classes.names"] {
        let path = root.join(name);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    // Nested class-name files (train/classes.txt ...)
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
            if ["classes.txt", "obj.names", "classes.names"]
                .iter()
                .any(|n| n.eq_ignore_ascii_case(fname))
            {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    // data.yaml / *.yaml that likely contains class names
    for name in &["data.yaml", "data.yml", "dataset.yaml", "dataset.yml"] {
        let path = root.join(name);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    for entry in WalkDir::new(root)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if (ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
                    && std::fs::read_to_string(path)
                        .map(|c| c.contains("names:"))
                        .unwrap_or(false)
                {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

fn find_coco_file(root: &Path) -> Option<String> {
    for name in &[
        "annotations.json",
        "instances.json",
        "coco.json",
        "train.json",
        "val.json",
        "test.json",
    ] {
        let path = root.join(name);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    // Check in annotations subdir
    let annotations_dir = root.join("annotations");
    if annotations_dir.exists() {
        for name in &["annotations.json", "instances.json", "coco.json"] {
            let path = annotations_dir.join(name);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        // Any json in annotations/ that contains "images" and "annotations"
        if let Ok(entries) = std::fs::read_dir(&annotations_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("\"images\"") && content.contains("\"annotations\"") {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn has_voc_format(root: &Path) -> bool {
    let annotations_dir = root.join("Annotations");
    let images_dir = root.join("JPEGImages").exists() || root.join("images").exists();
    annotations_dir.exists()
        && images_dir
        && dir_has_extension(&annotations_dir, &["xml"])
}

fn find_label_studio_file(root: &Path) -> Option<String> {
    for name in &[
        "proje_girdisi.json",
        "label_studio.json",
        "ls_export.json",
        "export.json",
    ] {
        let path = root.join(name);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    // Check for any .json that looks like LS format
    WalkDir::new(root)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path().extension().map(|ext| ext == "json").unwrap_or(false)
                && {
                    if let Ok(content) = std::fs::read_to_string(e.path()) {
                        content.contains("rectanglelabels")
                            || content.contains("polygonlabels")
                            || content.contains("brushlabels")
                    } else {
                        false
                    }
                }
        })
        .map(|e| e.path().to_string_lossy().to_string())
}

fn has_tfrecord_format(root: &Path) -> bool {
    WalkDir::new(root).max_depth(1).into_iter().any(|e| {
        e.ok()
            .map(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "tfrecord" || ext == "record")
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    })
}

fn find_tfrecord_files(root: &Path) -> Vec<String> {
    WalkDir::new(root)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "tfrecord" || ext == "record")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

pub fn detect_format(root: &Path) -> Result<DetectedFormat> {
    if !root.exists() {
        return Ok(DetectedFormat::Unknown);
    }

    let mut found_formats = Vec::new();

    // YOLO (flat and split layouts)
    if has_yolo_format(root) {
        let classes_file = find_classes_file(root);
        found_formats.push(DetectedFormat::YOLO { classes_file });
    }

    // COCO
    if let Some(coco_file) = find_coco_file(root) {
        found_formats.push(DetectedFormat::COCO {
            annotation_file: coco_file,
        });
    }

    // VOC
    if has_voc_format(root) {
        found_formats.push(DetectedFormat::VOC {
            annotations_dir: root.join("Annotations").to_string_lossy().to_string(),
        });
    }

    // Label Studio
    if let Some(ls_file) = find_label_studio_file(root) {
        found_formats.push(DetectedFormat::LabelStudio { json_file: ls_file });
    }

    // TFRecord
    if has_tfrecord_format(root) {
        let record_files = find_tfrecord_files(root);
        found_formats.push(DetectedFormat::TFRecord { record_files });
    }

    match found_formats.len() {
        0 => Ok(DetectedFormat::Unknown),
        1 => Ok(found_formats.into_iter().next().unwrap()),
        _ => {
            // Prefer YOLO when it is available, since the other detectors can
            // produce false positives on split YOLO layouts (e.g. a classes.txt
            // that also looks like a COCO file).
            if let Some(yolo) = found_formats
                .iter()
                .find(|f| matches!(f, DetectedFormat::YOLO { .. }))
            {
                return Ok(yolo.clone());
            }
            Ok(DetectedFormat::Mixed {
                formats: found_formats,
            })
        }
    }
}

#[allow(dead_code)]
pub(crate) fn is_image_ext(ext: &str) -> bool {
    IMAGE_EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_flat_yolo() {
        let dir = std::env::temp_dir().join(format!("dm_flat_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("labels")).unwrap();
        fs::create_dir_all(dir.join("images")).unwrap();
        fs::write(dir.join("labels/a.txt"), "0 0.5 0.5 0.1 0.1").unwrap();
        fs::write(dir.join("classes.txt"), "cat").unwrap();

        let fmt = detect_format(&dir).unwrap();
        assert!(matches!(fmt, DetectedFormat::YOLO { .. }), "got {:?}", fmt);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_split_yolo() {
        let dir = std::env::temp_dir().join(format!("dm_split_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for split in ["train", "valid", "test"] {
            fs::create_dir_all(dir.join(split).join("labels")).unwrap();
            fs::create_dir_all(dir.join(split).join("images")).unwrap();
            fs::write(
                dir.join(split).join("labels/a.txt"),
                "0 0.5 0.5 0.1 0.1",
            )
            .unwrap();
        }
        fs::write(dir.join("data.yaml"), "names:\n  0: cat\n").unwrap();

        let fmt = detect_format(&dir).unwrap();
        assert!(matches!(fmt, DetectedFormat::YOLO { .. }), "got {:?}", fmt);
        let _ = fs::remove_dir_all(&dir);
    }
}
