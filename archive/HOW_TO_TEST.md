# 🧪 কিভাবে Face Verification Test করবেন

## ধাপ ১: Server চালু করুন

একটি terminal-এ:

```powershell
cargo run --release
```

Server শুরু হবে এবং এই message দেখাবে:
```
INFO deepface_rs: All models loaded. Starting Axum server on 0.0.0.0:8080...
```

**এই terminal খোলা রাখুন!** Server চলতে থাকবে।

---

## ধাপ ২: Test Images রাখুন

`test_images` folder-এ দুটি face image রাখুন:

```
test_images/
├── face1.jpg  ← প্রথম ব্যক্তির ছবি
└── face2.jpg  ← দ্বিতীয় ব্যক্তির ছবি
```

**Image Requirements:**
- ✅ Format: JPG, PNG, WEBP
- ✅ Clear face visible
- ✅ Good lighting
- ✅ Any size (automatically resized)

---

## ধাপ ৩: Test Script চালান

**নতুন terminal** খুলুন এবং চালান:

```powershell
.\test_verify.ps1
```

---

## 📊 Output Example

### Same Person (Verified):
```
✅ VERIFIED: Same Person

Cosine Distance: 0.2341
Threshold:       0.30
Similarity:      76.59%

Model:    Facenet512
Detector: MTCNN
Metric:   cosine

Interpretation: Match (within threshold)
```

### Different Person (Not Verified):
```
❌ NOT VERIFIED: Different Person

Cosine Distance: 0.4523
Threshold:       0.30
Similarity:      54.77%

Model:    Facenet512
Detector: MTCNN
Metric:   cosine

Interpretation: Clearly different persons
```

---

## 🎯 Distance Interpretation

| Distance | Similarity | Meaning |
|----------|-----------|---------|
| 0.00 - 0.20 | 80-100% | **Very High Confidence** - একই ব্যক্তি |
| 0.20 - 0.30 | 70-80% | **Match** - threshold-এর মধ্যে ✅ |
| 0.30 - 0.40 | 60-70% | **Close** - কাছাকাছি কিন্তু verify হয়নি |
| 0.40+ | <60% | **Different** - সম্পূর্ণ ভিন্ন ব্যক্তি ❌ |

---

## 🔧 Alternative: Manual Testing

### Option A: Using curl (Multipart)

```powershell
curl -X POST http://localhost:8080/api/v1/verify_multipart `
  -F "image1=@test_images/face1.jpg" `
  -F "image2=@test_images/face2.jpg"
```

### Option B: Using PowerShell (JSON)

```powershell
# Encode images
$bytes1 = [System.IO.File]::ReadAllBytes("test_images\face1.jpg")
$base64_1 = [Convert]::ToBase64String($bytes1)

$bytes2 = [System.IO.File]::ReadAllBytes("test_images\face2.jpg")
$base64_2 = [Convert]::ToBase64String($bytes2)

# Send request
$body = @{
    image1_b64 = $base64_1
    image2_b64 = $base64_2
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/verify" `
  -Method POST `
  -ContentType "application/json" `
  -Body $body
```

---

## ❓ Troubleshooting

### Error: "Could not find face1.jpg"
- ✅ নিশ্চিত করুন `test_images` folder-এ `face1.jpg` এবং `face2.jpg` আছে
- ✅ File name ঠিক আছে কিনা check করুন (case-sensitive)

### Error: "Connection refused"
- ✅ Server চালু আছে কিনা check করুন: `cargo run --release`
- ✅ Server terminal-এ "Starting Axum server" message দেখা যাচ্ছে কিনা

### Error: "No face detected"
- ✅ Image-এ clear face visible আছে কিনা check করুন
- ✅ Better lighting এবং clear photo ব্যবহার করুন
- ✅ Face খুব ছোট বা খুব বড় না হলে ভালো

---

## 🚀 Next Steps

Test সফল হলে আপনি:

1. **Production-এ deploy** করতে পারেন
2. **Frontend** তৈরি করতে পারেন
3. **Database integration** করতে পারেন
4. **Multiple faces** support যোগ করতে পারেন

---

## 📝 API Endpoints

### POST `/api/v1/verify`
JSON endpoint with base64 encoded images

**Request:**
```json
{
  "image1_b64": "base64_string...",
  "image2_b64": "base64_string..."
}
```

**Response:**
```json
{
  "verified": true,
  "distance": 0.2341,
  "threshold": 0.30,
  "model": "Facenet512",
  "detector": "MTCNN",
  "metric": "cosine"
}
```

### POST `/api/v1/verify_multipart`
Multipart form-data endpoint

**Request:**
```
Content-Type: multipart/form-data
- image1: (binary file)
- image2: (binary file)
```

**Response:** Same as above

---

**Happy Testing! 🎉**
