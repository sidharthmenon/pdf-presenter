#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use std::fs;
use std::sync::Mutex;
use tauri::{WindowBuilder, WebviewWindowBuilder, Manager, Emitter, State};

struct AppState(Mutex<Option<String>>);

#[tauri::command]
fn close_singleview(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("singleview") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn load_pdf_bytes(path: String) -> Result<Vec<u8>, String> {
    fs::read(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_current_pdf_path(state: State<AppState>) -> Option<String> {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn set_current_pdf_path(state: State<AppState>, pdf_path: String) {
    *state.0.lock().unwrap() = Some(pdf_path.clone());
}

#[tauri::command]
async fn open_pdf_presenter(app: tauri::AppHandle) -> Result<(), String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;

    // println!("Monitors: {:?}", monitors);

    if monitors.len() > 1 {
        let primary = &monitors[0];
        let secondary = &monitors[1];

        // println!("Primary: {:?}", primary);
        // println!("Secondary: {:?}", secondary);


        let singleview = WebviewWindowBuilder::new(&app, "singleview", tauri::WebviewUrl::App("single".into()))
            .title("PDF Presenter")
            .inner_size(
                secondary.size().width as f64,
                secondary.size().height as f64,
            )
            .position(
                secondary.position().x as f64,
                secondary.position().y as f64,
            )
            .build()
            .map_err(|e| e.to_string())?;

        singleview.set_fullscreen(true).unwrap();
        
        
    } else {
        // Single combined window
        let singleview = WebviewWindowBuilder::new(&app, "singleview", tauri::WebviewUrl::App("single".into()))
            .title("PDF Presenter")
            .fullscreen(true)
            .build()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(None)))
        .plugin(tauri_plugin_dialog::init()) 
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            open_pdf_presenter, 
            get_current_pdf_path, 
            set_current_pdf_path, 
            load_pdf_bytes, 
            close_singleview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}