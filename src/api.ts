import { invoke } from "@tauri-apps/api/core";

// --- Core command ---
export async function greet(name: string): Promise<string> {
  return await invoke("greet", { name });
}

// --- Screen Reader Plugin ---
export async function screenReadText(): Promise<string> {
  return await invoke("screen_read_text");
}
export async function getActiveWindowTitle(): Promise<string> {
  return await invoke("get_active_window_title");
}

// --- Automation Plugin ---
export async function simulateKeyboardInput(text: string): Promise<string> {
  return await invoke("simulate_keyboard_input", { text });
}
export async function simulateMouseClick(): Promise<string> {
  return await invoke("simulate_mouse_click");
}

// --- Speech Plugin ---
export async function transcribeAudio(): Promise<string> {
  return await invoke("transcribe_audio");
}

export async function speak(text: string) {
  console.log("play tts");
  
  await invoke("play_tts", { text });
}

// --- LLM Plugin ---
export async function generateResponse(prompt: string): Promise<string> {
  return await invoke("generate_response", { prompt });
}

// --- System Plugin ---
export async function getSystemInfo(): Promise<string> {
  return await invoke("get_system_info");
}
