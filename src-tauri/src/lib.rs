// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod plugins;

use plugins::{automation::*, llm::*, screen_reader::*, speech::*, system::*};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // plugins
            screen_read_text,
            get_active_window_title,
            simulate_keyboard_input,
            simulate_mouse_click,
            init_whisper,
            push_pcm_chunk,
            generate_response,
            get_system_info
        ])
        .setup(|app| {
            // initialize whisper model path (adjust path to your model)
            // You can also call init_whisper from the frontend instead of here.
            let model_path = "C:/Users/USER/Documents/whisper-models/ggml-small.en.bin";
            // ignore error: if model missing, frontend can call init_whisper instead
            let _ = init_whisper(model_path.to_string());

            // start background transcription worker
            let app_handle = app.handle();
            start_transcription_worker(app_handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
