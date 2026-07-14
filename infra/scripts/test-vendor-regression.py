import os
import jwt
import time
import requests
import psycopg2
import sys
from uuid import uuid4

# Load centralized configurations
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

# Configuration
JWT_SECRET = env_vars.get("TEST_JWT_SECRET", "ce4c44b1ee349ed59a0beaada30a1913b3f30911adf9470f0a2dbbf3099565c0ff768f141ca07e1b94a54dc1053872ea54db252a0827d6bdc7d29961ccdeb2d4")
# Resolve database URL target
deploy_root = env_vars.get("DEPLOY_ROOT", "/opt/zafafworld.net")
master_env_file = os.path.join(deploy_root, ".env")
master_vars = {}
if os.path.exists(master_env_file):
    with open(master_env_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if '=' in line:
                k, v = line.split('=', 1)
                master_vars[k.strip()] = v.strip()

PGUSER = master_vars.get("POSTGRES_USER", "postgres")
PGPASS = master_vars.get("POSTGRES_PASSWORD", "postgres")
PGDB = master_vars.get("POSTGRES_DB", "zafaf_world")
DB_URL = f"postgres://{PGUSER}:{PGPASS}@127.0.0.1:5434/{PGDB}"
BASE_URL = "https://localhost/api/v1/vendor"

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

def main():
    vendor_a = ("edaef06d-3c79-4062-b21c-babb61223350", "Radisson@gmail.com")
    vendor_b = ("a0244784-3dbc-4257-a3b0-9570738e8188", "Al-Aseela@gmail.com")
    
    token_a = mint_jwt(vendor_a[0], vendor_a[1])
    token_b = mint_jwt(vendor_b[0], vendor_b[1])
    
    print(f"Vendor A: {vendor_a[0]}")
    print(f"Vendor B: {vendor_b[0]}")
    
    headers_a = {"Authorization": f"Bearer {token_a}"}
    headers_b = {"Authorization": f"Bearer {token_b}"}
    
    # 1. Products
    print("\n--- Testing Products ---")
    product_payload = {
        "title_en": "Test Product",
        "title_ar": "منتج تجريبي",
        "product_category": "wedding-gifts",
        "description_en": "A nice gift",
        "description_ar": "هدية جميلة",
        "price": 100.0,
        "location": "Riyadh",
        "features_en": ["Feature 1"],
        "features_ar": ["ميزة 1"],
        "status": "draft"
    }
    try:
        r = requests.post(f"{BASE_URL}/products", json=product_payload, headers=headers_a, verify=False, timeout=10)
        print("Create Product:", r.status_code, r.text)
        
        product_id = None
        if r.status_code == 200:
            product_id = r.json().get("data", {}).get("productId") or r.json().get("productId")
        
        r = requests.get(f"{BASE_URL}/products", headers=headers_a, verify=False, timeout=10)
        print("List Products:", r.status_code, r.text)
    except Exception as e:
        print("HTTP connection failed (backend offline?):", e)
        product_id = None
        
    # 3. Subscription
    print("\n--- Testing Subscription ---")
    try:
        r = requests.get(f"{BASE_URL}/subscription/requests", headers=headers_a, verify=False, timeout=10)
        print("Subscription Requests:", r.status_code, r.text)
    except Exception as e:
        print("Request failed:", e)
        
    # 4. Analytics
    print("\n--- Testing Analytics ---")
    try:
        r = requests.get(f"{BASE_URL}/analytics", headers=headers_a, verify=False, timeout=10)
        print("Analytics:", r.status_code, r.text)
    except Exception as e:
        print("Request failed:", e)
        
    # 5. Admin Support Chat
    print("\n--- Testing Admin Chat ---")
    try:
        r = requests.get(f"{BASE_URL}/tickets/messages", headers=headers_a, verify=False, timeout=10)
        print("Get Chat:", r.status_code, r.text)
    except Exception as e:
        print("Request failed:", e)
        
    # 6. Security Tests
    print("\n--- Testing Security Isolation ---")
    if product_id:
        print("Publishing product", product_id, "with Vendor B")
        try:
            r = requests.put(f"{BASE_URL}/products/{product_id}/publish", headers=headers_b, verify=False, timeout=10)
            print("Vendor B publishing Vendor A product:", r.status_code, r.text)
        except Exception as e:
            print("Request failed:", e)
    else:
        print("Skipping Security Isolation because product_id is None")

if __name__ == "__main__":
    main()
