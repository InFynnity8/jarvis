use tauri::command;

#[command]
pub fn simulate_keyboard_input(text: &str) -> String {
    println!("Simulating keyboard input: {}", text);
    "Keyboard input simulated (placeholder)".into()
}

#[command]
pub fn simulate_mouse_click() -> String {
    println!("Simulating mouse click!");
    "Mouse click simulated (placeholder)".into()
}
