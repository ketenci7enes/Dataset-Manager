use crate::{
    converter, formats, mixer, organizer, splitter, workspace,
};
use crate::formats::unified::UnifiedDataset;
use crate::types::*;
use image::ImageOutputFormat;
use parking_lot::Mutex;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use tauri::command;

lazy_static::lazy_static! {
    static ref OPERATION_HISTORY: Arc<Mutex<Vec<OperationLog>>> = Arc::new(Mutex::new(Vec::new()));
}

fn add_to_history(log: OperationLog) {
    OPERATION_HISTORY.lock().push(log);
}

#[command]
pub async fn pick_workspace() -> Result<WorkspaceInfo, String> {
    Err("Use frontend dialog to pick folder".to_string())
}

#[command]
pub async fn detect_workspace_format(path: String) -> Result<DetectedFormat, String> {
    workspace::detector::detect_format(&std::path::PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[command]
pub async fn scan_workspace(window: tauri::Window, path: String, options: Option<ScanOptions>) -> Result<UnifiedDataset, String> {
    let opts = options.clone().unwrap_or_default();
    let emit_progress = opts.emit_progress.unwrap_or(false);
    
    let result = workspace::scanner::scan_workspace(&std::path::PathBuf::from(path), options)
        .map_err(|e| e.to_string())?;
    
    if emit_progress {
        let _ = window.emit("scan-complete", serde_json::json!({
            "total_images": result.images.len(),
            "total_annotations": result.info.total_annotations,
            "total_classes": result.categories.len(),
        }));
    }
    
    Ok(result)
}

#[command]
pub async fn convert_format(
    source: UnifiedDataset,
    target_format: DetectedFormat,
    options: ConvertOptions,
) -> Result<OperationResult, String> {
    let output_dir = options.output_dir.as_ref().map(|s| s.clone()).unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap()
            .join("exports")
            .to_string_lossy()
            .to_string()
    });

    let result = converter::pipeline::convert_dataset(source, target_format, output_dir, options)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "convert_format".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: format!("Converted dataset to {:?}", result.format),
        reversible: false,
        data: serde_json::json!({ "output_path": result.output_path }),
    });

    Ok(OperationResult {
        success: true,
        message: "Conversion completed successfully".to_string(),
        output_path: Some(result.output_path),
        affected_files: Some(result.affected_files),
    })
}

#[command]
pub async fn export_label_studio(
    dataset: UnifiedDataset,
    config: LSExportConfig,
) -> Result<String, String> {
    let output_path = formats::label_studio::export_label_studio(dataset, config)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "export_label_studio".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: "Exported dataset to Label Studio format".to_string(),
        reversible: false,
        data: serde_json::json!({ "output_path": output_path }),
    });

    Ok(output_path)
}

#[command]
pub async fn organize_files(
    dataset: UnifiedDataset,
    config: OrganizeConfig,
) -> Result<OperationResult, String> {
    let result = organizer::renamer::organize_dataset(dataset, config)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "organize_files".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: "Organized dataset files".to_string(),
        reversible: true,
        data: serde_json::json!({ "output_path": result.output_path, "mappings": result.mappings }),
    });

    Ok(OperationResult {
        success: true,
        message: "Files organized successfully".to_string(),
        output_path: Some(result.output_path),
        affected_files: Some(result.affected_files),
    })
}

#[command]
pub async fn mix_datasets(
    sources: Vec<MixSource>,
    config: MixConfig,
) -> Result<OperationResult, String> {
    let result = mixer::mixer::mix_datasets(sources, config)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "mix_datasets".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: "Mixed multiple datasets".to_string(),
        reversible: false,
        data: serde_json::json!({ "output_path": result.output_path }),
    });

    Ok(OperationResult {
        success: true,
        message: "Datasets mixed successfully".to_string(),
        output_path: Some(result.output_path),
        affected_files: Some(result.affected_files),
    })
}

#[command]
pub async fn preview_augmentation(
    image_path: String,
    pipeline: AugmentPipeline,
) -> Result<Vec<String>, String> {
    crate::augment::pipeline::preview_augmentation(&image_path, pipeline)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn run_augmentation(
    dataset: UnifiedDataset,
    pipeline: AugmentPipeline,
    output_dir: String,
) -> Result<OperationResult, String> {
    let result = crate::augment::pipeline::run_augmentation(dataset, pipeline, output_dir)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "run_augmentation".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: "Applied augmentation pipeline".to_string(),
        reversible: false,
        data: serde_json::json!({ "output_path": result.output_path }),
    });

    Ok(OperationResult {
        success: true,
        message: "Augmentation completed".to_string(),
        output_path: Some(result.output_path),
        affected_files: Some(result.affected_files),
    })
}

#[command]
pub async fn split_dataset(
    dataset: UnifiedDataset,
    config: SplitConfig,
) -> Result<OperationResult, String> {
    let result = splitter::splitter::split_dataset(dataset, config)
        .map_err(|e| e.to_string())?;

    add_to_history(OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: "split_dataset".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        description: "Split dataset into train/val/test".to_string(),
        reversible: false,
        data: serde_json::json!({ "output_path": result.output_path }),
    });

    Ok(OperationResult {
        success: true,
        message: "Dataset split completed".to_string(),
        output_path: Some(result.output_path),
        affected_files: Some(result.affected_files),
    })
}

#[command]
pub async fn get_operation_history() -> Result<Vec<OperationLog>, String> {
    Ok(OPERATION_HISTORY.lock().clone())
}

#[command]
pub async fn undo_last_operation() -> Result<OperationResult, String> {
    let mut history = OPERATION_HISTORY.lock();
    if let Some(last) = history.pop() {
        if !last.reversible {
            return Err("Operation is not reversible".to_string());
        }
        Ok(OperationResult {
            success: true,
            message: "Undo completed (placeholder)".to_string(),
            output_path: None,
            affected_files: None,
        })
    } else {
        Err("No operations to undo".to_string())
    }
}

#[command]
pub async fn generate_thumbnail(image_path: String, max_size: Option<u32>) -> Result<String, String> {
    let path = Path::new(&image_path);
    if !path.exists() {
        return Err("Image file not found".to_string());
    }
    
    let size = max_size.unwrap_or(256);
    
    let img = image::open(path).map_err(|e| e.to_string())?;
    let thumbnail = img.thumbnail(size, size);
    
    let mut buffer = Vec::new();
    thumbnail.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;
    
    let base64 = base64::encode(&buffer);
    Ok(format!("data:image/png;base64,{}", base64))
}

#[command]
pub async fn generate_thumbnails_batch(image_paths: Vec<String>, max_size: Option<u32>) -> Result<Vec<String>, String> {
    let size = max_size.unwrap_or(256);
    let mut results = Vec::with_capacity(image_paths.len());
    
    for path_str in image_paths {
        let path = Path::new(&path_str);
        if !path.exists() {
            results.push(String::new());
            continue;
        }
        
        match image::open(path) {
            Ok(img) => {
                let thumbnail = img.thumbnail(size, size);
                let mut buffer = Vec::new();
                if thumbnail.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png).is_ok() {
                    let base64 = base64::encode(&buffer);
                    results.push(format!("data:image/png;base64,{}", base64));
                } else {
                    results.push(String::new());
                }
            }
            Err(_) => {
                results.push(String::new());
            }
        }
    }
    
    Ok(results)
}

#[command]
pub async fn cancel_scan() -> Result<String, String> {
    crate::workspace::scanner::set_scan_cancelled(true);
    Ok("Scan cancellation requested".to_string())
}