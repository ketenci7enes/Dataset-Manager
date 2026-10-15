#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod workspace;
pub mod formats;
pub mod converter;
pub mod organizer;
pub mod mixer;
pub mod augment;
pub mod splitter;
mod types;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::pick_workspace,
            commands::detect_workspace_format,
            commands::scan_workspace,
            commands::convert_format,
            commands::export_label_studio,
            commands::organize_files,
            commands::mix_datasets,
            commands::preview_augmentation,
            commands::run_augmentation,
            commands::split_dataset,
            commands::get_operation_history,
            commands::undo_last_operation,
            commands::generate_thumbnail,
            commands::generate_thumbnails_batch,
            commands::cancel_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}