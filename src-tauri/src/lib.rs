// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod plugins;

use plugins::{
    llm::*,
    system::*,
    speech::*,
    automation::*,
    screen_reader::*
};




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
            transcribe_audio,
            play_tts,  
            generate_response,
            get_system_info
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
