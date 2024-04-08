// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod llm;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_pdf(path: String) -> Result<String, String> {
    println!("{path}");
    let bytes = std::fs::read(path).unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();

    // println!("{out:?}");
    Ok(out)
}

#[tauri::command]
async fn get_answer(prompt: String) -> Result<String, String> {
    let response = crate::llm::run_mistral(prompt).unwrap();

    Ok(response)
}

fn main() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, get_pdf, get_answer])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
