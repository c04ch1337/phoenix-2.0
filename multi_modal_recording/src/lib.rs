//! Multi-modal (audio + video) recording for Phoenix.
//!
//! This crate is intentionally **feature-gated**:
//! - Default build uses stub implementations so the workspace compiles without native deps.
//! - Enable real capture/recognition with features:
//!   - `audio` => [`cpal`](https://crates.io/crates/cpal) + [`rodio`](https://crates.io/crates/rodio)
//!   - `video` => [`nokhwa`](https://crates.io/crates/nokhwa)
//!   - `speech-vosk` / `speech-whisper` => [`vosk`](https://crates.io/crates/vosk) / [`whisper-rs`](https://crates.io/crates/whisper-rs)
//!   - `face-rustface` / `face-dlib` => [`rustface`](https://crates.io/crates/rustface) / [`dlib-face-recognition`](https://crates.io/crates/dlib-face-recognition)

use chrono::Utc;
use emotion_detection::{EmotionalState, EmotionDetector};
use image::DynamicImage;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::Mutex;
use vital_organ_vaults::VitalOrganVaults;
use multi_modal_input::LiveMultiModalInput;

/// Image type used by [`MultiModalRecorder::recognize_user()`](crate::MultiModalRecorder::recognize_user).
pub type Image = DynamicImage;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("feature not enabled: {0}")]
    FeatureDisabled(&'static str),
}

/// Recognition confidence values for the enrolled user.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecognitionConfidence {
    pub voice: f32,
    pub face: f32,
    pub combined: f32,
    pub recognized: bool,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RecordingMeta {
    created_unix: i64,
    duration_secs: u64,
    audio_enabled: bool,
    video_enabled: bool,
    purpose: Option<String>,
    wake_word: String,
}

/// Multi-modal audio/video recording + user recognition.
///
/// **Note:** the public fields are part of the requested API surface.
#[derive(Clone)]
pub struct MultiModalRecorder {
    pub audio_enabled: bool,
    pub video_enabled: bool,
    pub always_listening: bool,
    pub wake_word: String,
    pub user_voice_model: Option<PathBuf>,
    pub user_face_model: Option<PathBuf>,

    // Internal state
    storage_path: PathBuf,
    last_recording: Arc<Mutex<Option<PathBuf>>>,
    listening_stop: Arc<AtomicBool>,

    // Live streaming mode (capture-only; no identification).
    live_stop: Arc<AtomicBool>,
    live_running: Arc<AtomicBool>,

    // Emotion detection + persistence hooks
    emotion_detector: EmotionDetector,
    last_emotional_state: Arc<Mutex<Option<EmotionalState>>>,
    vaults: Option<Arc<VitalOrganVaults>>,
}

impl std::fmt::Debug for MultiModalRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiModalRecorder")
            .field("audio_enabled", &self.audio_enabled)
            .field("video_enabled", &self.video_enabled)
            .field("always_listening", &self.always_listening)
            .field("wake_word", &self.wake_word)
            .field("user_voice_model", &self.user_voice_model)
            .field("user_face_model", &self.user_face_model)
            .field("storage_path", &self.storage_path)
            .finish_non_exhaustive()
    }
}

impl Default for MultiModalRecorder {
    fn default() -> Self {
        Self::from_env()
    }
}

impl MultiModalRecorder {
    /// Build a recorder from `.env` / environment variables.
    ///
    /// Reads:
    /// - `MULTI_MODAL_ENABLED`
    /// - `ALWAYS_LISTENING_ENABLED`
    /// - `WAKE_WORD`
    /// - `RECORDING_STORAGE_PATH`
    pub fn from_env() -> Self {
        let audio_enabled = std::env::var("MULTI_MODAL_ENABLED")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);
        let video_enabled = std::env::var("MULTI_MODAL_ENABLED")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);
        let always_listening = std::env::var("ALWAYS_LISTENING_ENABLED")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        let wake_word = std::env::var("WAKE_WORD").unwrap_or_else(|_| "Phoenix".to_string());
        let storage_path = std::env::var("RECORDING_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data/recordings/encrypted"));

        Self {
            audio_enabled,
            video_enabled,
            always_listening,
            wake_word,
            user_voice_model: None,
            user_face_model: None,
            storage_path,
            last_recording: Arc::new(Mutex::new(None)),
            listening_stop: Arc::new(AtomicBool::new(false)),

            live_stop: Arc::new(AtomicBool::new(false)),
            live_running: Arc::new(AtomicBool::new(false)),

            emotion_detector: EmotionDetector::from_env(),
            last_emotional_state: Arc::new(Mutex::new(None)),
            vaults: None,
        }
    }

    /// Attach an existing Soul Vault handle so recordings can log emotional moments without
    /// creating multi-open DB conflicts.
    pub fn attach_vaults(&mut self, vaults: Arc<VitalOrganVaults>) {
        self.vaults = Some(vaults);
    }

    /// Retrieve the most recently computed emotional state (if any).
    pub async fn last_emotion(&self) -> Option<EmotionalState> {
        self.last_emotional_state.lock().await.clone()
    }

    /// Best-effort read of the Soul-Vault emotion timeline (most recent last).
    pub fn emotional_moments_recent(&self, max: usize) -> Vec<String> {
        let Some(vaults) = self.vaults.as_ref() else {
            return Vec::new();
        };
        let raw = vaults.recall_soul("emotional_moments").unwrap_or_default();
        let mut lines = raw
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>();
        if max == 0 {
            return Vec::new();
        }
        if lines.len() > max {
            lines = lines.split_off(lines.len() - max);
        }
        lines
    }

    /// Convenience: clone this recorder but override audio/video enable flags.
    pub fn clone_with_modes(&self, audio_enabled: bool, video_enabled: bool) -> Self {
        let mut out = self.clone();
        out.audio_enabled = audio_enabled;
        out.video_enabled = video_enabled;
        out
    }

    /// Record audio+video on demand, save encrypted, return path.
    ///
    /// Current implementation:
    /// - Always writes an encrypted `.phoenixrec` bundle containing:
    ///   - JSON metadata
    ///   - placeholder payload bytes
    ///
    /// When features are enabled, the placeholder payload is where captured frames/samples
    /// should be serialized (container format TBD: e.g. Matroska/WebM).
    pub async fn start_on_demand(&self, duration_secs: u64) -> Result<PathBuf, Error> {
        if duration_secs == 0 {
            return Err(Error::InvalidArgument("duration_secs must be > 0".to_string()));
        }

        tokio::fs::create_dir_all(&self.storage_path).await?;

        let ts = Utc::now().timestamp();
        let id = uuid::Uuid::new_v4().to_string();
        let filename = format!("REC-{ts}-{id}.phoenixrec");
        let out_path = self.storage_path.join(filename);

        // TODO(real capture):
        // - audio: cpal input stream -> samples -> encode (wav/opus)
        // - video: nokhwa frames -> encode
        // - mux into a single container
        let meta = RecordingMeta {
            created_unix: ts,
            duration_secs,
            audio_enabled: self.audio_enabled,
            video_enabled: self.video_enabled,
            purpose: None,
            wake_word: self.wake_word.clone(),
        };

        let meta_json = serde_json::to_vec(&meta).unwrap_or_default();

        // Placeholder payload: random bytes sized to duration (tiny).
        let mut payload = vec![0u8; (duration_secs.min(300) as usize) * 256];
        rand::thread_rng().fill_bytes(&mut payload);

        let mut bundle = Vec::with_capacity(16 + meta_json.len() + payload.len());
        bundle.extend_from_slice(b"PHXREC\0\0");
        bundle.extend_from_slice(&(meta_json.len() as u32).to_le_bytes());
        bundle.extend_from_slice(&meta_json);
        bundle.extend_from_slice(&payload);

        let encrypted = xor_encrypt(&bundle, &derive_key_from_env());
        tokio::fs::write(&out_path, encrypted).await?;

        *self.last_recording.lock().await = Some(out_path.clone());

        // Emotion fusion (best-effort). For now we treat the encrypted recording path as an
        // audio hint for the heuristic backend.
        let state = self
            .emotion_detector
            .fused_emotional_state("", Some(out_path.clone()), None)
            .await;
        *self.last_emotional_state.lock().await = Some(state.clone());
        self.append_emotional_moment_best_effort(&state, &out_path);

        Ok(out_path)
    }

    /// Schedule a recurring recording.
    ///
    /// This spawns a background Tokio task. The `cron_expr` uses the [`cron`](https://crates.io/crates/cron)
    /// crate format (supports seconds).
    pub async fn schedule_recording(&self, cron_expr: &str, purpose: &str) {
        let expr = cron_expr.trim().to_string();
        let purpose = purpose.trim().to_string();
        let this = self.clone();

        tokio::spawn(async move {
            let schedule = match expr.parse::<cron::Schedule>() {
                Ok(s) => s,
                Err(_) => return,
            };

            loop {
                let now = chrono::Utc::now();
                let Some(next) = schedule.after(&now).next() else {
                    return;
                };
                let Ok(dur) = next.signed_duration_since(now).to_std() else {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                };
                tokio::time::sleep(dur).await;
                let p = this.start_on_demand(30).await.ok();

                // If we have a purpose, fuse it as text context too.
                if let Some(path) = p {
                    let state = this
                        .emotion_detector
                        .fused_emotional_state(&purpose, Some(path.clone()), None)
                        .await;
                    *this.last_emotional_state.lock().await = Some(state.clone());
                    this.append_emotional_moment_best_effort(&state, &path);
                }

                // Persist last purpose (best-effort) into a sidecar file.
                let _ = tokio::fs::write(
                    this.storage_path.join(".last_schedule_purpose"),
                    purpose.as_bytes(),
                )
                .await;
            }
        });
    }

    /// Start always-listening mode.
    ///
    /// This spawns a background Tokio task that (when fully implemented) will:
    /// - continuously capture a low-power audio stream
    /// - run wake-word detection (Vosk/Whisper backends)
    /// - optionally run speaker ID (voiceprint)
    /// - optionally trigger video capture for face recognition
    pub async fn start_always_listening(&self) {
        self.listening_stop.store(false, Ordering::Relaxed);
        let stop = self.listening_stop.clone();
        let wake = self.wake_word.clone();
        let this = self.clone();

        tokio::spawn(async move {
            // Placeholder loop.
            while !stop.load(Ordering::Relaxed) {
                // TODO(real impl): wire wake-word engine here.
                // If detected:
                // - optional recognition
                // - optional start_on_demand short clip
                let _ = &wake;
                let _ = &this;
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        });
    }

    /// Start live streaming mode (continuous capture).
    ///
    /// This is **capture-only** plumbing. It does not perform face/voice identification.
    ///
    /// Enable backends via crate features:
    /// - `multi_modal_recording/audio`
    /// - `multi_modal_recording/video`
    pub async fn start_live_streaming(&self) -> Result<(), Error> {
        let mut cfg = LiveMultiModalInput::from_env();
        cfg.microphone_enabled = cfg.microphone_enabled && self.audio_enabled;
        cfg.webcam_enabled = cfg.webcam_enabled && self.video_enabled;

        if !cfg.microphone_enabled && !cfg.webcam_enabled {
            return Err(Error::InvalidArgument(
                "live streaming requested but both microphone and webcam are disabled".to_string(),
            ));
        }

        // Validate compile-time feature gates up-front so we can return a typed error.
        if cfg.microphone_enabled && !cfg!(feature = "audio") {
            return Err(Error::FeatureDisabled("audio"));
        }
        if cfg.webcam_enabled && !cfg!(feature = "video") {
            return Err(Error::FeatureDisabled("video"));
        }

        self.live_stop.store(false, Ordering::Relaxed);
        self.live_running.store(true, Ordering::Relaxed);

        let stop = self.live_stop.clone();
        let running = self.live_running.clone();
        let this = self.clone();
        tokio::spawn(async move {
            // When built without `video`, the live-loop is capture-only and won't use `this`.
            #[cfg(not(feature = "video"))]
            let _ = &this;

            // Keep the streams alive for the duration of this loop.
            let audio = if cfg.microphone_enabled {
                cfg.start_audio_stream().await.ok()
            } else {
                None
            };
            let video = if cfg.webcam_enabled {
                cfg.start_webcam_stream().await.ok()
            } else {
                None
            };

            // If both requested streams failed to start, exit.
            if cfg.microphone_enabled && audio.is_none() && cfg.webcam_enabled && video.is_none() {
                running.store(false, Ordering::Relaxed);
                return;
            }

            // If we have a camera, try to open the stream before entering the loop.
            #[cfg(feature = "video")]
            let mut video = video;
            #[cfg(feature = "video")]
            if let Some(vs) = video.as_mut() {
                if let Err(e) = vs.camera.open_stream() {
                    eprintln!("[multi_modal_recording] failed to open webcam stream: {e}");
                }
            }

            #[cfg(not(feature = "video"))]
            let video = video;

            while !stop.load(Ordering::Relaxed) {
                // Video -> emotion (best-effort)
            #[cfg(feature = "video")]
            if let Some(vs) = video.as_ref() {
                    use nokhwa::pixel_format::RgbFormat;

                    match vs.camera.frame() {
                        Ok(buffer) => {
                            match buffer.decode_image::<RgbFormat>() {
                                Ok(rgb) => {
                                    let mut state = this
                                        .emotion_detector
                                        .fused_emotional_state("", None, Some(rgb.clone()))
                                        .await;

                                    *this.last_emotional_state.lock().await = Some(state.clone());
                                    this.append_emotional_moment_best_effort(
                                        &state,
                                        Path::new("(live-stream)"),
                                    );
                                }
                                Err(e) => {
                                    eprintln!("[multi_modal_recording] decode_image failed: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[multi_modal_recording] webcam frame capture failed: {e}");
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }

            drop(audio);
            drop(video);
            running.store(false, Ordering::Relaxed);
        });

        Ok(())
    }

    /// Stop live streaming mode.
    pub fn stop_live_streaming(&self) {
        self.live_stop.store(true, Ordering::Relaxed);
    }

    /// Best-effort flag for UI/status panels.
    pub fn live_streaming_active(&self) -> bool {
        self.live_running.load(Ordering::Relaxed)
    }

    /// Stop always-listening background loop (privacy command).
    pub fn stop_listening(&self) {
        self.listening_stop.store(true, Ordering::Relaxed);
    }

    /// Train / enroll a speaker identification model.
    ///
    /// Current behavior: stores sample list and creates a placeholder model file.
    pub fn enroll_user_voice(&mut self, samples: Vec<PathBuf>) -> Result<(), Error> {
        if samples.is_empty() {
            return Err(Error::InvalidArgument(
                "enroll_user_voice requires at least one sample".to_string(),
            ));
        }
        let model_dir = self
            .storage_path
            .join("..")
            .join("..")
            .join("models")
            .join("voice");
        std::fs::create_dir_all(&model_dir)?;
        let model_path = model_dir.join("user_voice.model.json");

        let data = serde_json::json!({
            "created_unix": Utc::now().timestamp(),
            "samples": samples,
            "backend": if cfg!(feature = "speech-vosk") {
                "vosk"
            } else if cfg!(feature = "speech-whisper") {
                "whisper-rs"
            } else {
                "stub"
            }
        });
        std::fs::write(&model_path, serde_json::to_vec_pretty(&data).unwrap_or_default())?;
        self.user_voice_model = Some(model_path);
        Ok(())
    }

    /// Train / enroll a face identification model.
    ///
    /// Current behavior: stores image list and creates a placeholder model file.
    pub fn enroll_user_face(&mut self, images: Vec<PathBuf>) -> Result<(), Error> {
        if images.is_empty() {
            return Err(Error::InvalidArgument(
                "enroll_user_face requires at least one image".to_string(),
            ));
        }
        let model_dir = self
            .storage_path
            .join("..")
            .join("..")
            .join("models")
            .join("face");
        std::fs::create_dir_all(&model_dir)?;
        let model_path = model_dir.join("user_face.model.json");

        let data = serde_json::json!({
            "created_unix": Utc::now().timestamp(),
            "images": images,
            "backend": if cfg!(feature = "face-dlib") {
                "dlib-face-recognition"
            } else if cfg!(feature = "face-rustface") {
                "rustface"
            } else {
                "stub"
            }
        });
        std::fs::write(&model_path, serde_json::to_vec_pretty(&data).unwrap_or_default())?;
        self.user_face_model = Some(model_path);
        Ok(())
    }

    /// Recognize the enrolled user from an audio sample + video frame.
    ///
    /// Current behavior:
    /// - if a model is enrolled, returns high confidence
    /// - otherwise returns low confidence
    pub fn recognize_user(&self, _audio_sample: &[f32], _video_frame: &Image) -> RecognitionConfidence {
        let voice: f32 = if self.user_voice_model.is_some() { 0.92_f32 } else { 0.10_f32 };
        let face: f32 = if self.user_face_model.is_some() { 0.93_f32 } else { 0.10_f32 };
        let combined: f32 = (voice * 0.5_f32 + face * 0.5_f32).clamp(0.0_f32, 1.0_f32);
        RecognitionConfidence {
            voice,
            face,
            combined,
            recognized: combined >= 0.80,
            label: if combined >= 0.80 {
                Some("Dad".to_string())
            } else {
                None
            },
        }
    }

    /// Delete the last on-disk recording created by this process (privacy command).
    pub async fn delete_last_recording(&self) -> Result<bool, Error> {
        let path = self.last_recording.lock().await.clone();
        let Some(p) = path else {
            return Ok(false);
        };
        if tokio::fs::try_exists(&p).await.unwrap_or(false) {
            tokio::fs::remove_file(&p).await?;
        }
        *self.last_recording.lock().await = None;
        Ok(true)
    }

    /// Clear all encrypted recordings in the configured storage directory (privacy command).
    pub async fn clear_all_recordings(&self) -> Result<u64, Error> {
        let mut removed = 0u64;
        if !tokio::fs::try_exists(&self.storage_path).await.unwrap_or(false) {
            return Ok(0);
        }
        let mut rd = tokio::fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = rd.next_entry().await? {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("phoenixrec") {
                let _ = tokio::fs::remove_file(&p).await;
                removed += 1;
            }
        }
        *self.last_recording.lock().await = None;
        Ok(removed)
    }
}

impl MultiModalRecorder {
    fn append_emotional_moment_best_effort(&self, state: &EmotionalState, recording_path: &Path) {
        let Some(vaults) = self.vaults.as_ref() else {
            return;
        };

        let entry = serde_json::json!({
            "ts_unix": state.timestamp.timestamp(),
            "emotion": format!("{:?}", state.primary_emotion),
            "intensity": state.intensity,
            "confidence": state.confidence,
            "voice_contribution": state.voice_contribution,
            "face_contribution": state.face_contribution,
            "text_contribution": state.text_contribution,
            "recording": recording_path.display().to_string(),
        })
        .to_string();

        let existing = vaults.recall_soul("emotional_moments").unwrap_or_default();
        let mut lines = existing
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>();
        lines.push(entry);
        if lines.len() > 200 {
            lines = lines.split_off(lines.len() - 200);
        }
        let updated = lines.join("\n");
        let _ = vaults.store_soul("emotional_moments", &updated);
    }
}

fn derive_key_from_env() -> Vec<u8> {
    let seed = std::env::var("SOUL_ENCRYPTION_KEY")
        .unwrap_or_else(|_| "phoenix-eternal-soul-key".to_string());
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.finalize().to_vec()
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

#[allow(dead_code)]
fn is_file(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}

