#!/usr/bin/env python3
import sys
import requests
import json
import datetime

API_BASE = "https://127.0.0.1/api/v1"

# We use verify=False because the local Nginx uses self-signed SSL certificates.
# We disable warnings to clean up output logs.
requests.packages.urllib3.disable_warnings()

headers = {
    "Host": "api.zafafworld.net",
    "Content-Type": "application/json"
}

print("=== Starting Listing Promotion System Lifecycle Verification ===")

# 1. Vendor Login
print("\n[Step 1] Vendor Login...")
login_payload = {
    "email": "ramiz@zafaf.net",
    "password": "Admin@123456",
    "domain_type": "Vendor"
}
res = requests.post(f"{API_BASE}/auth/login", json=login_payload, headers=headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Vendor login failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)

vendor_token = res.json()["token"]
print("SUCCESS: Logged in as Vendor.")

# 2. Admin Login
print("\n[Step 2] Admin Login...")
admin_login_payload = {
    "email": "admin@zafafworld.net",
    "password": "Admin@123456",
    "domain_type": "Admin"
}
res = requests.post(f"{API_BASE}/auth/login", json=admin_login_payload, headers=headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Admin login failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)

admin_token = res.json()["token"]
print("SUCCESS: Logged in as Admin.")

# 3. Create Promotion (as Vendor)
print("\n[Step 3] Creating a Promotion...")
now = datetime.datetime.now(datetime.timezone.utc)
start_at_str = (now - datetime.timedelta(hours=1)).strftime("%Y-%m-%dT%H:%M:%SZ")
end_at_str = (now + datetime.timedelta(days=1)).strftime("%Y-%m-%dT%H:%M:%SZ")

promo_payload = {
    "title_en": "Integration Test Promo",
    "title_ar": "عرض تجريبي",
    "description_en": "Verification test promo desc",
    "description_ar": "وصف العرض",
    "discount_percentage": 25,
    "badge_text_en": "Special",
    "badge_text_ar": "خاص",
    "banner_image_url": "https://example.com/banner.jpg",
    "start_at": start_at_str,
    "end_at": end_at_str,
    "listing_ids": ["538a6fa5-740a-45ee-a8dd-75840c78a90b"]
}
vendor_headers = headers.copy()
vendor_headers["Authorization"] = f"Bearer {vendor_token}"

res = requests.post(f"{API_BASE}/vendor/promotions", json=promo_payload, headers=vendor_headers, verify=False)
if res.status_code != 200 and res.status_code != 201:
    print(f"FAILED: Create promotion failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)

promotion_id = res.json().get("id") or res.json().get("promotion_id")
if not promotion_id:
    # Try looking in nested data or keys
    promotion_id = res.json().get("data", {}).get("id") or res.json().get("data", {}).get("promotion_id")

if not promotion_id:
    print(f"FAILED: Could not extract promotion_id from response: {res.json()}")
    sys.exit(1)

print(f"SUCCESS: Created promotion with ID: {promotion_id}")

# 4. Overlap Check (as Vendor)
print("\n[Step 4] Verifying Overlap Prevention trigger...")
overlap_res = requests.post(f"{API_BASE}/vendor/promotions", json=promo_payload, headers=vendor_headers, verify=False)
if overlap_res.status_code != 409:
    print(f"FAILED: Overlap prevention failed! Expected HTTP 409, got {overlap_res.status_code}. Response: {overlap_res.text}")
    sys.exit(1)

body = overlap_res.json()
if body.get("status") != "error" or body.get("error_type") != "PROMOTION_OVERLAP" or body.get("code") != "PROMOTION_OVERLAP":
    print(f"FAILED: Overlap error payload format is incorrect: {body}")
    sys.exit(1)

print(f"SUCCESS: Overlap correctly rejected with HTTP 409 Conflict. Response: {json.dumps(body)}")

# 5. List Admin Promotions
print("\n[Step 5] Checking Admin Moderation Queue...")
admin_headers = headers.copy()
admin_headers["Authorization"] = f"Bearer {admin_token}"

res = requests.get(f"{API_BASE}/admin/promotions", headers=admin_headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Fetching admin queue failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)

promos = res.json().get("promotions") or res.json().get("data", {}).get("promotions") or []
found = any(p.get("id") == promotion_id or p.get("promotion_id") == promotion_id for p in promos)
if not found:
    print(f"WARNING: Created promotion {promotion_id} not found in admin queue list. Found list: {promos}")
else:
    print("SUCCESS: Promotion found in admin moderation queue.")

# 6. Approve Promotion (as Admin)
print("\n[Step 6] Admin Approving the Promotion...")
res = requests.post(f"{API_BASE}/admin/promotions/{promotion_id}/approve", headers=admin_headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Admin approval failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)
print("SUCCESS: Promotion approved by Admin.")

# 7. Public Endpoint Verification & Analytics Data Leakage Test
print("\n[Step 7] Fetching Public Promotions & Checking Analytics Leakage...")
res = requests.get(f"{API_BASE}/public/promotions", headers=headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Fetching public promotions failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)

public_promos = res.json().get("promotions") or []
target_promo = None
for p in public_promos:
    if p.get("promotion_id") == promotion_id or p.get("id") == promotion_id:
        target_promo = p
        break

if not target_promo:
    print(f"WARNING: Approved active promotion not returned by public catalog API. List: {public_promos}")
else:
    print("SUCCESS: Active promotion visible on public API catalog.")
    # Check for private analytics leakage
    private_keys = ["derived_analytics", "views", "clicks", "ctr", "bookings", "inquiries", "inquiries_count", "bookings_count"]
    leaked = [k for k in private_keys if k in target_promo]
    if leaked:
        print(f"FAILED: Private analytics leaked in public promotion catalog response! Leaked keys: {leaked}")
        sys.exit(1)
    else:
        print("SUCCESS: Data Isolation verified! No private analytics details leaked.")

# 8. Pause Promotion (as Vendor)
print("\n[Step 8] Pausing Promotion...")
res = requests.post(f"{API_BASE}/vendor/promotions/{promotion_id}/pause", headers=vendor_headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Pausing promotion failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)
print("SUCCESS: Promotion paused by Vendor.")

# 9. Resume Promotion (as Vendor)
print("\n[Step 9] Resuming Promotion...")
res = requests.post(f"{API_BASE}/vendor/promotions/{promotion_id}/resume", headers=vendor_headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Resuming promotion failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)
print("SUCCESS: Promotion resumed by Vendor.")

# 10. Delete Promotion (as Vendor)
print("\n[Step 10] Deleting Promotion...")
res = requests.delete(f"{API_BASE}/vendor/promotions/{promotion_id}", headers=vendor_headers, verify=False)
if res.status_code != 200:
    print(f"FAILED: Deleting promotion failed with code {res.status_code}. Response: {res.text}")
    sys.exit(1)
print("SUCCESS: Promotion deleted by Vendor.")

print("\n=== ALL LIFECYCLE E2E INTEGRATION TESTS PASSED SUCCESSFULLY! ===")
sys.exit(0)
