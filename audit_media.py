import os
import json
from urllib.parse import unquote

db_keys_raw = [line.strip() for line in open('db_keys.txt') if line.strip()]
db_keys = set(db_keys_raw)

minio_keys = set()
with open('minio_json.txt') as f:
    for line in f:
        if line.strip():
            obj = json.loads(line)
            minio_keys.add(obj['key'])

gallery_rows = [line.strip() for line in open('gallery_keys.txt') if line.strip()]

# Dictionary of file_url to its row info
gallery_urls = {}
for row in gallery_rows:
    parts = row.split('|')
    if len(parts) >= 2:
        file_url = parts[0].strip()
        thumbnail_url = parts[1].strip()
        gallery_urls[file_url] = row

        
def to_minio_key(url):
    if not url: return None
    if url.startswith("http"):
        path = url.split("net/", 1)[-1] if "net/" in url else url
    else:
        path = url
    path = unquote(path)
    if path.startswith('/'):
        path = path[1:]
    return path

gallery_expected_keys = set()
broken_gallery_urls = set()

for file_url, row in gallery_urls.items():
    file_key = to_minio_key(file_url)
    thumb_url = row.split('|')[1].strip() if len(row.split('|')) > 1 else None
    thumb_key = to_minio_key(thumb_url)
    
    is_broken = False
    
    if file_key:
        gallery_expected_keys.add(file_key)
        if file_key not in minio_keys:
            is_broken = True
            
    if thumb_key:
        gallery_expected_keys.add(thumb_key)
        # We don't fail the whole gallery if just thumb is missing, but let's say if file_key is missing it's broken
        
    if is_broken:
        broken_gallery_urls.add(file_url)

orphaned_in_minio = minio_keys - db_keys - gallery_expected_keys
orphaned_in_db = db_keys - minio_keys

with open('cleanup.sql', 'w') as f:
    # 1. Delete broken vendor_gallery records
    for url in broken_gallery_urls:
        f.write(f"DELETE FROM vendor_gallery WHERE file_url = '{url}';\n")
    
    # 2. Delete orphaned uploaded_files
    for key in orphaned_in_db:
        # Avoid escaping issues
        safe_key = key.replace("'", "''")
        f.write(f"DELETE FROM uploaded_files WHERE object_key = '{safe_key}';\n")

with open('cleanup_minio.sh', 'w') as f:
    for key in orphaned_in_minio:
        f.write(f"podman exec zafafworld_minio_1 mc rm \"myminio/zafafworld-media/{key}\"\n")

print(f"Generated cleanup.sql with {len(broken_gallery_urls)} gallery deletes and {len(orphaned_in_db)} uploaded_files deletes.")
print(f"Generated cleanup_minio.sh with {len(orphaned_in_minio)} object deletes.")
