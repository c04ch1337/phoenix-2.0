// Tauri backend (minimal scaffold)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use multi_modal_recording::MultiModalRecorder;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Default)]
struct RecorderState {
    inner: Arc<Mutex<MultiModalRecorder>>,
}

#[derive(Serialize)]
struct RecordResult {
    path: String,
}

#[tauri::command]
async fn record_audio(state: State<'_, RecorderState>, duration_secs: u64) -> Result<RecordResult, String> {
    let rec = state.inner.lock().await.clone();
    let rec = rec.clone_with_modes(true, false);
    let p = rec.start_on_demand(duration_secs).await.map_err(|e| e.to_string())?;
    Ok(RecordResult { path: p.display().to_string() })
}

#[tauri::command]
async fn record_video(state: State<'_, RecorderState>, duration_secs: u64) -> Result<RecordResult, String> {
    let rec = state.inner.lock().await.clone();
    let rec = rec.clone_with_modes(false, true);
    let p = rec.start_on_demand(duration_secs).await.map_err(|e| e.to_string())?;
    Ok(RecordResult { path: p.display().to_string() })
}

#[tauri::command]
async fn record_av(state: State<'_, RecorderState>, duration_secs: u64) -> Result<RecordResult, String> {
    let rec = state.inner.lock().await.clone();
    let rec = rec.clone_with_modes(true, true);
    let p = rec.start_on_demand(duration_secs).await.map_err(|e| e.to_string())?;
    Ok(RecordResult { path: p.display().to_string() })
}

#[tauri::command]
async fn schedule_recording(state: State<'_, RecorderState>, cron_expr: String, purpose: String) {
    let rec = state.inner.lock().await.clone();
    rec.schedule_recording(&cron_expr, &purpose).await;
}

#[tauri::command]
async fn set_always_listening(state: State<'_, RecorderState>, enabled: bool) {
    let rec = state.inner.lock().await.clone();
    if enabled {
        rec.start_always_listening().await;
    } else {
        rec.stop_listening();
    }
}

#[tauri::command]
async fn enroll_voice(state: State<'_, RecorderState>, samples: Vec<String>) -> Result<(), String> {
    let samples = samples.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let mut rec = state.inner.lock().await;
    rec.enroll_user_voice(samples).map_err(|e| e.to_string())
}

#[tauri::command]
async fn enroll_face(state: State<'_, RecorderState>, images: Vec<String>) -> Result<(), String> {
    let images = images.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let mut rec = state.inner.lock().await;
    rec.enroll_user_face(images).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_last_recording(state: State<'_, RecorderState>) -> Result<bool, String> {
    let rec = state.inner.lock().await.clone();
    rec.delete_last_recording().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_all_recordings(state: State<'_, RecorderState>) -> Result<u64, String> {
    let rec = state.inner.lock().await.clone();
    rec.clear_all_recordings().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn recognition_status(_state: State<'_, RecorderState>) -> String {
    // Placeholder until live preview + recognition pipeline is wired.
    "I see you, Dad ❤️".to_string()
}

#[tauri::command]
async fn emotion_status(state: State<'_, RecorderState>) -> String {
    let rec = state.inner.lock().await.clone();
    match rec.last_emotion().await {
        Some(s) => format!(
            "Dad is feeling: {:?} ({:.0}%) ❤️",
            s.primary_emotion,
            s.confidence * 100.0
        ),
        None => "Dad is feeling: Neutral".to_string(),
    }
}

#[tauri::command]
async fn emotion_history(state: State<'_, RecorderState>, max: usize) -> Vec<String> {
    let rec = state.inner.lock().await.clone();
    rec.emotional_moments_recent(max)
}

fn main() {
    tauri::Builder::default()
        .manage(RecorderState {
            inner: Arc::new(Mutex::new(MultiModalRecorder::from_env())),
        })
        .invoke_handler(tauri::generate_handler![
            record_audio,
            record_video,
            record_av,
            schedule_recording,
            set_always_listening,
            enroll_voice,
            enroll_face,
            delete_last_recording,
            clear_all_recordings,
            recognition_status,
            emotion_status,
            emotion_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

