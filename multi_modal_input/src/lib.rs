//! Live multi-modal input (webcam + microphone) for Phoenix.
//!
//! This crate intentionally avoids any biometric identification (face/voice recognition).
//! It only provides **capture plumbing** and leaves higher-level interpretation to callers.

use thiserror::Error;

// Image types are only available when a feature that pulls in `image` is enabled.
// Keep the public API compiling even when `image` is not present.
#[cfg(any(
    feature = "face",
    feature = "emotion",
    // (native vision backends removed)
))]
use image::RgbImage;

#[cfg(not(any(
    feature = "face",
    feature = "emotion",
    // (native vision backends removed)
)))]
pub struct RgbImage;

#[derive(Debug, Error)]
pub enum LiveInputError {
    #[error("feature not enabled: {0}")]
    FeatureDisabled(&'static str),

    #[error("input disabled: {0}")]
    Disabled(&'static str),

    #[error("no microphone device available")]
    NoMicrophone,

    #[error("audio backend error: {0}")]
    AudioBackend(String),

    #[error("webcam backend error: {0}")]
    WebcamBackend(String),

    #[error("face recognition error: {0}")]
    FaceRecognitionError(String),

    #[error("voice recognition error: {0}")]
    VoiceRecognitionError(String),

    #[error("emotion analysis error: {0}")]
    EmotionAnalysisError(String),
}

#[derive(Debug, Clone)]
pub enum DetectedEmotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Neutral,
    /// Special: warmth/affection.
    Love,
}

/// Opaque handle to a running webcam capture backend.
///
/// The stream remains active as long as this value is kept alive.
pub struct VideoStream {
    #[cfg(feature = "video")]
    pub camera: nokhwa::Camera,

    #[cfg(not(feature = "video"))]
    _private: (),
}

/// Opaque handle to a running microphone capture backend.
///
/// The stream remains active as long as this value is kept alive.
pub struct AudioStream {
    #[cfg(feature = "audio")]
    pub stream: cpal::Stream,

    #[cfg(not(feature = "audio"))]
    _private: (),
}

/// Capture-only live input configuration.
///
/// **Note:** `face_recognition_enabled` / `voice_recognition_enabled` are kept for config
/// compatibility, but this crate does not implement biometric recognition.
#[derive(Debug, Clone)]
pub struct LiveMultiModalInput {
    pub webcam_enabled: bool,
    pub microphone_enabled: bool,
    pub face_recognition_enabled: bool,
    pub voice_recognition_enabled: bool,
    pub wake_word: String,
}

impl Default for LiveMultiModalInput {
    fn default() -> Self {
        Self::from_env()
    }
}

impl LiveMultiModalInput {
    /// Build configuration from environment variables.
    ///
    /// - `WEBCAM_ENABLED`
    /// - `MICROPHONE_ENABLED`
    /// - `FACE_RECOGNITION_ENABLED` (accepted, but not implemented here)
    /// - `VOICE_RECOGNITION_ENABLED` (accepted, but not implemented here)
    /// - `WAKE_WORD`
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            webcam_enabled: env_bool("WEBCAM_ENABLED").unwrap_or(false),
            microphone_enabled: env_bool("MICROPHONE_ENABLED").unwrap_or(false),
            face_recognition_enabled: env_bool("FACE_RECOGNITION_ENABLED").unwrap_or(false),
            voice_recognition_enabled: env_bool("VOICE_RECOGNITION_ENABLED").unwrap_or(false),
            wake_word: std::env::var("WAKE_WORD").unwrap_or_else(|_| "Phoenix".to_string()),
        }
    }

    /// Start a webcam capture backend.
    pub async fn start_webcam_stream(&self) -> Result<VideoStream, LiveInputError> {
        if !self.webcam_enabled {
            return Err(LiveInputError::Disabled("webcam"));
        }

        #[cfg(feature = "video")]
        {
            use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat};
            let backend = CameraIndex::Index(0);
            let format = CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
            let camera = nokhwa::Camera::new(backend, format)
                .map_err(|e| LiveInputError::WebcamBackend(e.to_string()))?;
            Ok(VideoStream { camera })
        }

        #[cfg(not(feature = "video"))]
        {
            Err(LiveInputError::FeatureDisabled("video"))
        }
    }

    /// Start a microphone capture backend.
    pub async fn start_audio_stream(&self) -> Result<AudioStream, LiveInputError> {
        if !self.microphone_enabled {
            return Err(LiveInputError::Disabled("microphone"));
        }

        #[cfg(feature = "audio")]
        {
            use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
            use cpal::{Sample, SampleFormat, StreamConfig};

            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or(LiveInputError::NoMicrophone)?;
            let supported_config = device
                .default_input_config()
                .map_err(|e| LiveInputError::AudioBackend(e.to_string()))?;

            let sample_format = supported_config.sample_format();
            let config: StreamConfig = supported_config.into();

            fn err_fn(err: cpal::StreamError) {
                eprintln!("[multi_modal_input] audio stream error: {err}");
            }

            fn process_audio_chunk(_data: &[f32]) {
                // Capture-only crate: callers should provide their own downstream processing.
                // (e.g. VAD / wake-word / emotion analysis in separate, explicitly opted-in crates.)
            }

            fn build_stream<T>(
                device: &cpal::Device,
                config: &StreamConfig,
            ) -> Result<cpal::Stream, LiveInputError>
            where
                T: Sample,
            {
                let stream = device
                    .build_input_stream(
                        config,
                        move |data: &[T], _info: &cpal::InputCallbackInfo| {
                            let mut buf = Vec::with_capacity(data.len());
                            for s in data {
                                buf.push(s.to_f32());
                            }
                            process_audio_chunk(&buf);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| LiveInputError::AudioBackend(e.to_string()))?;
                Ok(stream)
            }

            let stream = match sample_format {
                SampleFormat::F32 => build_stream::<f32>(&device, &config)?,
                SampleFormat::I16 => build_stream::<i16>(&device, &config)?,
                SampleFormat::U16 => build_stream::<u16>(&device, &config)?,
                other => {
                    return Err(LiveInputError::AudioBackend(format!(
                        "unsupported sample format: {other:?}"
                    )))
                }
            };

            stream
                .play()
                .map_err(|e| LiveInputError::AudioBackend(e.to_string()))?;

            Ok(AudioStream { stream })
        }

        #[cfg(not(feature = "audio"))]
        {
            Err(LiveInputError::FeatureDisabled("audio"))
        }
    }

    /// Detect wake word in audio buffer.
    pub async fn detect_wake_word(&self, _audio_buffer: &[f32]) -> bool {
        #[cfg(feature = "voice")]
        {
            use vosk::VoskRecognizer;
            
            if !self.voice_recognition_enabled {
                return false;
            }

            // Initialize Vosk recognizer with the wake word model
            let model_path = "path/to/vosk-model"; // Replace with actual model path
            let recognizer = VoskRecognizer::new(model_path, 16000.0)
                .map_err(|e| LiveInputError::VoiceRecognitionError(e.to_string()));

            match recognizer {
                Ok(mut rec) => {
                    if rec.accept_waveform(_audio_buffer, _audio_buffer.len()) {
                        let result = rec.result();
                        result.contains(&self.wake_word)
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        }

        #[cfg(not(feature = "voice"))]
        {
            false
        }
    }

    /// Recognize face in the provided image frame.
    pub async fn recognize_face(&self, frame: &RgbImage) -> Option<String> {
        #[cfg(feature = "face")]
        {
            if !self.face_recognition_enabled {
                return None;
            }

            // NOTE: `rustface::Detector` is a trait; constructing a detector requires a concrete
            // implementation + model data. Keep this crate compile-safe by using a stub until a
            // model-loading story is provided.
            let _ = frame;
            eprintln!(
                "[multi_modal_input] recognize_face invoked, but face recognition is stubbed (no model loaded); returning None"
            );
            None
        }

        #[cfg(not(feature = "face"))]
        {
            let _ = frame;
            eprintln!(
                "[multi_modal_input] recognize_face invoked, but feature `face` is not enabled; returning None"
            );
            None
        }
    }

    /// Analyze facial emotion in the provided image frame.
    pub async fn analyze_facial_emotion(&self, frame: &RgbImage) -> DetectedEmotion {
        // Fallback to the legacy tract-onnx stub (kept for compatibility).
        #[cfg(feature = "emotion")]
        {
            let _ = frame;
            eprintln!("[multi_modal_input] analyze_facial_emotion invoked (legacy tract-onnx stub enabled); returning Neutral");
            DetectedEmotion::Neutral
        }

        #[cfg(not(feature = "emotion"))]
        {
            let _ = frame;
            eprintln!(
                "[multi_modal_input] analyze_facial_emotion invoked, but no facial emotion backend is enabled; returning Neutral"
            );
            DetectedEmotion::Neutral
        }
    }

    /// Analyze voice emotion in the provided audio buffer.
    pub async fn analyze_voice_emotion(&self, audio: &[f32]) -> DetectedEmotion {
        // Pitch, energy, rate analysis
        let energy: f32 = audio.iter().map(|&x| x * x).sum();
        let _pitch = 0.0; // Placeholder for pitch detection
        let _rate = 0.0; // Placeholder for speech rate detection

        // Simple heuristic for emotion detection
        if energy > 0.5 {
            DetectedEmotion::Anger
        } else if energy < 0.1 {
            DetectedEmotion::Sadness
        } else {
            DetectedEmotion::Neutral
        }
    }
}

fn env_bool(key: &str) -> Option<bool> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .and_then(|s| match s.as_str() {
            "1" | "true" | "yes" | "y" | "on" => Some(true),
            "0" | "false" | "no" | "n" | "off" => Some(false),
            _ => None,
        })
}

