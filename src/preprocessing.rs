// ============================================================================
// preprocessing.rs — Image preprocessing for FaceNet512
//
// This is the most accuracy-critical file in the entire pipeline.
// A wrong normalization here will silently destroy accuracy — the model
// will still produce 512-dim vectors, but they'll be wrong.
//
// DeepFace source reference:
//   deepface/commons/image_utils.py → normalize_input(img, normalization)
//   deepface/models/FaceNet.py → target_size = (160, 160)
//
// For FaceNet512, DeepFace uses normalization="Facenet":
//   mean = img.mean()
//   std  = img.std()
//   std  = max(std, 1.0 / sqrt(total_pixels))   ← prevents division by ~0
//   normalized = (img - mean) / std
//
// This is "per-image standardization" from the original FaceNet paper
// (Schroff et al., 2015) — NOT the naive /255 approach.
//
// The output tensor shape for FaceNet512 ONNX input:
//   [1, 160, 160, 3] — NHWC layout (batch=1, height=160, width=160, channels=3)
//   dtype: f32
//
// NOTE on layout: DeepFace's Keras FaceNet uses NHWC (channels last).
// When exporting to ONNX, the layout is preserved.
// This is DIFFERENT from MTCNN which uses NCHW (channels first).
// Always verify the ONNX model's input shape before running.
// ============================================================================

use crate::mtcnn::FaceDetection;
use image::{DynamicImage, imageops::FilterType, GenericImageView};
use ndarray::Array4;

// FaceNet512 required input dimensions — hardcoded in DeepFace source
pub const FACENET_W: u32 = 160;
pub const FACENET_H: u32 = 160;

// ============================================================================
// crop_and_resize — extract the face region and scale to 160x160
//
// Python equivalent (deepface/commons/image_utils.py):
//   face_img = img[y1:y2, x1:x2]
//   face_img = cv2.resize(face_img, target_size)
//
// We clamp coordinates to image boundaries first (Python does this with
// padding, we simply clamp — same outcome for valid detections).
// ============================================================================
pub fn crop_and_resize(img: &DynamicImage, det: &FaceDetection, out_w: u32, out_h: u32) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();

    let x1 = det.bbox.0.max(0.0) as u32;
    let y1 = det.bbox.1.max(0.0) as u32;
    let x2 = (det.bbox.2 as u32).min(img_w);
    let y2 = (det.bbox.3 as u32).min(img_h);

    let crop_w = if x2 > x1 { x2 - x1 } else { 1 };
    let crop_h = if y2 > y1 { y2 - y1 } else { 1 };

    // Crop the face bounding box from the image
    let cropped = img.crop_imm(x1, y1, crop_w, crop_h);

    // Resize to the model's target size using bicubic interpolation
    // DeepFace uses PIL.Image.BICUBIC for final resize (set in v0.093 release)
    cropped.resize_exact(out_w, out_h, FilterType::CatmullRom) // CatmullRom ≈ bicubic
}

// ============================================================================
// normalize_facenet — apply per-image standardization for FaceNet
//
// This is the CRITICAL step. The normalization must exactly match what
// FaceNet512 was trained with. DeepFace's "Facenet" normalization is
// per-image z-score (mean=0, std=1), NOT the global [0,1] or [-1,1] scaling.
//
// Steps:
//   1. Convert u8 RGB pixels → f32
//   2. Compute per-image mean and std
//   3. Adjust std: std = max(std, 1/sqrt(N)) to prevent near-zero division
//   4. Normalize: (pixel - mean) / std
//   5. Reshape to [1, H, W, 3] NHWC for ONNX input (Keras layout)
//
// Output shape: [1, 160, 160, 3]   dtype: f32
// ============================================================================
pub fn normalize_facenet(img: &DynamicImage) -> Array4<f32> {
    let rgb = img.to_rgb8();
    let (w, h) = (FACENET_W as usize, FACENET_H as usize);
    let total_pixels = (w * h * 3) as f32; // R + G + B = 3 channels per pixel

    // ── Step 1: Collect all pixel values as f32 ───────────────────────────
    let mut raw: Vec<f32> = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let p = rgb.get_pixel(x as u32, y as u32);
            raw.push(p[0] as f32); // R
            raw.push(p[1] as f32); // G
            raw.push(p[2] as f32); // B
        }
    }

    // ── Step 2: Per-image mean ────────────────────────────────────────────
    let mean: f32 = raw.iter().sum::<f32>() / total_pixels;

    // ── Step 3: Per-image std ─────────────────────────────────────────────
    let variance: f32 = raw.iter()
        .map(|&v| (v - mean) * (v - mean))
        .sum::<f32>() / total_pixels;
    let std_raw = variance.sqrt();

    // Adjusted std: prevents division-by-zero for uniform images.
    // Matches DeepFace's: std_adj = np.maximum(std, 1.0 / np.sqrt(img.size))
    let std_adj = std_raw.max(1.0 / total_pixels.sqrt());

    // ── Step 4: Normalize ─────────────────────────────────────────────────
    let normalized: Vec<f32> = raw.iter()
        .map(|&v| (v - mean) / std_adj)
        .collect();

    // ── Step 5: Reshape to [1, H, W, 3] NHWC ─────────────────────────────
    // NHWC layout: index = n*H*W*C + y*W*C + x*C + c
    // This matches how Keras stores tensors and how FaceNet512 ONNX expects input
    let mut tensor = Array4::<f32>::zeros((1, h, w, 3));

    let mut idx = 0;
    for y in 0..h {
        for x in 0..w {
            tensor[[0, y, x, 0]] = normalized[idx];     // R
            tensor[[0, y, x, 1]] = normalized[idx + 1]; // G
            tensor[[0, y, x, 2]] = normalized[idx + 2]; // B
            idx += 3;
        }
    }

    tensor
}

// ============================================================================
// IMPORTANT NOTE on input normalization (for future maintainers):
//
// There are multiple normalization options in DeepFace:
//
//   "base"       → pixel / 255.0                      (range: [0, 1])
//   "raw"        → pixel as-is, no scaling             (range: [0, 255])
//   "Facenet"    → (pixel - mean) / std                ← WE USE THIS for FaceNet512
//   "Facenet2018"→ (pixel / 127.5) - 1.0               (range: [-1, 1])
//   "VGGFace"    → pixel - [93.595, 104.7, 129.186]    (BGR mean subtraction)
//   "VGGFace2"   → pixel - [91.4953, 103.8827, 131.0912]
//   "ArcFace"    → (pixel - 127.5) / 128.0             (range: ~[-1, 1])
//
// For FaceNet512, the correct one is "Facenet" = per-image z-score.
// Using the wrong one (e.g., "base" or "Facenet2018") will produce embeddings
// that appear valid but give wrong cosine distances, silently failing verification.
// ============================================================================
