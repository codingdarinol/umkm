// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;

use database::{Database, NewTransaction, Transaction};
use std::sync::Arc;
use tauri::Manager;

#[tauri::command]
fn add_transaction(
    amount: i64,
    description: Option<String>,
    category: Option<String>,
    db: tauri::State<Arc<Database>>,
) -> Result<Transaction, String> {
    let new_transaction = NewTransaction {
        amount,
        description,
        category,
    };
    
    db.add_transaction(new_transaction)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transactions(
    limit: Option<i64>,
    db: tauri::State<Arc<Database>>,
) -> Result<Vec<Transaction>, String> {
    db.get_transactions(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monthly_balance(db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_monthly_balance().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_time_balance(db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_all_time_balance().map_err(|e| e.to_string())
}

#[tauri::command]
fn export_csv(db: tauri::State<Arc<Database>>) -> Result<String, String> {
    db.export_transactions_csv().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_transaction(id: i64, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.delete_transaction(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_totals(db: tauri::State<Arc<Database>>) -> Result<Vec<(String, i64)>, String> {
    db.get_category_totals().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_categories(db: tauri::State<Arc<Database>>) -> Result<Vec<String>, String> {
    db.get_categories().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_category(name: String, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.add_category(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_category(name: String, db: tauri::State<Arc<Database>>) -> Result<(), String> {
    db.delete_category(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_available_months(db: tauri::State<Arc<Database>>) -> Result<Vec<String>, String> {
    db.get_available_months().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_balance_for_month(month: String, db: tauri::State<Arc<Database>>) -> Result<i64, String> {
    db.get_balance_for_month(month).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_transactions_for_month(
    month: String,
    limit: Option<i64>,
    db: tauri::State<Arc<Database>>,
) -> Result<Vec<Transaction>, String> {
    db.get_transactions_for_month(month, limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_totals_for_month(month: String, db: tauri::State<Arc<Database>>) -> Result<Vec<(String, i64)>, String> {
    db.get_category_totals_for_month(month).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
            
            let db_path = app_dir.join("spent.db");
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize database"));
            
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_transaction,
            get_transactions,
            get_monthly_balance,
            get_all_time_balance,
            delete_transaction,
            get_category_totals,
            get_categories,
            add_category,
            delete_category,
            export_csv,
            get_available_months,
            get_balance_for_month,
            get_transactions_for_month,
            get_category_totals_for_month
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
