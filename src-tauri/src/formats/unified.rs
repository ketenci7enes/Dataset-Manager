use crate::types::DetectedFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDataset {
    pub info: DatasetInfo,
    pub images: Vec<UnifiedImage>,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetInfo {
    pub name: String,
    pub path: String,
    pub total_images: usize,
    pub total_annotations: usize,
    pub class_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedImage {
    pub id: u64,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub annotations: Vec<UnifiedAnnotation>,
    pub source_format: DetectedFormat,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAnnotation {
    pub id: u64,
    pub image_id: u64,
    pub category_id: u64,
    pub bbox: BBox,
    pub segmentation: Option<Vec<Vec<f32>>>,
    pub area: f32,
    pub iscrowd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub supercategory: Option<String>,
    pub color: String,
}

impl BBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_yolo(class_id: u64, x_center: f32, y_center: f32, w: f32, h: f32, img_w: u32, img_h: u32) -> Self {
        let x = (x_center - w / 2.0) * img_w as f32;
        let y = (y_center - h / 2.0) * img_h as f32;
        let width = w * img_w as f32;
        let height = h * img_h as f32;
        Self { x, y, width, height }
    }

    pub fn to_yolo(&self, img_w: u32, img_h: u32) -> (f32, f32, f32, f32) {
        let x_center = (self.x + self.width / 2.0) / img_w as f32;
        let y_center = (self.y + self.height / 2.0) / img_h as f32;
        let w = self.width / img_w as f32;
        let h = self.height / img_h as f32;
        (x_center, y_center, w, h)
    }

    pub fn to_voc(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }

    pub fn to_coco(&self) -> [f32; 4] {
        [self.x, self.y, self.width, self.height]
    }

    pub fn to_label_studio(&self, img_w: u32, img_h: u32) -> (f32, f32, f32, f32) {
        let x = (self.x / img_w as f32) * 100.0;
        let y = (self.y / img_h as f32) * 100.0;
        let width = (self.width / img_w as f32) * 100.0;
        let height = (self.height / img_h as f32) * 100.0;
        (x, y, width, height)
    }

    pub fn from_voc(xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> Self {
        Self { x: xmin, y: ymin, width: xmax - xmin, height: ymax - ymin }
    }

    pub fn from_coco(bbox: [f32; 4]) -> Self {
        Self { x: bbox[0], y: bbox[1], width: bbox[2], height: bbox[3] }
    }

    pub fn from_label_studio(x: f32, y: f32, width: f32, height: f32, img_w: u32, img_h: u32) -> Self {
        let x = (x / 100.0) * img_w as f32;
        let y = (y / 100.0) * img_h as f32;
        let w = (width / 100.0) * img_w as f32;
        let h = (height / 100.0) * img_h as f32;
        Self { x, y, width: w, height: h }
    }
}