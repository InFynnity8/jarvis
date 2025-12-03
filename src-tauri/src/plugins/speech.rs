use anyhow::Result;
use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::Duration;
use tauri::AppHandle;
use tauri::{command, Emitter};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// --- Configuration ---
const SAMPLE_RATE: usize = 16000;
const CHUNK_SECONDS: f32 = 1.0;
const TRIGGER_SAMPLES: usize = (SAMPLE_RATE as f32 * CHUNK_SECONDS) as usize;

// --- Global state ---
static WHISPER_CTX_OPT: Lazy<Mutex<Option<WhisperContext>>> = Lazy::new(|| Mutex::new(None));
static AUDIO_BUFFER: Lazy<Mutex<Vec<f32>>> = Lazy::new(|| Mutex::new(Vec::new()));
static HAS_NEW_AUDIO: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Initialize/load the Whisper model once. Call this at app start.
#[command]
pub fn init_whisper(model_path: String) -> Result<(), String> {
    let model_path = PathBuf::from(model_path);
    if !model_path.exists() {
        return Err(format!("Model path does not exist: {:?}", model_path));
    }

    let mut guard = WHISPER_CTX_OPT.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let params = WhisperContextParameters::default();
    let ctx = WhisperContext::new_with_params(model_path.to_str().unwrap(), params)
        .map_err(|e| format!("Failed to create WhisperContext: {:?}", e))?;

    *guard = Some(ctx);
    println!("Whisper initialized from {:?}", model_path);
    Ok(())
}

/// Push raw PCM16LE chunk (Uint8 bytes) from the frontend.
/// IMPORTANT: frontend MUST send PCM16LE (mono) bytes.
/// Each pair of bytes = one i16 sample (little-endian).
#[command]
pub fn push_pcm_chunk(chunk: Vec<u8>) -> Result<(), String> {
    if chunk.len() % 2 != 0 {
        return Err("Chunk length not divisible by 2".into());
    }

    let mut samples_f32 = Vec::with_capacity(chunk.len() / 2);
    let mut i = 0usize;
    while i + 1 < chunk.len() {
        let lo = chunk[i] as u16;
        let hi = chunk[i + 1] as u16;
        let sample = ((hi << 8) | lo) as i16;
        let s_f32 = sample as f32 / 32768.0;
        samples_f32.push(s_f32);
        i += 2;
    }

    {
        let mut buf = AUDIO_BUFFER.lock().unwrap();
        buf.extend_from_slice(&samples_f32);
    }

    let buf_len = AUDIO_BUFFER.lock().unwrap().len();
    if buf_len >= TRIGGER_SAMPLES {
        HAS_NEW_AUDIO.store(true, Ordering::SeqCst);
    }

    Ok(())
}

/// Spawn a background worker thread that continuously watches HAS_NEW_AUDIO.
/// The worker will run transcription on snapshots and emit events.
/// Call this once from main on startup: start_transcription_worker(app.handle()).
pub fn start_transcription_worker(app_handle: AppHandle) {
    let app = app_handle.clone();
    std::thread::spawn(move || {
        loop {
            if HAS_NEW_AUDIO.swap(false, Ordering::SeqCst) {
                // Snapshot and clear buffer
                let audio_snapshot: Vec<f32> = {
                    let mut buf = AUDIO_BUFFER.lock().unwrap();
                    let snap = buf.clone();
                    buf.clear();
                    snap
                };

                if audio_snapshot.is_empty() {
                    continue;
                }

                // Do heavy transcription work in this worker thread
                let transcript = run_whisper_sync(&audio_snapshot);
                match transcript {
                    Ok(text) => {
                        let _ = app.emit("transcript", text.clone());
                        // Mock LLM reply — replace with your LLM call if you have one
                        let llm_text = format!("Echo: {}", text);
                        let _ = app.emit("llm-response", llm_text.clone());
                        // speak
                        
                        // Spawn TTS in a new thread
                        // let llm_text_clone = llm_text.clone();
                        // std::thread::spawn(move || {
                        //     if let Err(e) = play_tts_sync(&llm_text_clone) {
                        //         eprintln!("play_tts error: {}", e);
                        //     }
                        // });
                    }
                    Err(err) => {
                        eprintln!("Transcription error: {}", err);
                        let _ = app.emit("transcript-error", err);
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    });
}

/// Synchronous helper that runs whisper full() on PCM f32 samples.
fn run_whisper_sync(audio_snapshot: &[f32]) -> Result<String, String> {
    let guard = WHISPER_CTX_OPT.lock().unwrap();
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Whisper not initialized".to_string())?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(num_cpus::get() as i32);
    // params.set_language(Some("en"));

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("create_state failed: {:?}", e))?;
    state
        .full(params, audio_snapshot)
        .map_err(|e| format!("full() failed: {:?}", e))?;

    let mut text = String::new();
    for seg in state.as_iter() {
        if let Ok(s) = seg.to_str() {
            text.push_str(s);
            text.push(' ');
        } else if let Ok(s) = seg.to_str_lossy() {
            text.push_str(&s);
            text.push(' ');
        }
    }
    Ok(text.trim().to_string())
}

/// Synchronous TTS: call Python helper to create WAV and play it using rodio.
/// This blocks the worker thread while the WAV is being synthesized and played.
fn play_tts_sync(text: &str) -> Result<(), String> {
    // ensure output dir exists
    let output_path = PathBuf::from("src/models/output.wav");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create folder: {:?}", e))?;
    }

    // call python script (adjust path if needed)
    let status = Command::new("python")
        .arg("src/tts.py")
        .arg(text)
        .arg(output_path.to_str().unwrap())
        .status()
        .map_err(|e| format!("Failed to spawn python: {:?}", e))?;

    if !status.success() {
        return Err("Python TTS failed".into());
    }

    // play the WAV
    let stream_handle = OutputStreamBuilder::open_default_stream()
        .map_err(|e| format!("Failed to open audio stream: {:?}", e))?;
    let sink = Sink::connect_new(&stream_handle.mixer());

    let file = File::open(output_path).map_err(|e| format!("Failed to open wav: {:?}", e))?;
    let source =
        Decoder::new(BufReader::new(file)).map_err(|e| format!("Failed decode wav: {:?}", e))?;
    sink.append(source);
    // block until finished or detach depending on behavior you want
    sink.sleep_until_end();

    Ok(())
}
