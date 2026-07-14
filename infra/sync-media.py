#!/usr/bin/env python3
import os
import sys
import json
import hashlib
import subprocess

# Load env variables from centralized /opt/zafafworld.net/infra/.env
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
                env_vars[k.strip()] = v.strip().strip('"').strip("'")

# MinIO Config
endpoint = env_vars.get("MINIO_ENDPOINT", "http://minio:9000")
bucket = env_vars.get("MINIO_BUCKET", "zafafworld-media")
# Sourcing credentials from master project env if available
master_env_file = "/opt/zafafworld.net/.env"
if os.path.exists(master_env_file):
    with open(master_env_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if '=' in line:
                k, v = line.split('=', 1)
                env_vars[k.strip()] = v.strip().strip('"').strip("'")

root_user = env_vars.get("MINIO_ROOT_USER", "zafaf_minio_admin")
root_password = env_vars.get("MINIO_ROOT_PASSWORD", "zafafminiosupersecurepass2026")

# Local directory for uploads
local_dir = env_vars.get("UPLOADS_VOLUME_DIR", "/var/lib/zafafworld/uploads")

# Function to run containerized mc
def run_mc_json(cmd):
    docker_cmd = [
        "podman", "run", "--rm", "--net", "zafafworld_zafaf_network", "--entrypoint", "sh",
        "-v", f"{local_dir}:/var/www/uploads:ro,Z",
        "docker.io/minio/mc", "-c",
        f"mc alias set myminio {endpoint} {root_user} {root_password} >/dev/null 2>&1 && {cmd}"
    ]
    try:
        output = subprocess.check_output(docker_cmd)
        return output.decode()
    except Exception as e:
        print(f"Error running mc command: {e}")
        return ""

def calculate_md5(filepath):
    hasher = hashlib.md5()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hasher.update(chunk)
    return hasher.hexdigest()

def audit():
    print("=================================================================")
    print("  Media Stack Consistency Audit (Local Volume vs MinIO S3)")
    print("=================================================================")
    
    # 1. Scan local files
    local_files = {}
    total_local_size = 0
    for root, dirs, files in os.walk(local_dir):
        if '/temp/' in root or root.endswith('/temp'):
            continue
        for file in files:
            if file == ".keep":
                continue
            filepath = os.path.join(root, file)
            rel_path = os.path.relpath(filepath, local_dir)
            size = os.path.getsize(filepath)
            local_files[rel_path] = {
                "path": filepath,
                "size": size
            }
            total_local_size += size
            
    # 2. Fetch S3 objects
    s3_json = run_mc_json(f"mc ls -r --json myminio/{bucket}")
    s3_objects = {}
    total_s3_size = 0
    for line in s3_json.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
            if data.get('status') == 'success' and data.get('type') == 'file':
                key = data.get('key', '')
                if '.keep' in key:
                    continue
                size = data.get('size', 0)
                etag = data.get('etag', '').strip('"')
                s3_objects[key] = {
                    "size": size,
                    "etag": etag
                }
                total_s3_size += size
        except Exception:
            pass
            
    # 3. Perform Comparison
    missing = []
    mismatched = []
    
    for rel_path, info in local_files.items():
        if rel_path not in s3_objects:
            missing.append(rel_path)
        else:
            s3_info = s3_objects[rel_path]
            # Check size
            if info["size"] != s3_info["size"]:
                mismatched.append((rel_path, "Size mismatch"))
            else:
                local_md5 = calculate_md5(info["path"])
                if local_md5 != s3_info["etag"]:
                    mismatched.append((rel_path, "Checksum mismatch"))

    print(f"Local Files: {len(local_files)} ({total_local_size / 1024:.1f} KiB)")
    print(f"S3 Objects:  {len(s3_objects)} ({total_s3_size / 1024:.1f} KiB)")
    print("\nAnalyzing directory structures...")
    
    if not missing and not mismatched:
        print("-----------------------------------------------------------------")
        print("  Result: MATCH (All local files are synced in MinIO)")
        print("=================================================================")
        return True
    else:
        if missing:
            print("\nMissing files detected in MinIO:")
            for m in missing:
                print(f"  - [MISSING] {m}")
        if mismatched:
            print("\nMismatched files detected in MinIO:")
            for m, reason in mismatched:
                print(f"  - [MISMATCH] {m} ({reason})")
        print("-----------------------------------------------------------------")
        print("  Result: MISMATCH")
        print("  Run repair to sync.")
        print("=================================================================")
        return False

def repair(execute=False):
    print("=================================================================")
    print("  Media Stack Consistency Repair (Local Volume -> MinIO S3)")
    print("=================================================================")
    
    if not execute:
        print("DEFAULT SAFE MODE: Running in DRY-RUN mode. Pass --execute to run live.")
        cmd = f"mc mirror -a --overwrite --fake /var/www/uploads myminio/{bucket}"
    else:
        print("Running live mirror repair...")
        cmd = f"mc mirror -a --overwrite /var/www/uploads myminio/{bucket}"
        
    result = run_mc_json(cmd)
    print(result)
    print("Sync completed successfully.")
    print("=================================================================")

if __name__ == "__main__":
    mode = "audit"
    execute = False
    
    args = sys.argv[1:]
    if len(args) > 0:
        if args[0] in ["audit", "repair"]:
            mode = args[0]
            args = args[1:]
            
    # Check for execute/dry-run flags
    for arg in args:
        if arg in ["--execute", "--force"]:
            execute = True
        elif arg == "--dry-run":
            execute = False
            
    if mode == "audit":
        success = audit()
        sys.exit(0 if success else 1)
    elif mode == "repair":
        repair(execute=execute)
    else:
        print(f"Usage: {sys.argv[0]} {{audit|repair}} [--execute]")
        sys.exit(1)
