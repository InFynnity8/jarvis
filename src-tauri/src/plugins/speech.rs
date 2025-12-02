use tauri::command;
use tauri::AppHandle;
use tauri::Emitter;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Load the Whisper model only once globally
pub static WHISPER_CTX: Lazy<WhisperContext> = Lazy::new(|| {
    WhisperContext::new_with_params(
        "C:/Users/USER/Documents/whisper-models/ggml-large-v3-q5_0.bin",
        WhisperContextParameters::default(),
    )
    .expect("failed to load whisper model")
});

#[command]
pub fn transcribe_audio(app_handle: AppHandle) -> Result<(), String> {
    // spawn a background thread
    std::thread::spawn(move || {
        // 1️⃣ Emit status
        let _ = app_handle.emit("recording-status", "Recording started");

        // 2️⃣ Record audio
        let pcm = match record_until_silence(app_handle.clone()) {
            Ok(p) => p,
            Err(e) => {
                let _ = app_handle.emit("recording-status", format!("Recording error: {}", e));
                return;
            }
        };

        let _ = app_handle.emit("recording-status", "Recording stopped. Transcribing...");

        // 3️⃣ Whisper transcription
        let ctx = &*WHISPER_CTX;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of:1 });
        // let mut params = FullParams::new(whisper_rs::SamplingStrategy::BeamSearch {
        //     beam_size: 5,
        //     patience: -1.0,
        // });
        params.set_n_threads(num_cpus::get() as i32);
        // params.set_translate(false);
        // params.set_language(Some("en"));
        // params.set_no_context(true);
        // params.set_single_segment(true);
        // params.set_temperature(0.0);
        // params.set_max_len(1);
        // params.set_token_timestamps(false);
        // params.set_print_special(false);
        // params.set_print_progress(false);
        // params.set_print_realtime(false);
        // params.set_print_timestamps(false);

        let mut state = match ctx.create_state() {
            Ok(s) => s,
            Err(e) => {
                let _ = app_handle.emit("recording-status", format!("Whisper error: {:?}", e));
                return;
            }
        };

        if let Err(e) = state.full(params, &pcm) {
            let _ = app_handle.emit("recording-status", format!("Whisper error: {:?}", e));
            return;
        }

        // 4️⃣ Collect transcription
        let mut result = String::new();
        for seg in state.as_iter() {
            if let Ok(s) = seg.to_str() {
                result.push_str(s);
                result.push(' ');
            } else if let Ok(s) = seg.to_str_lossy() {
                result.push_str(&s);
                result.push(' ');
            }
        }
        println!("Results: {}", result.trim());
        // 5️⃣ Emit final transcription to frontend
        let _ = app_handle.emit("recording-result", result.trim());
    });

    Ok(()) // return immediately
}

pub fn record_until_silence(app_handle: AppHandle) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // 1. Setup CPAL host & device
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0 as usize;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();

    // 2. Buffers and silence detection
    let samples_arc = Arc::new(Mutex::new(Vec::<f32>::new()));
    let samples_clone = samples_arc.clone();

    let last_voice_time = Arc::new(Mutex::new(None::<Instant>));
    let last_voice_time_clone = last_voice_time.clone();

    let silence_threshold = 0.01; // RMS threshold
    let max_silence_duration = Duration::from_millis(1000); // 1s of silence
    let min_recording_duration = Duration::from_millis(500); // 0.5s minimum
    let check_interval_samples = sample_rate / 10; // ~100ms

    // 3. Error callback
    let err_fn = |err| eprintln!("Error on audio input: {:?}", err);

    // Emit initial status
    app_handle.emit("recording-status", "Recording started")?;

    // 4. Build input stream
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                let mut s = samples_clone.lock().unwrap();
                s.extend_from_slice(data);

                if s.len() >= check_interval_samples {
                    let chunk = &s[s.len() - check_interval_samples..];
                    let rms: f32 =
                        (chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
                    if rms > silence_threshold {
                        let mut lv = last_voice_time_clone.lock().unwrap();
                        *lv = Some(Instant::now());
                    }
                }
            },
            err_fn,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let mut s = samples_clone.lock().unwrap();
                s.extend(data.iter().map(|&v| v as f32 / i16::MAX as f32));

                if s.len() >= check_interval_samples {
                    let chunk = &s[s.len() - check_interval_samples..];
                    let rms: f32 =
                        (chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
                    if rms > silence_threshold {
                        let mut lv = last_voice_time_clone.lock().unwrap();
                        *lv = Some(Instant::now());
                    }
                }
            },
            err_fn,
            None,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                let mut s = samples_clone.lock().unwrap();
                s.extend(data.iter().map(|&v| v as f32 / u16::MAX as f32 - 0.5));

                if s.len() >= check_interval_samples {
                    let chunk = &s[s.len() - check_interval_samples..];
                    let rms: f32 =
                        (chunk.iter().map(|x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
                    if rms > silence_threshold {
                        let mut lv = last_voice_time_clone.lock().unwrap();
                        *lv = Some(Instant::now());
                    }
                }
            },
            err_fn,
            None,
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    // 5. Start recording
    stream.play()?;

    let start_time = Instant::now();

    // 6. Wait until silence
    loop {
        std::thread::sleep(Duration::from_millis(50));

        let lv = last_voice_time.lock().unwrap();
        let elapsed_since_last_voice = lv.map_or(Duration::from_millis(0), |v| v.elapsed());

        if elapsed_since_last_voice >= max_silence_duration
            && start_time.elapsed() >= min_recording_duration
        {
            break;
        }

        // Optional: emit periodic RMS info (or other status)
        if let Some(last_voice) = *lv {
            let elapsed_ms = last_voice.elapsed().as_millis();
            let status_msg = format!("Speaking... silence elapsed: {}ms", elapsed_ms);
            let _ = app_handle.emit("recording-status", status_msg);
        }
    }

    drop(stream); // stop recording
    app_handle.emit("recording-status", "Recording stopped")?;

    // 7. Retrieve samples and convert to mono
    let mut pcm = samples_arc.lock().unwrap().clone();
    if channels > 1 {
        pcm = pcm.chunks(channels).map(|frame| frame[0]).collect();
    }

    Ok(pcm)
}
