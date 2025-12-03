use anyhow::Result;
use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use tauri::{command, Emitter};
use tokio::task;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub static WHISPER_CTX: Lazy<WhisperContext> = Lazy::new(|| {
    WhisperContext::new_with_params(
        "C:/Users/USER/Documents/whisper-models/ggml-large-v3-q5_0.bin",
        WhisperContextParameters::default(),
    )
    .expect("Failed to load Whisper model")
});

#[command]
pub async fn transcribe_audio(chunk: Vec<u8>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let pcm: Vec<f32> = chunk.iter().map(|b| *b as f32 / 32768.0).collect();

    // Spawn task for Whisper transcription
    let app_clone = app_handle.clone();
    task::spawn_blocking(move || {
        let ctx = &*WHISPER_CTX;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus::get() as i32);

        if let Ok(mut state) = ctx.create_state() {
            if state.full(params, &pcm).is_ok() {
                let mut transcript = String::new();
                for seg in state.as_iter() {
                    if let Ok(s) = seg.to_str() {
                        transcript.push_str(s);
                        transcript.push(' ');
                    }
                }
                let _ = app_clone.emit("transcript", transcript.clone());

                // Generate LLM response (mock for now)
                let llm_text = format!("Echo: {}", transcript);
                let _ = app_clone.emit("llm-response", llm_text.clone());

                // Play TTS
                play_tts(llm_text);
            }
        }
    });

    Ok(())
}

#[command]
pub fn play_tts(text: String) -> Result<(), String> {
    // 1️⃣ Output path for TTS WAV
    let output_path = Path::new("src/models/output.wav");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create folder: {:?}", e))?;
    }
    // 2️⃣ Call Python TTS script
    let status = Command::new("python")
        .arg("C:/Users/USER/Desktop/jarvis/src-tauri/src/tts.py")
        .arg(text)
        .arg(output_path)
        .status()
        .map_err(|e| format!("failed: {:?}", e))?;

    if !status.success() {
        return Err("Python TTS failed".to_string());
    }

    // 3️⃣ Play WAV using rodio
    let stream_handle =
        OutputStreamBuilder::open_default_stream().expect("Failed to open default audio stream");

    let sink = Sink::connect_new(&stream_handle.mixer());

    let file = File::open(output_path).map_err(|e| format!("failed: {:?}", e))?;
    let source = Decoder::new(BufReader::new(file)).map_err(|e| format!("failed: {:?}", e))?;
    sink.append(source);

    // Optional: detach so playback continues without blocking
    sink.detach();

    Ok(())
}
