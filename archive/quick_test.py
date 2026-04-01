#!/usr/bin/env python3
"""
Quick test script for Face Verification API
"""
import requests
import base64
import sys
from pathlib import Path

def test_verify(image1_path, image2_path):
    """Test face verification with two images"""
    
    # Check if images exist
    if not Path(image1_path).exists():
        print(f"❌ Error: {image1_path} not found!")
        return
    
    if not Path(image2_path).exists():
        print(f"❌ Error: {image2_path} not found!")
        return
    
    print("\n=== Face Verification Test ===")
    print(f"Image 1: {image1_path}")
    print(f"Image 2: {image2_path}")
    
    # Read and encode images
    with open(image1_path, 'rb') as f:
        image1_b64 = base64.b64encode(f.read()).decode('utf-8')
    
    with open(image2_path, 'rb') as f:
        image2_b64 = base64.b64encode(f.read()).decode('utf-8')
    
    # Send request
    print("\nSending request to server...")
    
    try:
        response = requests.post(
            'http://localhost:8080/api/v1/verify',
            json={
                'image1_b64': image1_b64,
                'image2_b64': image2_b64
            },
            timeout=120
        )
        
        if response.status_code == 200:
            result = response.json()
            
            print("\n╔════════════════════════════════════════╗")
            print("║        VERIFICATION RESULT             ║")
            print("╚════════════════════════════════════════╝")
            
            # Verified status
            if result['verified']:
                print("\n✅ VERIFIED: Same Person")
            else:
                print("\n❌ NOT VERIFIED: Different Person")
            
            # Distance and similarity
            distance = result['distance']
            threshold = result['threshold']
            similarity = (1 - distance) * 100
            
            print(f"\nCosine Distance: {distance:.4f}")
            print(f"Threshold:       {threshold}")
            print(f"Similarity:      {similarity:.2f}%")
            
            # Model info
            print(f"\nModel:    {result['model']}")
            print(f"Detector: {result['detector']}")
            print(f"Metric:   {result['metric']}")
            
            # Interpretation
            print("\n─────────────────────────────────────────")
            if distance <= 0.20:
                print("Interpretation: Very High Confidence Match")
            elif distance <= 0.30:
                print("Interpretation: Match (within threshold)")
            elif distance <= 0.40:
                print("Interpretation: Close but not verified")
            else:
                print("Interpretation: Clearly different persons")
            print("─────────────────────────────────────────\n")
            
        else:
            print(f"\n❌ Error: HTTP {response.status_code}")
            print(response.text)
            
    except requests.exceptions.ConnectionError:
        print("\n❌ Error: Cannot connect to server")
        print("Make sure the server is running:")
        print("  cargo run --release")
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == '__main__':
    # Default test images
    image1 = 'test_images/face1.jpg'
    image2 = 'test_images/face2.jpg'
    
    # Allow command line arguments
    if len(sys.argv) == 3:
        image1 = sys.argv[1]
        image2 = sys.argv[2]
    
    test_verify(image1, image2)
