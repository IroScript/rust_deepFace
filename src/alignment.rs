// ============================================================================
// alignment.rs — Face alignment via 2D affine rotation
//
// Replaces Python's:
//   from deepface.commons import image_utils
//   aligned_img = image_utils.align_face(img, left_eye, right_eye)
//
// What alignment does:
//   Given that MTCNN found two eye positions, we rotate the image so those
//   two eyes are on a perfectly horizontal line. This is what bumps FaceNet
//   accuracy from ~98.87% to ~99.63% (reported by Google, cited in DeepFace).
//
// Math:
//   1. Find the angle θ between the eye line and the horizontal axis
//   2. Build a 2D rotation matrix around the midpoint of the two eyes
//   3. Apply the rotation to every pixel (bilinear interpolation)
//
// We implement this WITHOUT OpenCV dependency using pure ndarray math,
// matching the exact logic in DeepFace's align_face() function.
// ============================================================================

use crate::mtcnn::FaceDetection;
use image::{DynamicImage, RgbImage, Rgb, GenericImageView};

// ============================================================================
// align_face — rotate the image so eyes become horizontal
//
// Returns a new DynamicImage with the face centered and rotated.
// The bounding box in `det` still refers to pre-rotation coordinates,
// but preprocessing::crop_and_resize will handle the crop after rotation.
// ============================================================================
pub fn align_face(img: &DynamicImage, det: &FaceDetection) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let rgb = img.to_rgb8();

    // ── Extract eye coordinates from MTCNN landmarks ───────────────────────
    // Landmark order from O-Net:
    //   [0] = left_eye   (viewer's left, person's right)
    //   [1] = right_eye  (viewer's right, person's left)
    let left_eye  = det.landmarks[0];
    let right_eye = det.landmarks[1];

    // ── Compute rotation angle θ ───────────────────────────────────────────
    // We want the eyes to be on the same horizontal line (y1 == y2).
    // The angle between the current eye line and the horizontal is:
    //   θ = atan2(right_eye.y - left_eye.y, right_eye.x - left_eye.x)
    //
    // Python equivalent (deepface/commons/image_utils.py):
    //   angle = np.degrees(np.arctan2(
    //       right_eye[1] - left_eye[1],
    //       right_eye[0] - left_eye[0]
    //   ))
    let dy = right_eye.1 - left_eye.1;
    let dx = right_eye.0 - left_eye.0;
    let angle_rad = dy.atan2(dx); // radians

    // ── Rotation center = midpoint between the eyes ───────────────────────
    let center_x = (left_eye.0 + right_eye.0) / 2.0;
    let center_y = (left_eye.1 + right_eye.1) / 2.0;

    // ── 2D Affine rotation matrix ─────────────────────────────────────────
    // We rotate by -angle (to correct for the tilt).
    // Standard 2D rotation about a center point (cx, cy):
    //
    //   cos_a  sin_a   cx - cx*cos_a - cy*sin_a
    //  -sin_a  cos_a   cy + cx*sin_a - cy*cos_a
    //
    // This is the formula used by cv2.getRotationMatrix2D(center, angle, 1.0)
    // We replicate it exactly so results match DeepFace.
    let cos_a = (-angle_rad).cos();
    let sin_a = (-angle_rad).sin();

    let tx = center_x - center_x * cos_a - center_y * sin_a;
    let ty = center_y + center_x * sin_a - center_y * cos_a;

    // ── Apply inverse affine map to each output pixel ─────────────────────
    // We use the INVERSE transform to avoid holes:
    // For each output pixel (ox, oy), find the source pixel (sx, sy):
    //   [sx]   [cos_a   sin_a] [ox - tx]
    //   [sy] = [-sin_a  cos_a] [oy - ty]
    //
    // Source pixel lookup = inverse of the forward rotation.
    let inv_cos = cos_a;   // inverse of rotation matrix is its transpose
    let inv_sin = -sin_a;

    let out_w = img_w;
    let out_h = img_h;

    let mut out = RgbImage::new(out_w, out_h);

    for oy in 0..out_h {
        for ox in 0..out_w {
            // Map output → source via inverse affine
            let sx = inv_cos * (ox as f32 - tx) + inv_sin * (oy as f32 - ty);
            let sy = -inv_sin * (ox as f32 - tx) + inv_cos * (oy as f32 - ty);

            // Bilinear interpolation — avoids blocky artifacts on rotation
            let pixel = bilinear_sample(&rgb, sx, sy, img_w, img_h);
            out.put_pixel(ox, oy, pixel);
        }
    }

    DynamicImage::ImageRgb8(out)
}

// ============================================================================
// bilinear_sample — smooth pixel lookup at non-integer coordinates
//
// Replaces: cv2.warpAffine's default INTER_LINEAR interpolation.
//
// How it works:
//   Given fractional coordinate (x, y), find the four surrounding integer
//   pixels and blend them proportionally to their distance.
//
//   TL (x0,y0) ─── TR (x1,y0)
//       |                |
//   BL (x0,y1) ─── BR (x1,y1)
//
//   weight_x = x - x0   (fractional part)
//   weight_y = y - y0
//   result = TL*(1-wx)*(1-wy) + TR*wx*(1-wy) + BL*(1-wx)*wy + BR*wx*wy
// ============================================================================
fn bilinear_sample(img: &RgbImage, x: f32, y: f32, w: u32, h: u32) -> Rgb<u8> {
    // Out-of-bounds → black pixel (matches cv2.warpAffine BORDER_CONSTANT=0)
    if x < 0.0 || y < 0.0 || x >= (w - 1) as f32 || y >= (h - 1) as f32 {
        return Rgb([0u8, 0u8, 0u8]);
    }

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);

    let wx = x - x0 as f32; // horizontal weight
    let wy = y - y0 as f32; // vertical weight

    let tl = img.get_pixel(x0, y0);
    let tr = img.get_pixel(x1, y0);
    let bl = img.get_pixel(x0, y1);
    let br = img.get_pixel(x1, y1);

    let mut result = [0u8; 3];
    for c in 0..3 {
        let v = tl[c] as f32 * (1.0 - wx) * (1.0 - wy)
              + tr[c] as f32 * wx           * (1.0 - wy)
              + bl[c] as f32 * (1.0 - wx)   * wy
              + br[c] as f32 * wx           * wy;
        result[c] = v.clamp(0.0, 255.0) as u8;
    }

    Rgb(result)
}
