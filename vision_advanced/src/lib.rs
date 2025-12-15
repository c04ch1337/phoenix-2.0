//! Advanced vision backend crate.
//!
//! This crate previously contained a native backend.
//! It is now a minimal stub that compiles without native libraries.

use image::{ImageBuffer, Rgb};

/// Local shim so callers can write `tract::Model` (requested API shape).
pub mod tract {
    #[derive(Debug, Clone)]
    pub struct Model;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DetectedEmotion {
    Happy,
    Sad,
    Angry,
    Fearful,
    Disgusted,
    Surprised,
    Neutral,
    Love,
}

#[derive(Debug, Clone)]
pub struct AdvancedVisionResult {
    pub face_rect: Rect,
    pub landmarks: Vec<Point>,
    pub emotion: DetectedEmotion,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct VisionResult {
    pub faces_detected: usize,
    pub primary_emotion: Option<DetectedEmotion>,
    pub results: Vec<AdvancedVisionResult>,
}

#[derive(Debug, Clone)]
pub enum VisionError {
    BackendUnavailable(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct AdvancedVision {
    // Stubbed: left as Options to preserve the intended shape.
    pub landmark_predictor: Option<tract::Model>,
    pub emotion_model: Option<tract::Model>,
}

impl AdvancedVision {
    pub fn new() -> Result<Self, VisionError> {
        dotenvy::dotenv().ok();

        // Keep env reads for compatibility, but do not load native backends.
        let _ = std::env::var("LANDMARK_MODEL_PATH").ok();
        let _ = std::env::var("EMOTION_ONNX_MODEL_PATH").ok();

        Ok(Self {
            landmark_predictor: None,
            emotion_model: None,
        })
    }

    pub async fn process_live_frame(
        &self,
        frame: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<VisionResult, VisionError> {
        let _ = frame;
        Ok(VisionResult {
            faces_detected: 0,
            primary_emotion: None,
            results: Vec::new(),
        })
    }
}


