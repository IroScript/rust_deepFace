╔════════════════════════════════════════════════════════════════╗
║                    TEST IMAGES FOLDER                          ║
╚════════════════════════════════════════════════════════════════╝

এই folder-এ আপনার test করার জন্য face images রাখুন:

📁 test_images/
   ├── face1.jpg  ← প্রথম ব্যক্তির ছবি রাখুন
   └── face2.jpg  ← দ্বিতীয় ব্যক্তির ছবি রাখুন

═══════════════════════════════════════════════════════════════

কিভাবে test করবেন:

1. এই folder-এ দুটি face image রাখুন:
   - face1.jpg
   - face2.jpg

2. Server চালু আছে কিনা নিশ্চিত করুন:
   cargo run --release

3. Test script চালান:
   .\test_verify.ps1

═══════════════════════════════════════════════════════════════

Image Requirements:

✓ Format: JPG, PNG, WEBP
✓ Size: যেকোনো size (automatically resized হবে)
✓ Content: Clear face visible
✓ Quality: ভালো lighting এবং clear face

═══════════════════════════════════════════════════════════════

Example Output:

✅ VERIFIED: Same Person

Cosine Distance: 0.2341
Threshold:       0.30
Similarity:      76.59%

Model:    Facenet512
Detector: MTCNN
Metric:   cosine

Interpretation: Match (within threshold)

═══════════════════════════════════════════════════════════════

Distance Interpretation:

0.00 - 0.20  →  Very High Confidence Match (একই ব্যক্তি)
0.20 - 0.30  →  Match (threshold-এর মধ্যে)
0.30 - 0.40  →  Close but not verified (কাছাকাছি কিন্তু verify হয়নি)
0.40+        →  Clearly different persons (সম্পূর্ণ ভিন্ন ব্যক্তি)

═══════════════════════════════════════════════════════════════
