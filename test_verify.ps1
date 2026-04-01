# ============================================================================
# Face Verification Test Script
# ============================================================================

Write-Host "`n=== Face Verification Test ===" -ForegroundColor Cyan
Write-Host "Server: http://localhost:8080" -ForegroundColor Yellow

# Check if images exist
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$image1Path = Join-Path $scriptDir "test_images\face1.jpg"
$image2Path = Join-Path $scriptDir "test_images\face2.jpg"

if (-not (Test-Path $image1Path)) {
    Write-Host "`n❌ Error: $image1Path not found!" -ForegroundColor Red
    Write-Host "Please place face1.jpg in the test_images folder" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $image2Path)) {
    Write-Host "`n❌ Error: $image2Path not found!" -ForegroundColor Red
    Write-Host "Please place face2.jpg in the test_images folder" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n✓ Found: $image1Path" -ForegroundColor Green
Write-Host "✓ Found: $image2Path" -ForegroundColor Green

# Read and encode images
Write-Host "`nEncoding images to base64..." -ForegroundColor Cyan
$bytes1 = [System.IO.File]::ReadAllBytes($image1Path)
$base64_1 = [Convert]::ToBase64String($bytes1)

$bytes2 = [System.IO.File]::ReadAllBytes($image2Path)
$base64_2 = [Convert]::ToBase64String($bytes2)

Write-Host "✓ Image 1: $($bytes1.Length) bytes" -ForegroundColor Green
Write-Host "✓ Image 2: $($bytes2.Length) bytes" -ForegroundColor Green

# Create JSON request
$body = @{
    image1_b64 = $base64_1
    image2_b64 = $base64_2
} | ConvertTo-Json

# Send request
Write-Host "`nSending request to server..." -ForegroundColor Cyan

try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/verify" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body
    
    # Display results
    Write-Host "`n╔════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║        VERIFICATION RESULT             ║" -ForegroundColor Cyan
    Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
    
    # Verified status
    if ($response.verified) {
        Write-Host "`n✅ VERIFIED: Same Person" -ForegroundColor Green
    } else {
        Write-Host "`n❌ NOT VERIFIED: Different Person" -ForegroundColor Red
    }
    
    # Distance
    $distance = [math]::Round($response.distance, 4)
    $threshold = $response.threshold
    Write-Host "`nCosine Distance: $distance" -ForegroundColor Yellow
    Write-Host "Threshold:       $threshold" -ForegroundColor Yellow
    
    # Similarity percentage (1 - distance) * 100
    $similarity = [math]::Round((1 - $response.distance) * 100, 2)
    Write-Host "Similarity:      $similarity%" -ForegroundColor Cyan
    
    # Model info
    Write-Host "`nModel:    $($response.model)" -ForegroundColor Gray
    Write-Host "Detector: $($response.detector)" -ForegroundColor Gray
    Write-Host "Metric:   $($response.metric)" -ForegroundColor Gray
    
    # Interpretation
    Write-Host "`n─────────────────────────────────────────" -ForegroundColor Gray
    if ($distance -le 0.20) {
        Write-Host "Interpretation: Very High Confidence Match" -ForegroundColor Green
    } elseif ($distance -le 0.30) {
        Write-Host "Interpretation: Match (within threshold)" -ForegroundColor Green
    } elseif ($distance -le 0.40) {
        Write-Host "Interpretation: Close but not verified" -ForegroundColor Yellow
    } else {
        Write-Host "Interpretation: Clearly different persons" -ForegroundColor Red
    }
    Write-Host "─────────────────────────────────────────`n" -ForegroundColor Gray
    
} catch {
    Write-Host "`n❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "`nMake sure the server is running:" -ForegroundColor Yellow
    Write-Host "  cargo run --release" -ForegroundColor Cyan
    exit 1
}
