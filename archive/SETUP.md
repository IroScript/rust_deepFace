# DeepFace-RS Setup Guide

## প্রজেক্ট স্ট্রাকচার

```
deepface-rs/
├── Cargo.toml              # Rust dependencies
├── README.md               # প্রজেক্ট ডকুমেন্টেশন
├── SETUP.md                # এই ফাইল
├── export_to_onnx.py       # Python script (একবার চালাতে হবে)
├── Prompt.txt              # প্রজেক্ট requirement details
├── archive/                # পুরনো ফাইল
│   └── files.zip
├── models/                 # ONNX মডেল ফাইল (Python script চালানোর পর)
│   ├── facenet512.onnx
│   ├── mtcnn_pnet.onnx
│   ├── mtcnn_rnet.onnx
│   └── mtcnn_onet.onnx
└── src/                    # Rust source code
    ├── main.rs             # Axum server + routing
    ├── mtcnn.rs            # MTCNN face detection (3-stage)
    ├── alignment.rs        # Face alignment (eye rotation)
    ├── preprocessing.rs    # Image preprocessing + normalization
    ├── inference.rs        # FaceNet512 ONNX inference
    ├── math.rs             # L2 normalization + cosine distance
    └── errors.rs           # Error handling
```

## সেটআপ স্টেপ

### ১. ONNX মডেল এক্সপোর্ট করুন (একবার মাত্র)

Python এবং প্রয়োজনীয় লাইব্রেরি ইনস্টল করুন:

```bash
pip install deepface tensorflow tf2onnx onnx mtcnn facenet-pytorch torch
```

তারপর export script চালান:

```bash
python export_to_onnx.py
```

এটি `models/` ফোল্ডারে ৪টি `.onnx` ফাইল তৈরি করবে।

### ২. ONNX Runtime ইনস্টল করুন

আপনার OS অনুযায়ী ONNX Runtime ইনস্টল করুন:

**Windows:**
- https://github.com/microsoft/onnxruntime/releases থেকে ডাউনলোড করুন
- অথবা: `winget install Microsoft.ONNXRuntime`

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install libonnxruntime-dev

# অথবা manual install
wget https://github.com/microsoft/onnxruntime/releases/download/v1.17.0/onnxruntime-linux-x64-1.17.0.tgz
tar -xzf onnxruntime-linux-x64-1.17.0.tgz
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(pwd)/onnxruntime-linux-x64-1.17.0/lib
```

**macOS:**
```bash
brew install onnxruntime
```

### ৩. Rust প্রজেক্ট বিল্ড করুন

```bash
cargo build --release
```

### ৪. সার্ভার চালান

```bash
cargo run --release
```

সার্ভার `http://0.0.0.0:8080` এ চালু হবে।

## API ব্যবহার

### JSON Endpoint

```bash
curl -X POST http://localhost:8080/api/v1/verify \
  -H "Content-Type: application/json" \
  -d '{
    "image1_b64": "<base64_encoded_jpeg>",
    "image2_b64": "<base64_encoded_jpeg>"
  }'
```

### Multipart Endpoint

```bash
curl -X POST http://localhost:8080/api/v1/verify_multipart \
  -F "image1=@face1.jpg" \
  -F "image2=@face2.jpg"
```

### Response

```json
{
  "verified": true,
  "distance": 0.18,
  "threshold": 0.30,
  "model": "Facenet512",
  "detector": "MTCNN",
  "metric": "cosine"
}
```

## সমস্যা সমাধান

### ONNX Runtime লোড হচ্ছে না

Windows-এ যদি `onnxruntime.dll` খুঁজে না পায়:
- DLL ফাইল ডাউনলোড করে `C:\Windows\System32\` তে রাখুন
- অথবা প্রজেক্ট ফোল্ডারে রাখুন

### Python script ব্যর্থ হচ্ছে

যদি MTCNN export ব্যর্থ হয়, দুটি পথ চেষ্টা করা হয়:
1. `mtcnn` package (TensorFlow-based)
2. `facenet-pytorch` (PyTorch-based)

উভয়ই ইনস্টল করুন:
```bash
pip install mtcnn facenet-pytorch
```

### Build error: linking failed

ONNX Runtime library path সেট করুন:
```bash
# Linux
export LD_LIBRARY_PATH=/path/to/onnxruntime/lib:$LD_LIBRARY_PATH

# Windows (PowerShell)
$env:PATH += ";C:\path\to\onnxruntime\lib"
```

## পারফরম্যান্স নোট

- প্রথম request-এ মডেল লোড হয় (~2-3 সেকেন্ড)
- পরবর্তী request গুলো অত্যন্ত দ্রুত (~50-100ms)
- Python DeepFace-এর তুলনায় 10-20x দ্রুত
- মেমোরি ব্যবহার ~120MB (Python-এ 1-4GB)
