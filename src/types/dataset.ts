export interface Category {
  id: number;
  name: string;
  supercategory?: string;
  color: string;
}

export interface BBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface UnifiedAnnotation {
  id: number;
  image_id: number;
  category_id: number;
  bbox: BBox;
  segmentation?: number[][];
  area: number;
  iscrowd: boolean;
}

export interface UnifiedImage {
  id: number;
  file_name: string;
  width: number;
  height: number;
  annotations: UnifiedAnnotation[];
  source_format: DetectedFormat;
  source_path: string;
}

export interface DatasetInfo {
  name: string;
  path: string;
  total_images: number;
  total_annotations: number;
  class_distribution: Record<string, number>;
}

export interface UnifiedDataset {
  info: DatasetInfo;
  images: UnifiedImage[];
  categories: Category[];
}

export type DetectedFormat =
  | { type: 'YOLO'; classes_file?: string }
  | { type: 'COCO'; annotation_file: string }
  | { type: 'VOC'; annotations_dir: string }
  | { type: 'LabelStudio'; json_file: string }
  | { type: 'TFRecord'; record_files: string[] }
  | { type: 'Mixed'; formats: DetectedFormat[] }
  | { type: 'Unknown' };

export interface WorkspaceInfo {
  id: string;
  path: string;
  name: string;
  detected_format: DetectedFormat;
  last_opened: string;
  custom_class_names?: Record<string, string>;
}

export interface OperationLog {
  id: string;
  type: string;
  timestamp: string;
  description: string;
  reversible: boolean;
  data: any;
}

export interface OperationResult {
  success: boolean;
  message: string;
  output_path?: string;
  affected_files?: number;
}

export interface ConvertOptions {
  class_mapping?: Record<string, string>;
  normalize_coordinates?: boolean;
  output_dir?: string;
}

export interface LSExportConfig {
  from_name: string;
  to_name: string;
  image_source: string;
  output_format: 'Tasks' | 'Annotations' | 'Both';
}

export interface OrganizeConfig {
  naming_template: string;
  create_class_folders: boolean;
  conflict_strategy: 'Skip' | 'Overwrite' | 'AutoRename';
  dry_run: boolean;
}

export interface MixSource {
  path: string;
  weight: number;
  max_samples?: number;
}

export interface MixConfig {
  sources: MixSource[];
  strategy: 'Concatenate' | 'Stratified' | 'Balanced';
  output_format: DetectedFormat;
  balance_classes: boolean;
}

export interface AugmentOp {
  opType: string;
  params: Record<string, any>;
  probability: number;
}

export interface AugmentPipeline {
  operations: AugmentOp[];
  seed?: number;
}

export interface SplitRatios {
  train: number;
  val: number;
  test: number;
}

export interface SplitConfig {
  ratios: SplitRatios;
  strategy: 'Random' | 'Stratified' | 'BySource' | 'ByDate';
  output_formats: DetectedFormat[];
  seed: number;
}

export interface ScanOptions {
  offset?: number;
  limit?: number;
  emit_progress?: boolean;
}

export interface ScanProgress {
  current: number;
  total: number;
  stage: string;
}

export interface ThumbnailOptions {
  max_size?: number;
}