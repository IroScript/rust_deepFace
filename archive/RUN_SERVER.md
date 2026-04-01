# Running the Face Verification Server

## Quick Start

1. **Build and run in release mode (RECOMMENDED for performance):**
   ```powershell
   cargo run --release
   ```

2. **Wait for the server to start** (you'll see):
   ```
   INFO deepface_rs: All models loaded. Starting Axum server on 0.0.0.0:8080...
   ```

3. **Test the server** (in another terminal):
   ```powershell
   python -m quick_test
   ```
   or
   ```powershell
   .\test_verify.ps1
   ```

## Performance Notes

- **Release mode** is 10-50x faster than debug mode for ML inference
- First request may take 5-10 seconds (model warmup)
- Subsequent requests should be faster (1-3 seconds)
- Debug prints have been removed for better performance

## Troubleshooting

If you get timeout errors:
1. Make sure you're running in `--release` mode
2. The timeout has been increased to 120 seconds
3. Check that the server is actually running and listening on port 8080
