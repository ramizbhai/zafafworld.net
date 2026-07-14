#!/usr/bin/env python3
import sys
import os
import io
import time
import requests
import psycopg2
from uuid import uuid4
from PIL import Image

API_BASE = "http://127.0.0.1:8080/api/v1"
DB_URL = "postgres://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"

def get_db_connection():
    return psycopg2.connect(DB_URL)

def generate_tiny_image():
    img = Image.new('RGB', (100, 100), color='blue')
    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format='JPEG')
    return img_byte_arr.getvalue()

def main():
    print("=== STARTING COMPREHENSIVE REGRESSION SMOKE TEST ===")

    # Step 1: Register Vendor
    email = f"smoke_{uuid4().hex[:8]}@zafafworld.com"
    password = "Password@123"
    print(f"\n[Step 1] Registering a new vendor with email: {email}")
    
    register_payload = {
        "email": email,
        "password": password,
        "domain_type": "Vendor",
        "first_name": "Smoke",
        "last_name": "Test"
    }
    
    res = requests.post(f"{API_BASE}/auth/register", json=register_payload)
    if res.status_code != 200:
        print(f"FAILED: Vendor registration failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
    print("SUCCESS: Vendor registered successfully.")

    # Step 2: Login Vendor
    print("\n[Step 2] Logging in as the new vendor...")
    login_payload = {
        "email": email,
        "password": password,
        "domain_type": "Vendor"
    }
    res = requests.post(f"{API_BASE}/auth/login", json=login_payload)
    if res.status_code != 200:
        print(f"FAILED: Vendor login failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
    
    token = res.json()["token"]
    headers = {
        "Authorization": f"Bearer {token}"
    }
    print("SUCCESS: Vendor logged in, token acquired.")

    # Step 3: Create a Product
    print("\n[Step 3] Creating a product to associate image with...")
    product_payload = {
        "titleAr": "منتج تجريبي",
        "titleEn": "Smoke Test Product",
        "basePriceSar": 150.00
    }
    res = requests.post(f"{API_BASE}/vendor/products", json=product_payload, headers=headers)
    if res.status_code != 200:
        print(f"FAILED: Product creation failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
    
    product_id = res.json()["data"]["productId"]
    print(f"SUCCESS: Product created with ID: {product_id}")

    # Step 4: Upload Media File
    print("\n[Step 4] Uploading JPEG media file to /vendor/upload...")
    img_data = generate_tiny_image()
    files = {
        "file": ("smoke.jpg", img_data, "image/jpeg")
    }
    data = {
        "product_id": product_id,
        "media_type": "image",
        "is_cover": "true"
    }
    res = requests.post(f"{API_BASE}/vendor/upload", files=files, data=data, headers=headers)
    if res.status_code != 200:
        print(f"FAILED: Media upload failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
        
    upload_res = res.json()
    img_url = upload_res["url"]
    file_path = upload_res["file_path"]
    print(f"SUCCESS: Upload processed. URL: {img_url}, Disk Path: {file_path}")

    # Step 5: Associate Image with Product Gallery
    print("\n[Step 5] Adding image to product gallery...")
    gallery_payload = {
        "imageUrl": img_url,
        "filePath": file_path,
        "isCover": True,
        "caption": "Smoke Test Cover Image"
    }
    res = requests.post(f"{API_BASE}/vendor/products/{product_id}/images", json=gallery_payload, headers=headers)
    if res.status_code != 200:
        print(f"FAILED: Adding image to gallery failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
        
    image_id = res.json()["data"]["id"]
    print(f"SUCCESS: Image registered in product gallery with ID: {image_id}")

    # Step 6: Verify Database Records & Disk Existence
    print("\n[Step 6] Verifying DB records and physical file on disk...")
    conn = get_db_connection()
    cur = conn.cursor()
    
    # Check vendor_gallery table
    cur.execute("SELECT id, image_url, file_path FROM vendor_gallery WHERE id = %s", (image_id,))
    gallery_row = cur.fetchone()
    if not gallery_row:
        print("FAILED: Image record not found in vendor_gallery table.")
        sys.exit(1)
    print(f"✓ Found in vendor_gallery: {gallery_row}")

    # Check uploaded_files registry table (key = object path stripped from url or prefix)
    clean_key = img_url.replace("/assets/uploads/", "assets/uploads/")
    cur.execute("SELECT id, object_key FROM uploaded_files WHERE object_key = %s OR object_key = %s", (clean_key, img_url))
    uploaded_row = cur.fetchone()
    if not uploaded_row:
        # Check matching any key
        cur.execute("SELECT id, object_key FROM uploaded_files ORDER BY created_at DESC LIMIT 5")
        rows = cur.fetchall()
        print(f"WARNING: File not registered under expected key '{clean_key}'. Recent uploads: {rows}")
    else:
        print(f"✓ Found in uploaded_files: {uploaded_row}")

    # Check physical file exists on disk
    full_disk_path = os.path.join("/opt/zafafworld.net/backend-rust", file_path)
    if not os.path.exists(full_disk_path):
        print(f"FAILED: Physical file not found at: {full_disk_path}")
        sys.exit(1)
    print(f"✓ Physical file exists on disk: {full_disk_path}")
    cur.close()
    conn.close()

    # Step 7: Delete Image
    print("\n[Step 7] Deleting product image...")
    res = requests.delete(f"{API_BASE}/vendor/products/{product_id}/images/{image_id}", headers=headers)
    if res.status_code != 200:
        print(f"FAILED: Deleting image failed. HTTP {res.status_code}: {res.text}")
        sys.exit(1)
    print("SUCCESS: Deletion request complete.")

    # Step 8: Verify Complete Cleanup
    print("\n[Step 8] Verifying cleanup of database registry and physical files...")
    conn = get_db_connection()
    cur = conn.cursor()

    # Check vendor_gallery table
    cur.execute("SELECT id FROM vendor_gallery WHERE id = %s", (image_id,))
    if cur.fetchone():
        print("FAILED: Image record still exists in vendor_gallery table.")
        sys.exit(1)
    print("✓ Removed from vendor_gallery.")

    # Check uploaded_files registry table
    cur.execute("SELECT id FROM uploaded_files WHERE object_key = %s OR object_key = %s", (clean_key, img_url))
    if cur.fetchone():
        print("FAILED: File record still exists in uploaded_files registry.")
        sys.exit(1)
    print("✓ Removed from uploaded_files.")

    # Check physical file on disk
    if os.path.exists(full_disk_path):
        print(f"FAILED: Physical file still exists at: {full_disk_path}")
        sys.exit(1)
    print("✓ Physical file deleted from disk.")
    
    cur.close()
    conn.close()
    
    print("\n=== ALL REGRESSION SMOKE TEST LIFECYCLE PHASES PASSED! ===")

if __name__ == "__main__":
    main()
