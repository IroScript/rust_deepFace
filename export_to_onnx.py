"""
export_to_onnx.py — Run this ONCE to export DeepFace models to ONNX format.
After this, you never need Python or TensorFlow again.

Requirements:
    pip install deepface tensorflow tf2onnx onnx

Usage:
    python export_to_onnx.py

Output files (place in ./models/ next to your Rust binary):
    models/facenet512.onnx
    models/mtcnn_pnet.onnx
    models/mtcnn_rnet.onnx
    models/mtcnn_onet.onnx
"""

import os
import numpy as np
import tf2onnx
import onnx
import tensorflow as tf

os.makedirs("models", exist_ok=True)

# ─────────────────────────────────────────────────────────────────────────────
# Export FaceNet512
# DeepFace internally loads this as a Keras model.
# We build it and export directly to ONNX.
# ─────────────────────────────────────────────────────────────────────────────
print("[1/4] Building FaceNet512 model via DeepFace...")
from deepface import DeepFace

# This triggers DeepFace to download and cache the weights
facenet_client = DeepFace.build_model("Facenet512")

# Extract the actual Keras model from the client wrapper
facenet_model = facenet_client.model

print("[2/4] Exporting FaceNet512 to ONNX...")
input_signature = [
    tf.TensorSpec(shape=(None, 160, 160, 3), dtype=tf.float32, name="input_1")
]
onnx_model, _ = tf2onnx.convert.from_keras(
    facenet_model,
    input_signature=input_signature,
    opset=13,           # ONNX opset 13 is well-supported by ort 2.x
    output_path="models/facenet512.onnx"
)
print("    → Saved: models/facenet512.onnx")

# ─────────────────────────────────────────────────────────────────────────────
# Export MTCNN (P-Net, R-Net, O-Net)
# facenet-pytorch's MTCNN is the one DeepFace wraps.
# We export each of the 3 sub-networks separately.
# ─────────────────────────────────────────────────────────────────────────────
print("[3/4] Building MTCNN model...")
try:
    # DeepFace uses ipazc/mtcnn which is a TF/Keras-based MTCNN
    from mtcnn import MTCNN
    mtcnn_model = MTCNN()

    # Access internal sub-networks
    pnet = mtcnn_model.mtcnn.pnet
    rnet = mtcnn_model.mtcnn.rnet
    onet = mtcnn_model.mtcnn.onet

    print("[4/4] Exporting MTCNN P-Net → ONNX...")
    pnet_input_sig = [tf.TensorSpec(shape=(None, None, None, 3), dtype=tf.float32, name="input_pnet")]
    tf2onnx.convert.from_keras(pnet, input_signature=pnet_input_sig, opset=13, output_path="models/mtcnn_pnet.onnx")
    print("    → Saved: models/mtcnn_pnet.onnx")

    rnet_input_sig = [tf.TensorSpec(shape=(None, 24, 24, 3), dtype=tf.float32, name="input_rnet")]
    tf2onnx.convert.from_keras(rnet, input_signature=rnet_input_sig, opset=13, output_path="models/mtcnn_rnet.onnx")
    print("    → Saved: models/mtcnn_rnet.onnx")

    onet_input_sig = [tf.TensorSpec(shape=(None, 48, 48, 3), dtype=tf.float32, name="input_onet")]
    tf2onnx.convert.from_keras(onet, input_signature=onet_input_sig, opset=13, output_path="models/mtcnn_onet.onnx")
    print("    → Saved: models/mtcnn_onet.onnx")

except Exception as e:
    print(f"MTCNN export via mtcnn package failed: {e}")
    print("Trying alternative: exporting via facenet-pytorch MTCNN...")

    # Alternative: use facenet-pytorch's ONNX-friendly MTCNN
    # pip install facenet-pytorch
    import torch
    from facenet_pytorch import MTCNN as TorchMTCNN, fixed_image_standardization

    mtcnn = TorchMTCNN()

    # Export PNet
    dummy_pnet = torch.randn(1, 3, 12, 12)
    torch.onnx.export(
        mtcnn.pnet, dummy_pnet, "models/mtcnn_pnet.onnx",
        input_names=["input"], output_names=["cls", "reg"],
        dynamic_axes={"input": {2: "H", 3: "W"}},
        opset_version=13
    )

    # Export RNet
    dummy_rnet = torch.randn(1, 3, 24, 24)
    torch.onnx.export(
        mtcnn.rnet, dummy_rnet, "models/mtcnn_rnet.onnx",
        input_names=["input"], output_names=["cls", "reg"],
        opset_version=13
    )

    # Export ONet
    dummy_onet = torch.randn(1, 3, 48, 48)
    torch.onnx.export(
        mtcnn.onet, dummy_onet, "models/mtcnn_onet.onnx",
        input_names=["input"], output_names=["cls", "reg", "pts"],
        opset_version=13
    )
    print("    → Saved all MTCNN ONNX files via PyTorch path.")

print("\n✅ Export complete. Files ready in ./models/")
print("   Copy these 4 .onnx files next to your compiled Rust binary.")
print("   Then: cargo run --release")
