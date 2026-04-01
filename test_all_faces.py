"""
Dynamic Face Verification Test
Compares the first image with all other images in test_images folder.
Usage: py -m test_all_faces
"""

import requests
import base64
import os
from pathlib import Path

API_URL = "http://localhost:8080"

def encode_image(image_path):
    """Encode image to base64"""
    with open(image_path, "rb") as f:
        return base64.b64encode(f.read()).decode("utf-8")

def verify_faces(img1_path, img2_path):
    """Call the /api/v1/verify endpoint"""
    try:
        img1_b64 = encode_image(img1_path)
        img2_b64 = encode_image(img2_path)
        
        response = requests.post(
            f"{API_URL}/api/v1/verify",
            json={"image1_b64": img1_b64, "image2_b64": img2_b64},
            timeout=120  # Increased to 120 seconds for slow processing
        )
        
        if response.status_code == 200:
            return response.json()
        else:
            return {"error": f"HTTP {response.status_code}: {response.text}"}
    except Exception as e:
        return {"error": str(e)}

def main():
    print("=" * 70)
    print("Dynamic Face Verification Test")
    print("=" * 70)
    
    # Get all image files from test_images folder
    test_dir = Path("test_images")
    if not test_dir.exists():
        print(f"❌ Error: {test_dir} folder not found!")
        return
    
    # Get all jpg/jpeg/png files, sorted
    image_files = sorted([
        f for f in test_dir.iterdir() 
        if f.suffix.lower() in ['.jpg', '.jpeg', '.png']
    ])
    
    if len(image_files) < 2:
        print(f"❌ Error: Need at least 2 images in {test_dir}")
        return
    
    # First image is the reference
    reference_img = image_files[0]
    other_images = image_files[1:]
    
    print(f"\n📸 Reference Image: {reference_img.name}")
    print(f"🔍 Comparing with {len(other_images)} other images...\n")
    print("-" * 70)
    
    results = []
    
    for i, img in enumerate(other_images, 1):
        print(f"\n[{i}/{len(other_images)}] Comparing: {reference_img.name} vs {img.name}")
        print(f"  Processing... (this may take 10-30 seconds per image)")
        
        result = verify_faces(str(reference_img), str(img))
        
        if "error" in result:
            print(f"  ❌ Error: {result['error']}")
            results.append({
                "image": img.name,
                "status": "error",
                "error": result['error']
            })
        else:
            verified = result.get("verified", False)
            distance = result.get("distance", 0)
            similarity = result.get("similarity_percent", 0)
            
            status = "✅ MATCH" if verified else "❌ NO MATCH"
            print(f"  {status}")
            print(f"  Distance: {distance:.4f}")
            print(f"  Similarity: {similarity:.2f}%")
            
            results.append({
                "image": img.name,
                "verified": verified,
                "distance": distance,
                "similarity": similarity
            })
    
    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    
    matches = sum(1 for r in results if r.get("verified", False))
    total = len(results)
    
    print(f"\nReference: {reference_img.name}")
    print(f"Total Comparisons: {total}")
    print(f"Matches: {matches}")
    print(f"Non-Matches: {total - matches}")
    
    print("\nDetailed Results:")
    print("-" * 70)
    for r in results:
        if r.get("status") == "error":
            print(f"  {r['image']:20s} - ERROR: {r['error']}")
        else:
            status = "MATCH    " if r['verified'] else "NO MATCH "
            print(f"  {r['image']:20s} - {status} (similarity: {r['similarity']:.2f}%)")
    
    print("\n" + "=" * 70)

if __name__ == "__main__":
    main()
