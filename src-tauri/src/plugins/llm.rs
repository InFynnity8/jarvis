use tauri::command;

#[command]
pub fn generate_response(prompt: &str) -> String {
    // later: integrate llama.cpp or ggml-rs 
    format!("(AI Placeholder Response) You said: {}", prompt)
}
