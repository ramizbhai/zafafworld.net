import os
import sys
import requests
import time
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

def test_cache_flushing():
    # 1. Login
    print("1. Logging in...")
    login_data = {
        "email": "vendor@test.com",
        "password": "Ramiz@789",
        "domain_type": "Vendor"
    }
    r = requests.post(f"{API_URL}/auth/login", headers=HEADERS_BASE, json=login_data, verify=False)
    if r.status_code != 200:
        print(f"Login failed: {r.status_code}")
        sys.exit(1)
    token = r.json().get("token")

    # 2. Upload test image
    print("2. Uploading test image...")
    dummy_img_path = "/tmp/test_cache_dummy.webp"
    from PIL import Image
    img = Image.new('RGB', (100, 100), color='blue')
    img.save(dummy_img_path, format='WEBP')

    upload_headers = {"Host": "api.zafafworld.net", "Authorization": f"Bearer {token}"}
    files = {"file": ("test_cache_dummy.webp", open(dummy_img_path, "rb"), "image/webp")}
    r = requests.post(f"{API_URL}/vendor/upload", headers=upload_headers, files=files, data={"media_type": "image"}, verify=False)
    if r.status_code != 200:
        print(f"Upload failed: {r.status_code}")
        sys.exit(1)
    
    file_url = r.json().get("url")
    print(f"Image uploaded successfully. URL: {file_url}")

    # 3. Request once to populate Nginx cache (MISS)
    print("3. Fetching image 1st time to populate cache...")
    r1 = requests.get(f"https://127.0.0.1{file_url}", headers=HEADERS_BASE, verify=False)
    print(f"  Response: HTTP {r1.status_code}, X-Cache-Status: {r1.headers.get('X-Cache-Status')}")

    # 4. Request again to verify cache HIT
    print("4. Fetching image 2nd time to verify cache HIT...")
    r2 = requests.get(f"https://127.0.0.1{file_url}", headers=HEADERS_BASE, verify=False)
    cache_status = r2.headers.get('X-Cache-Status')
    print(f"  Response: HTTP {r2.status_code}, X-Cache-Status: {cache_status}")
    if cache_status != "HIT":
        print(f"WARNING: Expected cache status 'HIT', got '{cache_status}'. Proxy caching might not be enabled for this URL.")

    # 5. Stop MinIO
    print("5. Stopping MinIO container...")
    code, stdout, stderr = run_command("podman stop zafafworld_minio_1")
    if code != 0:
        print(f"Failed to stop MinIO: {stderr}")
        sys.exit(1)

    # 6. Restart Nginx to flush cache
    print("6. Restarting Nginx container (should flush cache)...")
    code, stdout, stderr = run_command("podman stop zafafworld_nginx_1 && podman start zafafworld_nginx_1")
    if code != 0:
        print(f"Failed to restart Nginx: {stderr}")
        run_command("podman start zafafworld_minio_1")
        sys.exit(1)

    # Wait a few seconds for Nginx container health check
    print("  Waiting 5 seconds for Nginx service to settle...")
    time.sleep(5)

    # 7. Request again while MinIO is down and Nginx was restarted
    print("7. Fetching image after Nginx restart (MinIO is still down)...")
    r3 = requests.get(f"https://127.0.0.1{file_url}", headers=HEADERS_BASE, verify=False)
    print(f"  Response: HTTP {r3.status_code}, X-Cache-Status: {r3.headers.get('X-Cache-Status')}")

    # If Nginx successfully flushed the cache, this request must be a MISS/EXPIRED and fail (502 or 504)
    # If the cache did NOT flush, Nginx would return HTTP 200 from its cache
    success = False
    if r3.status_code in [502, 504]:
        print("PASS: Cache was successfully flushed on Nginx restart! (Returned HTTP 502/504 as expected).")
        success = True
    elif r3.status_code == 200:
        print("FAIL: Cache was NOT flushed on Nginx restart! (Returned HTTP 200).")
    else:
        print(f"Unexpected status code: HTTP {r3.status_code}")

    # 8. Clean up and restore MinIO
    print("8. Restarting MinIO container...")
    run_command("podman start zafafworld_minio_1")
    
    if not success:
        sys.exit(1)

if __name__ == '__main__':
    test_cache_flushing()
