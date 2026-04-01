// ============================================================================
// inference.rs — FaceNet512 ONNX inference with tract
//
// Replaces Python's:
//   embedding = model.predict(normalized_img)[0, :]
//
// The ONNX model was loaded once at startup in main.rs AppState.
// We pass the preprocessed [1, 160, 160, 3] f32 tensor through the
// FaceNet512 model and extract the raw 512-dimensional embedding vector.
//
// NO INT8, NO QUANTIZATION — we use f32 throughout.
// FaceNet512 uses 512-dimensional embeddings for higher accuracy than
// the original 128-dimensional FaceNet model.
//
// ONNX input/output format for FaceNet512:
//   Input name:  "input_1"   shape [1, 160, 160, 3]  dtype f32  (NHWC)
//   Output name: "Bottleneck_BatchNorm/batchnorm/add_1:0"
//                             shape [1, 512]          dtype f32
// ============================================================================

use crate::errors::AppError;
use ndarray::{Array1, Array4};
use tract_onnx::prelude::*;

// ============================================================================
// get_embedding — run FaceNet512 on one preprocessed face crop
//
// Input:  normalized_image — Array4<f32> shape [1, 160, 160, 3]
// Output: raw_embedding    — Array1<f32> shape [512]
//
// The returned embedding is RAW (not L2-normalized yet).
// L2 normalization is applied separately in math.rs to keep responsibilities
// clean and testable.
// ============================================================================
pub fn get_embedding(
    model: &SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    normalized_image: Array4<f32>,
) -> Result<Array1<f32>, AppError> {

    // Convert ndarray to tract tensor
    let input_tensor: Tensor = normalized_image.into_dyn().into();
    
    // Run inference
    let outputs = model.run(tvec![input_tensor.into()])
        .map_err(|e| AppError::OnnxError(format!("FaceNet inference failed: {e}")))?;

    // Extract embedding from output
    let output = outputs[0]
        .to_array_view::<f32>()
        .map_err(|e| AppError::OnnxError(format!("Failed to convert output to array: {e}")))?;
    
    // Get first batch element [0, :] - tract uses its own ndarray
    let embedding_slice = output.index_axis(tract_core::ndarray::Axis(0), 0);
    
    // Convert from tract's ndarray to our ndarray
    let embedding: Array1<f32> = Array1::from_iter(embedding_slice.iter().copied());

    // Sanity check
    debug_assert_eq!(
        embedding.len(),
        512,
        "Expected 512-dim embedding from FaceNet512, got {}",
        embedding.len()
    );

    Ok(embedding)
}
