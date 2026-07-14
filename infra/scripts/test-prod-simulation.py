#!/usr/bin/env python3
"""
Phase 4 Production Simulation Test
Tests complete user journeys across Vendor Portal, Client Web, Admin Panel, and Backend.
"""
import os
import base64
import hmac
import hashlib
import json
import time
import requests
import sys
from uuid import uuid4
from datetime import datetime

def base64url_encode(payload):
    if isinstance(payload, dict):
        payload = json.dumps(payload).encode('utf-8')
    elif isinstance(payload, str):
        payload = payload.encode('utf-8')
    return base64.urlsafe_b64encode(payload).replace(b'=', b'').decode('utf-8')

class PyJWTStub:
    @staticmethod
    def encode(payload, secret, algorithm="HS256"):
        header = {"alg": "HS256", "typ": "JWT"}
        header_b64 = base64url_encode(header)
        payload_b64 = base64url_encode(payload)
        signing_input = f"{header_b64}.{payload_b64}".encode('utf-8')
        signature = hmac.new(secret.encode('utf-8'), signing_input, hashlib.sha256).digest()
        signature_b64 = base64.urlsafe_b64encode(signature).replace(b'=', b'').decode('utf-8')
        return f"{header_b64}.{payload_b64}.{signature_b64}"

jwt = PyJWTStub()

requests.packages.urllib3.disable_warnings()

# ═══════════════════════════════════════════════════════════════════════
# Configuration (Sourced from centralized infra/.env)
# ═══════════════════════════════════════════════════════════════════════
env_vars = {}
env_file = "/opt/zafafworld.net/infra/.env"
if os.path.exists(env_file):
    with open(env_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if '=' in line:
                k, v = line.split('=', 1)
                env_vars[k.strip()] = v.strip()

JWT_SECRET = env_vars.get("TEST_JWT_SECRET", "ce4c44b1ee349ed59a0beaada30a1913b3f30911adf9470f0a2dbbf3099565c0ff768f141ca07e1b94a54dc1053872ea54db252a0827d6bdc7d29961ccdeb2d4")
BASE = "https://localhost:8443"
API = f"{BASE}/api/v1"

# Known production users
VENDOR_A_USER = "5c2f696d-a0a1-41d7-bc3d-b8eb17503563"
VENDOR_A_EMAIL = "vendor@test.com"
VENDOR_B_USER = "d3ef96be-1d85-4bd7-b0e9-1ab78992f4d1"
VENDOR_B_EMAIL = "prod_vendor@test.com"
ADMIN_USER = "11111111-1111-1111-1111-111111111111"
ADMIN_EMAIL = "afrah@zafafworld.com"
CLIENT_USER = "6e6f4a14-1fc8-4a77-bec1-d4bfcb58c1ec"
CLIENT_EMAIL = "testvendor@zafafworld.net"

# Known active product
ACTIVE_PRODUCT_ID = "f40b82d2-f44e-42e0-9ef9-8777b3e076ea"
ACTIVE_VENDOR_SLUG = "jouri-ballroom"

results = []

def mint_jwt(user_id, email, role="Vendor"):
    now = int(time.time())
    payload = {
        "sub": str(user_id),
        "email": email,
        "role": role,
        "scopes": ["owner"],
        "exp": now + 3600,
        "iat": now
    }
    return jwt.encode(payload, JWT_SECRET, algorithm="HS256")

def test(name, method, url, headers=None, json_data=None, files=None, expected_status=None, check_fields=None, host_override=None):
    try:
        req_headers = headers or {}
        if host_override:
            req_headers["Host"] = host_override

        if method == "GET":
            r = requests.get(url, headers=req_headers, verify=False, timeout=10)
        elif method == "POST":
            if files:
                r = requests.post(url, headers=req_headers, files=files, verify=False, timeout=10)
            else:
                r = requests.post(url, headers=req_headers, json=json_data, verify=False, timeout=10)
        elif method == "PUT":
            r = requests.put(url, headers=req_headers, json=json_data, verify=False, timeout=10)
        elif method == "PATCH":
            r = requests.patch(url, headers=req_headers, json=json_data, verify=False, timeout=10)
        elif method == "DELETE":
            r = requests.delete(url, headers=req_headers, verify=False, timeout=10)
        else:
            results.append({"name": name, "status": "FAIL", "detail": f"Unknown method {method}"})
            return None

        status_ok = True
        detail = f"HTTP {r.status_code}"

        if expected_status:
            if isinstance(expected_status, list):
                status_ok = r.status_code in expected_status
            else:
                status_ok = r.status_code == expected_status

        body = None
        try:
            body = r.json()
        except:
            body = r.text[:200] if r.text else ""

        field_ok = True
        if check_fields and isinstance(body, dict):
            missing = [f for f in check_fields if f not in body]
            if missing:
                field_ok = False
                detail += f" | MISSING fields: {missing}"

        error_indicators = False
        is_expected_error = expected_status and (
            (isinstance(expected_status, list) and any(s >= 400 for s in expected_status)) or 
            (isinstance(expected_status, int) and expected_status >= 400)
        )
        
        if isinstance(body, dict) and not is_expected_error:
            if body.get("error_type") == "DATABASE_ERROR":
                error_indicators = True
                detail += " | DATABASE_ERROR"
            if body.get("status") == "error":
                error_indicators = True
                detail += f" | error: {body.get('message', '')[:80]}"

        passed = status_ok and field_ok and not error_indicators
        result = {
            "name": name,
            "status": "PASS" if passed else "FAIL",
            "http_status": r.status_code,
            "detail": detail,
            "response_ms": r.elapsed.total_seconds() * 1000,
        }
        results.append(result)

        icon = "✅" if passed else "❌"
        print(f"  {icon} {name}: {r.status_code} ({r.elapsed.total_seconds()*1000:.0f}ms)")
        if not passed:
            snippet = json.dumps(body, ensure_ascii=False)[:200] if isinstance(body, dict) else str(body)[:200]
            print(f"     ↳ {detail}")
            print(f"     ↳ Body: {snippet}")

        return body
    except Exception as e:
        results.append({"name": name, "status": "FAIL", "detail": str(e)})
        print(f"  ❌ {name}: EXCEPTION - {e}")
        return None

def test_ssr_page(name, url, expected_status=200, host_override=None):
    try:
        headers = {}
        if host_override:
            headers["Host"] = host_override
        
        r = requests.get(url, headers=headers, verify=False, timeout=15, allow_redirects=True)
        passed = r.status_code == expected_status
        result = {
            "name": name,
            "status": "PASS" if passed else "FAIL",
            "http_status": r.status_code,
            "detail": f"HTTP {r.status_code} | {len(r.text)} bytes",
            "response_ms": r.elapsed.total_seconds() * 1000,
        }
        results.append(result)
        icon = "✅" if passed else "❌"
        print(f"  {icon} {name}: {r.status_code} ({r.elapsed.total_seconds()*1000:.0f}ms) [{len(r.text)} bytes]")
        return r
    except Exception as e:
        results.append({"name": name, "status": "FAIL", "detail": str(e)})
        print(f"  ❌ {name}: EXCEPTION - {e}")
        return None

# ═══════════════════════════════════════════════════════════════════════
# Mint tokens
# ═══════════════════════════════════════════════════════════════════════
vendor_a_token = mint_jwt(VENDOR_A_USER, VENDOR_A_EMAIL, "Vendor")
vendor_b_token = mint_jwt(VENDOR_B_USER, VENDOR_B_EMAIL, "Vendor")
admin_token = mint_jwt(ADMIN_USER, ADMIN_EMAIL, "Admin")
client_token = mint_jwt(CLIENT_USER, CLIENT_EMAIL, "Client")

headers_vendor_a = {"Authorization": f"Bearer {vendor_a_token}"}
headers_vendor_b = {"Authorization": f"Bearer {vendor_b_token}"}
headers_admin = {"Authorization": f"Bearer {admin_token}"}
headers_client = {"Authorization": f"Bearer {client_token}"}

print("=" * 70)
print("  PHASE 4 PRODUCTION SIMULATION TEST")
print(f"  {datetime.now().isoformat()}")
print("=" * 70)

# ═══════════════════════════════════════════════════════════════════════
# 1. VENDOR PORTAL JOURNEY
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "━" * 70)
print("  1. VENDOR PORTAL JOURNEY")
print("━" * 70)

# 1.1 Dashboard
print("\n  ── 1.1 Dashboard ──")
test("Vendor Dashboard Stats", "GET", f"{API}/vendor/stats/dashboard", headers_vendor_a, expected_status=200)

# 1.2 Manage Listings
print("\n  ── 1.2 Manage Listings ──")
body = test("List Vendor Products", "GET", f"{API}/vendor/products", headers_vendor_a,
     expected_status=200, check_fields=["products", "status", "total"])

# 1.3 Edit Listing (Get Edit Context)
print("\n  ── 1.3 Edit Listing ──")
test("Get Edit Context", "GET", f"{API}/vendor/products/{ACTIVE_PRODUCT_ID}/edit-context", headers_vendor_a,
     expected_status=200)

# 1.4 Create Listing (Draft)
print("\n  ── 1.4 Create Listing (Draft) ──")
create_body = test("Create Draft Listing", "POST", f"{API}/vendor/products", headers_vendor_a,
     json_data={
         "title_en": "Production Sim Test Venue",
         "title_ar": "مكان اختبار المحاكاة",
         "product_category": "wedding-palace",
         "description_en": "Test venue for production simulation",
         "description_ar": "مكان اختبار لمحاكاة الإنتاج",
         "price": 500.0,
         "location": "Riyadh",
         "features_en": ["WiFi", "Parking"],
         "features_ar": ["واي فاي", "موقف سيارات"],
         "status": "draft"
     },
     expected_status=200, check_fields=["productId"])

new_product_id = None
if create_body:
    new_product_id = create_body.get("productId")

# 1.5 Update Listing (Edit)
print("\n  ── 1.5 Update Listing ──")
if new_product_id:
    test("Update Draft Listing", "PUT", f"{API}/vendor/products/{new_product_id}", headers_vendor_a,
         json_data={
             "title_en": "Production Sim Test Venue (Updated)",
             "title_ar": "مكان اختبار المحاكاة (محدث)",
             "product_category": "wedding-palace",
             "description_en": "Updated test venue for production simulation",
             "description_ar": "مكان اختبار محدث لمحاكاة الإنتاج",
             "price": 600.0,
             "location": "Riyadh",
             "features_en": ["WiFi", "Parking", "Valet"],
             "features_ar": ["واي فاي", "موقف سيارات", "خدمة صف السيارات"],
             "version": 1
         },
         expected_status=200)
else:
    print("  ⚠️  Skipping update — no product ID from creation")

# 1.6 Publish Listing
print("\n  ── 1.6 Publish Listing ──")
if new_product_id:
    test("Publish Draft Listing", "PATCH", f"{API}/vendor/products/{new_product_id}/status", headers_vendor_a,
         json_data={"status": "pending_approval"},
         expected_status=200)
else:
    print("  ⚠️  Skipping publish — no product ID")

# 1.7 Gallery
print("\n  ── 1.7 Gallery ──")
test("List Gallery", "GET", f"{API}/vendor/gallery", headers_vendor_a, expected_status=200)

# 1.8 Packages
print("\n  ── 1.8 Packages ──")
test("List Packages", "GET", f"{API}/vendor/packages", headers_vendor_a, expected_status=200)

# 1.9 Subscription
print("\n  ── 1.9 Subscription ──")
test("List Subscription Requests", "GET", f"{API}/vendor/subscription/requests", headers_vendor_a,
     expected_status=200, check_fields=["requests", "status"])

# 1.10 Analytics
print("\n  ── 1.10 Analytics ──")
test("Get Analytics", "GET", f"{API}/vendor/analytics", headers_vendor_a,
     expected_status=200, check_fields=["data", "status"])

# 1.11 Support Chat
print("\n  ── 1.11 Support Chat ──")
chat_body = test("Get Chat Messages", "GET", f"{API}/vendor/tickets/messages", headers_vendor_a,
     expected_status=200, check_fields=["chat_id", "messages", "status"])

test("Send Chat Reply", "POST", f"{API}/vendor/tickets/reply", headers_vendor_a,
     files={"body": (None, "Production simulation test message")},
     expected_status=200)

test("Get Chat After Reply", "GET", f"{API}/vendor/tickets/messages", headers_vendor_a,
     expected_status=200)

# 1.12 Inquiries
print("\n  ── 1.12 Inquiries ──")
test("List Inquiries", "GET", f"{API}/vendor/inquiries", headers_vendor_a, expected_status=200)

# 1.13 Bookings
print("\n  ── 1.13 Bookings ──")
test("List Bookings", "GET", f"{API}/vendor/bookings", headers_vendor_a, expected_status=200)

# 1.14 Notifications
print("\n  ── 1.14 Notifications ──")
test("List Notifications", "GET", f"{API}/vendor/notifications", headers_vendor_a, expected_status=200)

# 1.15 Wallet
print("\n  ── 1.15 Wallet ──")
test("Get Wallet", "GET", f"{API}/vendor/wallet", headers_vendor_a, expected_status=200)

# ═══════════════════════════════════════════════════════════════════════
# 2. SECURITY / RLS ISOLATION
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "━" * 70)
print("  2. SECURITY / RLS ISOLATION")
print("━" * 70)

if new_product_id:
    test("RLS: Vendor B cannot edit Vendor A product", "PUT",
         f"{API}/vendor/products/{new_product_id}", headers_vendor_b,
         json_data={"title_en": "Hacked!", "title_ar": "مخترق!", "product_category": "wedding-palace",
                    "description_en": "hacked", "description_ar": "مخترق", "version": 1},
         expected_status=[403, 404])

    test("RLS: Vendor B cannot publish Vendor A product", "PATCH",
         f"{API}/vendor/products/{new_product_id}/status", headers_vendor_b,
         json_data={"status": "pending_approval"},
         expected_status=[403, 404])

# Expired token
print("\n  ── Expired Token Test ──")
expired_payload = {
    "sub": VENDOR_A_USER,
    "email": VENDOR_A_EMAIL,
    "role": "Vendor",
    "scopes": ["owner"],
    "exp": int(time.time()) - 3600,
    "iat": int(time.time()) - 7200
}
expired_token = PyJWTStub.encode(expired_payload, JWT_SECRET, algorithm="HS256")
test("Expired Token Rejected", "GET", f"{API}/vendor/products",
     headers={"Authorization": f"Bearer {expired_token}"},
     expected_status=401)

test("No Token Rejected", "GET", f"{API}/vendor/products", expected_status=401)

# ═══════════════════════════════════════════════════════════════════════
# 3. CLIENT WEB JOURNEY
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "━" * 70)
print("  3. CLIENT WEB JOURNEY")
print("━" * 70)

# 3.1 Public endpoints
print("\n  ── 3.1 Public API Endpoints ──")
test("Public: List Categories", "GET", f"{API}/public/categories", expected_status=200)
test("Public: List Cities", "GET", f"{API}/public/cities", expected_status=200)
test("Public: List Amenities", "GET", f"{API}/public/amenities", expected_status=200)
test("Public: List Venue Types", "GET", f"{API}/public/venue-types", expected_status=200)

# 3.2 Vendor listing / Search / Filter
print("\n  ── 3.2 Vendor Listing / Search / Filter ──")
test("Public: List Vendors (all)", "GET", f"{API}/public/vendors", expected_status=200)
test("Public: Search Vendors", "GET", f"{API}/public/vendors?search=Radisson", expected_status=200)
test("Public: Filter Vendors by Category", "GET", f"{API}/public/vendors?category=hotel-venue", expected_status=200)

# 3.3 Vendor detail page
print("\n  ── 3.3 Vendor Detail ──")
public_vendors_response = test("Public: List Vendors (all)", "GET", f"{API}/public/vendors", expected_status=200)
dynamic_vendor_slug = ACTIVE_VENDOR_SLUG
if public_vendors_response and "vendors" in public_vendors_response and len(public_vendors_response["vendors"]) > 0:
    dynamic_vendor_slug = public_vendors_response["vendors"][0].get("slug", ACTIVE_VENDOR_SLUG)

test("Public: Vendor Detail by Slug", "GET", f"{API}/public/vendors/{dynamic_vendor_slug}",
     expected_status=200)

# 3.4 Inquiry Submit
print("\n  ── 3.4 Client Inquiry ──")
test("Client: Submit Inquiry", "POST", f"{API}/public/inquiries", headers_client,
     json_data={
         "vendor_id": ACTIVE_PRODUCT_ID,
         "listing_id": ACTIVE_PRODUCT_ID,
         "message": "Production simulation inquiry test",
         "eventDate": "2026-12-15",
         "guestCount": 200,
         "customer_name": "Test Client",
         "phone": "0555555555",
         "wedding_date": "2026-12-15"
     },
     expected_status=[200, 201, 400])

# 3.5 SSR Page Loads
print("\n  ── 3.5 SSR Page Loads (Client Web) ──")
test_ssr_page("Client Homepage", f"{BASE}", host_override="zafafworld.net")
test_ssr_page("Client Vendor Listing", f"{BASE}/vendors", host_override="zafafworld.net")

# ═══════════════════════════════════════════════════════════════════════
# 4. ADMIN PANEL JOURNEY
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "━" * 70)
print("  4. ADMIN PANEL JOURNEY")
print("━" * 70)

# 4.1 Dashboard
print("\n  ── 4.1 Admin Dashboard ──")
test("Admin: Dashboard Context", "GET", f"{API}/admin/vendors-context", headers_admin,
     expected_status=200)

# 4.2 Vendor Management
print("\n  ── 4.2 Vendor Management ──")
test("Admin: List Vendors", "GET", f"{API}/admin/vendors", headers_admin,
     expected_status=200)

# 4.3 Listing Management
print("\n  ── 4.3 Listing Management ──")
test("Admin: List Pending Listings", "GET", f"{API}/admin/listings?status=pending", headers_admin,
     expected_status=200)
test("Admin: List All Listings", "GET", f"{API}/admin/listings", headers_admin,
     expected_status=200)

# ═══════════════════════════════════════════════════════════════════════
# 5. CLEANUP
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "━" * 70)
print("  5. CLEANUP")
print("━" * 70)
if new_product_id:
    test("Delete Test Product", "DELETE", f"{API}/vendor/products/{new_product_id}", headers_vendor_a,
         expected_status=[200, 204])

# ═══════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════
print("\n" + "=" * 70)
print("  RESULTS SUMMARY")
print("=" * 70)

passed = sum(1 for r in results if r["status"] == "PASS")
failed = sum(1 for r in results if r["status"] == "FAIL")
total = len(results)

print(f"\n  Total: {total}  |  ✅ Passed: {passed}  |  ❌ Failed: {failed}")
print()

if failed > 0:
    print("  FAILED TESTS:")
    print("  " + "-" * 60)
    for r in results:
        if r["status"] == "FAIL":
            print(f"    ❌ {r['name']}: {r.get('detail', 'unknown')}")

print()

deploy_root = env_vars.get("DEPLOY_ROOT", "/opt/zafafworld.net")
report_path = os.path.join(deploy_root, "infra/scripts/simulation_results.json")
with open(report_path, "w") as f:
    json.dump({
        "timestamp": datetime.now().isoformat(),
        "total": total,
        "passed": passed,
        "failed": failed,
        "results": results
    }, f, indent=2, default=str)

print(f"  Results saved to {report_path}")
print("=" * 70)

sys.exit(1 if failed > 0 else 0)
