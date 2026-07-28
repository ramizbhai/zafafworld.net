import os
import sys
import requests
import json
import subprocess
from dotenv import load_dotenv

# Load environment
dotenv_path = '/opt/zafafworld.net/.env'
load_dotenv(dotenv_path)

API_URL = "https://127.0.0.1/api/v1"
HEADERS_BASE = {"Host": "api.zafafworld.net"}

def run_command(cmd):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr

def test_flow():
    # 1. Login
    print("1. Logging in as vendor...")
    login_data = {
        "email": "vendor@test.com",
        "password": "Ramiz@789",
        "domain_type": "Vendor"
    }
    
    # Disable SSL verification for localhost/self-signed certs
    r = requests.post(f"{API_URL}/auth/login", headers=HEADERS_BASE, json=login_data, verify=False)
    if r.status_code != 200:
        print(f"Login failed: {r.status_code} - {r.text}")
        sys.exit(1)
        
    token = r.json().get("token")
    print("Login successful.")
    
    # 2. Upload a file
    print("2. Uploading a test image...")
    # Create a valid webp image using PIL
    dummy_img_path = "/tmp/test_upload_dummy.webp"
    from PIL import Image
    img = Image.new('RGB', (100, 100), color='red')
    img.save(dummy_img_path, format='WEBP')
        
    upload_headers = {
        "Host": "api.zafafworld.net",
        "Authorization": f"Bearer {token}"
    }
    
    files = {
        "file": ("test_upload_dummy.webp", open(dummy_img_path, "rb"), "image/webp")
    }
    data = {
        "media_type": "image"
    }
    
    r = requests.post(f"{API_URL}/vendor/upload", headers=upload_headers, files=files, data=data, verify=False)
    if r.status_code != 200:
        print(f"Upload failed: {r.status_code} - {r.text}")
        sys.exit(1)
        
    upload_res = r.json()
    print(f"Upload response: {json.dumps(upload_res, indent=2)}")
    
    file_url = upload_res.get("url")
    file_id = upload_res.get("id")
    
    if not file_url:
        print("Upload failed: No URL returned in response.")
        sys.exit(1)
        
    print(f"File uploaded successfully. URL: {file_url}")
    
    # 3. Verify no local files exist permanently in local uploads folder
    print("3. Verifying no local files are stored permanently...")
    # Look for files matching the ID in the local uploads directory
    local_dir = "/var/lib/zafafworld/uploads"
    found_local = False
    for root, dirs, filenames in os.walk(local_dir):
        for f in filenames:
            if file_id in f:
                print(f"  Found local file matching ID: {os.path.join(root, f)}")
                found_local = True
                
    if found_local:
        print("FAIL: Found local files stored permanently on disk!")
        sys.exit(1)
    else:
        print("PASS: No permanent local files found.")
        
    # 4. Verify file retrieval via Nginx from MinIO
    print("4. Retrieving file via Nginx...")
    r = requests.get(f"https://127.0.0.1{file_url}", headers=HEADERS_BASE, verify=False)
    print(f"Nginx retrieval status code: {r.status_code}")
    if r.status_code != 200:
        print("FAIL: Could not retrieve file via Nginx while MinIO is running.")
        sys.exit(1)
    print("PASS: Retrieved file successfully via Nginx.")
    
    # 5. Check if local directory serves uploads via Nginx
    print("5. Checking if local /uploads/ is served via Nginx...")
    # Test path /uploads/gallery/... instead of /assets/uploads/gallery/...
    local_path_url = file_url.replace("/assets/uploads/", "/uploads/")
    r = requests.get(f"https://127.0.0.1{local_path_url}", headers=HEADERS_BASE, verify=False)
    print(f"Local path url: {local_path_url}, Nginx status code: {r.status_code}")
    if r.status_code == 200:
        print("FAIL: Local /uploads/ is still served by Nginx!")
        sys.exit(1)
    else:
        print("PASS: Local /uploads/ is NOT served by Nginx (status code != 200).")
        
    # 6. Stop MinIO and check if media immediately becomes unavailable
    print("6. Stopping MinIO and verifying media becomes unavailable...")
    code, stdout, stderr = run_command("podman stop zafafworld_minio_1")
    if code != 0:
        print(f"Failed to stop MinIO container: {stderr}")
        sys.exit(1)
        
    # To bypass Nginx proxy cache, we append a query parameter
    url_uncached = f"{file_url}?nocache=1"
    r = requests.get(f"https://127.0.0.1{url_uncached}", headers=HEADERS_BASE, verify=False)
    print(f"Retrieve file when MinIO is down (uncached): HTTP {r.status_code}")
    
    # Since MinIO is down, Nginx must return 502 Bad Gateway or 504 Gateway Timeout or similar error.
    # It must NOT return 200.
    if r.status_code == 200:
        print("FAIL: Media is still accessible even when MinIO is down!")
        # Restart MinIO before exiting to restore state
        run_command("podman start zafafworld_minio_1")
        sys.exit(1)
    else:
        print("PASS: Media became unavailable when MinIO was down.")
        
    # 7. Restart MinIO and verify media becomes available again
    print("7. Restarting MinIO and verifying media is restored...")
    code, stdout, stderr = run_command("podman start zafafworld_minio_1")
    if code != 0:
        print(f"Failed to start MinIO container: {stderr}")
        sys.exit(1)
        
    # Wait a moment for MinIO to start
    import time
    time.sleep(12)
    
    r = requests.get(f"https://127.0.0.1{url_uncached}", headers=HEADERS_BASE, verify=False)
    print(f"Retrieve file after MinIO restart: HTTP {r.status_code}")
    if r.status_code == 200:
        print("PASS: Media restored successfully after restarting MinIO.")
    else:
        print("FAIL: Media not available after restarting MinIO.")
        sys.exit(1)
        
    print("ALL TESTS PASSED SUCCESSFULLY!")

if __name__ == '__main__':
    test_flow()
