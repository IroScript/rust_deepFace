// ============================================================================
// main.rs — Axum server entry point
// Replaces: Python's `flask run` / `uvicorn main:app`
// Key difference: models are loaded ONCE into Arc<AppState> at startup.
// Python reloads heavy TF/Keras graph on every cold start. We never do that.
// ============================================================================

mod mtcnn;
mod alignment;
mod preprocessing;
mod inference;
mod math;
mod errors;

use std::sync::Arc;
use axum::{
    routing::post,
    Router,
    extract::{State, Multipart},
    Json,
};
use tract_onnx::prelude::*;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::info;
use anyhow::Result;

// ============================================================================
// AppState — loaded once at startup, Arc-shared across all async tasks
// Python equivalent: global model = DeepFace.build_model("Facenet512")
// The difference: Python keeps the Keras session alive per-process.
// We keep it alive globally in Arc — zero re-init cost per request.
// tract models are thread-safe by design (no Mutex needed)
// ============================================================================
pub struct AppState {
    /// FaceNet512 ONNX model — the 512-dim embedding model
    pub facenet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// MTCNN P-Net model
    pub pnet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// MTCNN R-Net model
    pub rnet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    /// MTCNN O-Net model
    pub onet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
}

// ============================================================================
// Request/Response shapes
// ============================================================================

/// Accepts two base64-encoded images in JSON body.
/// Alternatively the /verify_multipart endpoint accepts multipart/form-data.
#[derive(Deserialize)]
pub struct VerifyRequest {
    /// Base64 encoded image 1 (JPEG or PNG)
    pub image1_b64: String,
    /// Base64 encoded image 2 (JPEG or PNG)
    pub image2_b64: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    /// True if same person (cosine_distance <= COSINE_THRESHOLD)
    pub verified: bool,
    /// Raw cosine distance — lower = more similar
    pub distance: f32,
    /// Threshold used (FaceNet512 + cosine default = 0.30)
    pub threshold: f32,
    /// Which model was used
    pub model: &'static str,
    /// Which detector was used
    pub detector: &'static str,
    /// Which distance metric was used
    pub metric: &'static str,
}

// ============================================================================
// The cosine threshold for FaceNet512 as defined in DeepFace source:
// deepface/modules/verification.py → dst_threshold map
// "Facenet512" + "cosine" → 0.30
// ============================================================================
pub const COSINE_THRESHOLD: f32 = 0.30;

// ============================================================================
// Main — load all ONNX models with tract, start Axum
// ============================================================================
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("deepface_rs=debug,tract=warn")
        .init();

    info!("Loading ONNX models with tract...");
    
    // Load FaceNet512 model
    info!("Loading FaceNet512 ONNX model...");
    let facenet_model = tract_onnx::onnx()
        .model_for_path("models/facenet512.onnx")?
        .into_optimized()?
        .into_runnable()?;
    
    // Load MTCNN models
    info!("Loading MTCNN P-Net model...");
    let pnet_model = tract_onnx::onnx()
        .model_for_path("models/mtcnn_pnet.onnx")?
        .into_optimized()?
        .into_runnable()?;
    
    info!("Loading MTCNN R-Net model...");
    let rnet_model = tract_onnx::onnx()
        .model_for_path("models/mtcnn_rnet.onnx")?
        .into_optimized()?
        .into_runnable()?;
    
    info!("Loading MTCNN O-Net model...");
    let onet_model = tract_onnx::onnx()
        .model_for_path("models/mtcnn_onet.onnx")?
        .into_optimized()?
        .into_runnable()?;

    info!("All models loaded. Starting Axum server on 0.0.0.0:8080...");

    let state = Arc::new(AppState {
        facenet_model: Arc::new(facenet_model),
        pnet_model: Arc::new(pnet_model),
        rnet_model: Arc::new(rnet_model),
        onet_model: Arc::new(onet_model),
    });

    let app = Router::new()
        // JSON endpoint: {"image1_b64": "...", "image2_b64": "..."}
        .route("/api/v1/verify", post(verify_json_handler))
        // Multipart endpoint: form fields image1 + image2 (raw bytes)
        .route("/api/v1/verify_multipart", post(verify_multipart_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// JSON handler — decodes base64, runs the full pipeline
// ============================================================================
async fn verify_json_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, errors::AppError> {
    use base64::{Engine as _, engine::general_purpose};

    let bytes1 = general_purpose::STANDARD
        .decode(&req.image1_b64)
        .map_err(|e| errors::AppError::BadRequest(format!("image1 base64 decode failed: {e}")))?;

    let bytes2 = general_purpose::STANDARD
        .decode(&req.image2_b64)
        .map_err(|e| errors::AppError::BadRequest(format!("image2 base64 decode failed: {e}")))?;

    let response = run_pipeline(&state, &bytes1, &bytes2)?;
    Ok(Json(response))
}

// ============================================================================
// Multipart handler — reads raw image bytes from form fields
// ============================================================================
async fn verify_multipart_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<VerifyResponse>, errors::AppError> {
    let mut image1_bytes: Option<Vec<u8>> = None;
    let mut image2_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| errors::AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await
            .map_err(|e| errors::AppError::BadRequest(e.to_string()))?;

        match name.as_str() {
            "image1" => image1_bytes = Some(data.to_vec()),
            "image2" => image2_bytes = Some(data.to_vec()),
            _ => {}
        }
    }

    let bytes1 = image1_bytes.ok_or_else(|| errors::AppError::BadRequest("missing field: image1".into()))?;
    let bytes2 = image2_bytes.ok_or_else(|| errors::AppError::BadRequest("missing field: image2".into()))?;

    let response = run_pipeline(&state, &bytes1, &bytes2)?;
    Ok(Json(response))
}

// ============================================================================
// run_pipeline — THE full DeepFace pipeline in Rust:
//   detect → align → preprocess → embed → distance → threshold
//
// Python equivalent:
//   result = DeepFace.verify(img1, img2,
//       model_name="Facenet512",
//       detector_backend="mtcnn",
//       distance_metric="cosine")
// ============================================================================
fn run_pipeline(
    state: &AppState,
    bytes1: &[u8],
    bytes2: &[u8],
) -> Result<VerifyResponse, errors::AppError> {
    use image::DynamicImage;

    info!("Starting verification pipeline...");
    
    // ── Step 1: Decode raw bytes → DynamicImage (replaces cv2.imread) ──────
    info!("Decoding images...");
    let img1: DynamicImage = image::load_from_memory(bytes1)
        .map_err(|e| errors::AppError::ImageDecode(e.to_string()))?;
    let img2: DynamicImage = image::load_from_memory(bytes2)
        .map_err(|e| errors::AppError::ImageDecode(e.to_string()))?;
    info!("Images decoded: {}x{} and {}x{}", img1.width(), img1.height(), img2.width(), img2.height());

    // ── Step 2: MTCNN detection → bbox + 5 landmarks ──────────────────────
    // Returns the tightest face bounding box and eye/nose/mouth coordinates
    info!("Detecting face in image 1...");
    let det1 = mtcnn::detect_face(state, &img1)?;
    info!("Face 1 detected at ({:.1}, {:.1}, {:.1}, {:.1}) confidence: {:.3}", 
        det1.bbox.0, det1.bbox.1, det1.bbox.2, det1.bbox.3, det1.confidence);
    
    info!("Detecting face in image 2...");
    let det2 = mtcnn::detect_face(state, &img2)?;
    info!("Face 2 detected at ({:.1}, {:.1}, {:.1}, {:.1}) confidence: {:.3}", 
        det2.bbox.0, det2.bbox.1, det2.bbox.2, det2.bbox.3, det2.confidence);

    // ── Step 3: Alignment (rotate so eyes are horizontal) ─────────────────
    // This is DeepFace's align=True logic — adds ~1% accuracy per the paper
    info!("Aligning faces...");
    let aligned1 = alignment::align_face(&img1, &det1);
    let aligned2 = alignment::align_face(&img2, &det2);

    // ── Step 4: Crop to bounding box, resize to 160x160 ───────────────────
    // FaceNet512 target_size = (160, 160) — hardcoded in DeepFace source
    info!("Cropping and resizing to 160x160...");
    let cropped1 = preprocessing::crop_and_resize(&aligned1, &det1, 160, 160);
    let cropped2 = preprocessing::crop_and_resize(&aligned2, &det2, 160, 160);

    // ── Step 5: Pixel normalization — DeepFace "Facenet" normalization ─────
    // Source: deepface/commons/image_utils.py → normalize_input(img, "Facenet")
    // Formula: (pixel - mean(img)) / max(std(img), 1/sqrt(N))
    // This is per-image standardization, NOT the naive /255 approach.
    // Using the wrong normalization here silently tanks accuracy by ~10-15%.
    info!("Normalizing pixels...");
    let tensor1 = preprocessing::normalize_facenet(&cropped1); // shape [1, 160, 160, 3] f32
    let tensor2 = preprocessing::normalize_facenet(&cropped2);

    // ── Step 6: ONNX Inference → raw 512-dim embedding ───────────────────
    info!("Running FaceNet512 inference...");
    let raw_embed1 = inference::get_embedding(&state.facenet_model, tensor1)?;
    let raw_embed2 = inference::get_embedding(&state.facenet_model, tensor2)?;

    // ── Step 7: L2 Normalize embeddings (unit vectors) ────────────────────
    // DeepFace always l2_normalize before distance computation
    // After this, ||embed|| = 1.0, so cosine denominator = 1
    info!("Computing distance...");
    let embed1 = math::l2_normalize(&raw_embed1);
    let embed2 = math::l2_normalize(&raw_embed2);

    // ── Step 8: Cosine Distance ───────────────────────────────────────────
    // distance = 1 - dot(embed1, embed2)
    // (denominator = 1 because we already L2-normalized above)
    let distance = math::cosine_distance(&embed1, &embed2);

    let verified = distance <= COSINE_THRESHOLD;
    
    info!("Verification complete: {} (distance: {:.4})", if verified { "MATCH" } else { "NO MATCH" }, distance);

    Ok(VerifyResponse {
        verified,
        distance,
        threshold: COSINE_THRESHOLD,
        model: "Facenet512",
        detector: "MTCNN",
        metric: "cosine",
    })
}
