//! Artifex — Tauri v2 Desktop Application
//!
//! Phase B: Tauri App + Database
//! Provides project management with SQLite persistence.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    src_tauri::run_app();
}
