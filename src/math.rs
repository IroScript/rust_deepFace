// ============================================================================
// math.rs — L2 normalization and cosine distance
//
// Replaces Python's:
//   from deepface.commons import distance as dst
//   embed1 = dst.l2_normalize(embed1)
//   dist   = dst.findCosineDistance(embed1, embed2)
//
// Source: deepface/commons/distance.py
//
// These are the two most critical math operations in the whole pipeline.
// Getting either wrong means all verification results are wrong.
// ============================================================================

use ndarray::Array1;

// ============================================================================
// l2_normalize — convert a raw embedding vector to a unit vector
//
// Formula:
//   ||v|| = sqrt(sum(v_i^2))
//   v_norm = v / ||v||
//
// After this operation: dot(v_norm, v_norm) == 1.0
//
// Why is this needed?
//   FaceNet's raw output has varying magnitudes. Normalizing makes all
//   embeddings lie on a unit hypersphere. Then cosine distance simplifies
//   to: 1 - dot(a, b) — no denominator needed.
//
// Python equivalent (deepface/commons/distance.py):
//   def l2_normalize(x):
//       return x / np.sqrt(np.sum(np.multiply(x, x)))
// ============================================================================
pub fn l2_normalize(embedding: &Array1<f32>) -> Array1<f32> {
    // Compute the L2 norm (Euclidean length of the vector)
    let norm: f32 = embedding.iter()
        .map(|&x| x * x)
        .sum::<f32>()
        .sqrt();

    // Guard against zero-vector (should never happen with a valid face image,
    // but a corrupted input could theoretically produce all-zero output)
    if norm < f32::EPSILON {
        return embedding.clone();
    }

    // Divide every element by the norm → unit vector
    embedding.mapv(|x| x / norm)
}

// ============================================================================
// cosine_distance — measure how "different" two faces are
//
// Formula (full):
//   cosine_distance = 1 - (dot(A, B) / (||A|| * ||B||))
//
// Formula (simplified, since we already L2-normalized):
//   cosine_distance = 1 - dot(A, B)
//   (because ||A|| == ||B|| == 1.0 after l2_normalize)
//
// Interpretation:
//   distance = 0.0  → identical (same face, same lighting, same angle)
//   distance = 0.30 → threshold (DeepFace default for FaceNet512 + cosine)
//   distance = 1.0  → completely different
//   distance = 2.0  → mathematically opposite (impossible with real faces)
//
// Python equivalent (deepface/commons/distance.py):
//   def findCosineDistance(source_representation, test_representation):
//       a = np.matmul(np.transpose(source_representation), test_representation)
//       b = np.sum(np.multiply(source_representation, source_representation))
//       c = np.sum(np.multiply(test_representation, test_representation))
//       return 1 - (a / (np.sqrt(b) * np.sqrt(c)))
//
// Our version skips the denominator (== 1 after L2 norm), making it
// slightly faster and numerically cleaner.
// ============================================================================
pub fn cosine_distance(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Embedding dimensions must match");

    // Dot product = sum(a_i * b_i)
    // This is equivalent to numpy's np.dot(a, b)
    let dot: f32 = a.iter().zip(b.iter())
        .map(|(&ai, &bi)| ai * bi)
        .sum();

    // Cosine similarity = dot product (since both vectors are unit vectors)
    // Cosine distance = 1 - similarity
    1.0 - dot
}

// ============================================================================
// euclidean_distance — alternative metric (included but not used by default)
//
// DeepFace also supports euclidean and euclidean_l2.
// For FaceNet512 + euclidean_l2, the threshold is 0.30 (same value as cosine).
// For FaceNet512 + euclidean (without L2 norm), the threshold is 23.56.
//
// We keep this available but cosine is the default per your specification.
//
// Python equivalent:
//   def findEuclideanDistance(source_representation, test_representation):
//       euclidean_distance = source_representation - test_representation
//       euclidean_distance = np.sum(np.multiply(euclidean_distance, euclidean_distance))
//       euclidean_distance = np.sqrt(euclidean_distance)
//       return euclidean_distance
// ============================================================================
#[allow(dead_code)]
pub fn euclidean_distance(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    a.iter().zip(b.iter())
        .map(|(&ai, &bi)| (ai - bi) * (ai - bi))
        .sum::<f32>()
        .sqrt()
}

// ============================================================================
// THRESHOLD REFERENCE (from deepface/modules/verification.py):
//
//   Model          | Metric       | Threshold
//   ---------------|--------------|----------
//   Facenet512     | cosine       | 0.30     ← we use this
//   Facenet512     | euclidean    | 23.56
//   Facenet512     | euclidean_l2 | 0.30
//   Facenet (128d) | cosine       | 0.40
//   VGG-Face       | cosine       | 0.40
//   ArcFace        | cosine       | 0.68
// ============================================================================
