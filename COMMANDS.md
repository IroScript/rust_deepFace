# DeepFace-RS Commands Reference

এই ফাইলে সব প্রয়োজনীয় commands আছে।

---

## ⚡ Quick Start - প্রথমবার Setup (Step by Step)

### ১. Python Install করুন
```powershell
# Python 3.7+ install করুন (যদি না থাকে)
python --version
# অথবা
py --version
```

### ২. Python Dependencies Install করুন
```powershell
pip install deepface tensorflow tf2onnx onnx mtcnn
```

### ৩. ONNX Models Export করুন
```powershell
python export_to_onnx.py
```

### ৪. Release Build করুন
```powershell
cargo build --release
```

### ৫. Server চালান (.exe file হিসেবে)
```powershell
.\target\release\deepface-rs.exe
```

### ৬. Test করুন (নতুন terminal এ)
```powershell
py test_all_faces.py
```

---

## ⚡ পরবর্তীতে (Models থাকলে শুধু এই ৩টি)

### ১. Release Build করুন
```powershell
cargo build --release
```

### ২. Server চালান
```powershell
.\target\release\deepface-rs.exe
```

### ৩. Test করুন
```powershell
py test_all_faces.py
```

---

## 🚀 Server চালানো

### Development Mode (Debug Build)
```powershell
cargo run
```
- দ্রুত compile হয়
- Debugging সহজ
- ধীর performance

### Production Mode (Release Build)
```powershell
cargo run --release
```
- Compile করতে সময় লাগে (প্রথমবার)
- 22x দ্রুত
- Production এর জন্য best

### Direct .exe Run
```powershell
# Release build
.\target\release\deepface-rs.exe

# Debug build
.\target\debug\deepface-rs.exe
```

---

## 🛠️ Build Commands

### Debug Build
```powershell
cargo build
```
Output: `target\debug\deepface-rs.exe`

### Release Build (Optimized)
```powershell
cargo build --release
```
Output: `target\release\deepface-rs.exe`

### Clean Build
```powershell
cargo clean
cargo build --release
```

---

## 🧪 Testing

### Python Test Script চালানো
```powershell
# সব ছবি test করুন
py test_all_faces.py

# অথবা
python test_all_faces.py
```

### PowerShell Test Script
```powershell
# Execution policy set করুন (প্রথমবার)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Test চালান
.\test_verify.ps1
```

### Manual cURL Test
```powershell
# JSON endpoint test
curl -X POST http://localhost:8080/api/v1/verify `
  -H "Content-Type: application/json" `
  -d '{\"image1_b64\":\"...\",\"image2_b64\":\"...\"}'
```

---

## 📦 ONNX Models Export

### Models তৈরি করুন (প্রথমবার)
```powershell
# Python dependencies install
pip install deepface tensorflow tf2onnx onnx mtcnn

# Models export করুন
python export_to_onnx.py
```

Output:
```
models/
├── facenet512.onnx
├── mtcnn_pnet.onnx
├── mtcnn_rnet.onnx
└── mtcnn_onet.onnx
```

---

## 🔍 Process Management

### Running Processes দেখুন
```powershell
Get-Process | Where-Object {$_.ProcessName -like "*deepface*"}
```

### Server Stop করুন
```powershell
# সব deepface processes kill করুন
Get-Process | Where-Object {$_.ProcessName -like "*deepface*"} | Stop-Process -Force

# অথবা Ctrl+C চাপুন terminal এ
```

### Port Check করুন
```powershell
# Port 8080 কে use করছে দেখুন
netstat -ano | findstr :8080
```

---

## 📊 Logs দেখা

### Server Logs
Server চালালে automatically logs দেখাবে:
```
2026-04-01T10:40:47.930369Z  INFO deepface_rs: Loading ONNX models...
2026-04-01T10:40:49.917921Z  INFO deepface_rs: All models loaded. Starting Axum server on 0.0.0.0:8080...
```

### Log Level Change করুন
```powershell
# Environment variable set করুন
$env:RUST_LOG="deepface_rs=debug,tract=warn"
cargo run --release
```

---

## 🐛 Debugging

### Syntax Check (দ্রুত)
```powershell
cargo check
```

### Warnings দেখুন
```powershell
cargo build 2>&1 | Select-String "warning"
```

### Clippy (Linter)
```powershell
cargo clippy
```

### Format Code
```powershell
cargo fmt
```

---

## 📁 File Management

### Test Images যোগ করুন
```powershell
# test_images folder এ ছবি copy করুন
Copy-Item "C:\path\to\image.jpg" -Destination "test_images\"
```

### Models Backup
```powershell
# Models folder backup করুন
Copy-Item -Recurse "models" -Destination "models_backup"
```

---

## 🌐 API Endpoints

### Verify Endpoint (JSON)
```
POST http://localhost:8080/api/v1/verify
Content-Type: application/json

{
  "image1_b64": "base64_encoded_image_1",
  "image2_b64": "base64_encoded_image_2"
}
```

### Verify Endpoint (Multipart)
```
POST http://localhost:8080/api/v1/verify_multipart
Content-Type: multipart/form-data

image1: <file>
image2: <file>
```

---

## 🔧 Troubleshooting

### Port Already in Use
```powershell
# Old process kill করুন
Get-Process | Where-Object {$_.ProcessName -like "*deepface*"} | Stop-Process -Force

# অথবা port change করুন (main.rs এ)
# let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
```

### Models Not Found
```powershell
# Models আছে কিনা check করুন
Test-Path "models\facenet512.onnx"

# না থাকলে export করুন
python export_to_onnx.py
```

### Python Not Found
```powershell
# Python install check করুন
python --version

# অথবা py command use করুন
py --version
```

### Cargo Not Found
```powershell
# Rust install করুন
# https://rustup.rs/

# অথবা PATH check করুন
$env:PATH
```

---

## 📈 Performance Testing

### Single Request Time
```powershell
Measure-Command {
    py test_all_faces.py
}
```

### Multiple Requests (Concurrent)
```powershell
# PowerShell এ parallel requests
1..10 | ForEach-Object -Parallel {
    Invoke-RestMethod -Uri "http://localhost:8080/api/v1/verify" -Method POST -Body $body
}
```

---

## 🎯 Quick Start

### প্রথমবার Setup
```powershell
# 1. Models export করুন
pip install deepface tensorflow tf2onnx onnx mtcnn
python export_to_onnx.py

# 2. Build করুন
cargo build --release

# 3. Server চালান
.\target\release\deepface-rs.exe

# 4. নতুন terminal এ test করুন
py test_all_faces.py
```

### পরবর্তীতে
```powershell
# Server চালান
.\target\release\deepface-rs.exe

# Test করুন
py test_all_faces.py
```

---

## 📝 Notes

- **Release build** সবসময় production এ use করুন (22x faster)
- **Models folder** অবশ্যই থাকতে হবে
- **Port 8080** free থাকতে হবে
- **Python 3.7+** লাগবে test script এর জন্য

---

## 🆘 Help

### Cargo Help
```powershell
cargo --help
cargo run --help
cargo build --help
```

### Project Structure দেখুন
```powershell
tree /F
```

### Dependencies দেখুন
```powershell
cargo tree
```

---

**Made with ❤️ using Rust + ONNX + MTCNN + FaceNet512**
