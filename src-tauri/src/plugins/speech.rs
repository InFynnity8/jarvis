use tauri::command;

#[command]
pub fn transcribe_audio() -> String {
    // later: integrate whisper-rs
    "Audio transcription not implemented yet.".into()
}
