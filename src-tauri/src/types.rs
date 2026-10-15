use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DetectedFormat {
    YOLO {
        #[serde(default)]
        classes_file: Option<String>,
    },
    COCO {
        #[serde(default)]
        annotation_file: String,
    },
    VOC {
        #[serde(default)]
        annotations_dir: String,
    },
    LabelStudio {
        #[serde(default)]
        json_file: String,
    },
    TFRecord {
        #[serde(default)]
        record_files: Vec<String>,
    },
    Mixed { formats: Vec<DetectedFormat> },
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub detected_format: DetectedFormat,
    pub last_opened: String,
    pub custom_class_names: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub emit_progress: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub class_mapping: Option<HashMap<String, String>>,
    pub normalize_coordinates: Option<bool>,
    pub output_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LSExportConfig {
    pub from_name: String,
    pub to_name: String,
    pub image_source: String,
    pub output_format: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeConfig {
    pub naming_template: String,
    pub create_class_folders: bool,
    pub conflict_strategy: String,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MixSource {
    pub path: String,
    pub weight: f32,
    pub max_samples: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MixConfig {
    pub sources: Vec<MixSource>,
    pub strategy: String,
    pub output_format: DetectedFormat,
    pub balance_classes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AugmentOp {
    pub op_type: String,
    pub params: HashMap<String, serde_json::Value>,
    pub probability: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AugmentPipeline {
    pub operations: Vec<AugmentOp>,
    pub seed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SplitRatios {
    pub train: f32,
    pub val: f32,
    pub test: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SplitConfig {
    pub ratios: SplitRatios,
    pub strategy: String,
    pub output_formats: Vec<DetectedFormat>,
    pub seed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub output_path: Option<String>,
    pub affected_files: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationLog {
    pub id: String,
    pub operation_type: String,
    pub timestamp: String,
    pub description: String,
    pub reversible: bool,
    pub data: serde_json::Value,
}