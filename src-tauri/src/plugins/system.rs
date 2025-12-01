use tauri::command;

#[command]
pub fn get_system_info() -> String {
    // later: CPU, RAM, uptime, GPU, battery, OS details
    "System info placeholder.".into()
}
