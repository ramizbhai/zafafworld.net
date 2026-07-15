import os
import uuid
import logging
import psycopg2
from psycopg2.extras import DictCursor
from minio import Minio
import difflib

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"
MINIO_ENDPOINT = '127.0.0.1:9000'
MINIO_ACCESS_KEY = 'zafaf_minio_admin'
MINIO_SECRET_KEY = 'zafaf_minio_secret'
MINIO_BUCKET = 'zafafworld-media'

def get_minio_client():
    return Minio(MINIO_ENDPOINT, access_key=MINIO_ACCESS_KEY, secret_key=MINIO_SECRET_KEY, secure=False)

def remap_minio_files():
    minio_client = get_minio_client()
    conn = psycopg2.connect(DB_DSN)
    conn.autocommit = False
    cur = conn.cursor(cursor_factory=DictCursor)
    
    # 1. List all vendors from DB
    cur.execute("SELECT id, name_ar, name_en FROM vendors")
    vendors = cur.fetchall()
    
    # 2. List all folders in MinIO under assets/uploads/vendors/
    objects = minio_client.list_objects(MINIO_BUCKET, prefix="assets/uploads/vendors/", recursive=True)
    
    # Group MinIO files by hotel_name
    minio_folders = {}
    for obj in objects:
        parts = obj.object_name.split('/')
        if len(parts) >= 5:
            hotel_name = parts[3]
            file_name = parts[-1]
            if hotel_name not in minio_folders:
                minio_folders[hotel_name] = []
            minio_folders[hotel_name].append((obj.object_name, obj.size, file_name))
            
    logging.info(f"Found {len(minio_folders)} hotel folders in MinIO.")
    
    minio_folder_names = list(minio_folders.keys())
    
    updated_rows = 0
    
    for vendor in vendors:
        vendor_id = vendor['id']
        name_ar = vendor['name_ar'] or ""
        name_en = vendor['name_en'] or ""
        
        # Try to find a matching folder
        best_match = None
        best_score = 0
        
        for folder in minio_folder_names:
            score_ar = difflib.SequenceMatcher(None, name_ar.lower(), folder.lower()).ratio()
            score_en = difflib.SequenceMatcher(None, name_en.lower(), folder.lower()).ratio()
            # Also check if folder name is a substring of the vendor name
            if folder.lower() in name_ar.lower() or folder.lower() in name_en.lower():
                score_ar = max(score_ar, 0.8)
            
            score = max(score_ar, score_en)
            if score > best_score:
                best_score = score
                best_match = folder
                
        if best_match and best_score > 0.4:
            logging.info(f"Matched vendor '{name_ar}' | '{name_en}' to folder '{best_match}' (score: {best_score:.2f})")
            files = minio_folders[best_match]
            
            # Separate thumbnails and main files
            thumbnails = [f for f in files if '_thumb' in f[2]]
            main_files = [f for f in files if '_thumb' not in f[2] and 'ZWI_' in f[2]]
            video_files = [f for f in files if 'ZWV_' in f[2]]
            
            # Fetch vendor_gallery rows
            cur.execute("SELECT id, is_cover, media_type FROM vendor_gallery WHERE vendor_id = %s ORDER BY is_cover DESC, created_at ASC", (vendor_id,))
            gallery_rows = cur.fetchall()
            
            if not gallery_rows:
                continue
                
            img_idx = 0
            vid_idx = 0
            
            for row in gallery_rows:
                row_id = row['id']
                media_type = row['media_type']
                
                assigned_file = None
                assigned_thumb = None
                
                if media_type == 'image' and img_idx < len(main_files):
                    assigned_file = main_files[img_idx]
                    # try to find corresponding thumb
                    base_uuid = assigned_file[2].replace('ZWI_', '').replace('.webp', '')
                    for t in thumbnails:
                        if base_uuid in t[2]:
                            assigned_thumb = t
                            break
                    img_idx += 1
                elif media_type == 'video' and vid_idx < len(video_files):
                    assigned_file = video_files[vid_idx]
                    vid_idx += 1
                    
                if assigned_file:
                    minio_path = assigned_file[0]
                    file_size = assigned_file[1]
                    basename = assigned_file[2]
                    file_url = f"/{minio_path}"
                    thumb_url = f"/{assigned_thumb[0]}" if assigned_thumb else None
                    mime_type = 'image/webp' if media_type == 'image' else 'video/mp4'
                    
                    try:
                        cur.execute("""
                            INSERT INTO uploaded_files (id, bucket_name, object_key, file_name, file_size, mime_type, status)
                            VALUES (%s, %s, %s, %s, %s, %s, 'ready')
                            ON CONFLICT (object_key) DO NOTHING
                        """, (row_id, MINIO_BUCKET, minio_path, basename, file_size, mime_type))
                        
                        cur.execute("""
                            UPDATE vendor_gallery 
                            SET image_url = %s, file_url = %s, thumbnail_url = %s 
                            WHERE id = %s
                        """, (file_url, file_url, thumb_url, row_id))
                        
                        updated_rows += 1
                    except Exception as e:
                        logging.error(f"Error updating DB for row {row_id}: {e}")
                        conn.rollback()
                        
    conn.commit()
    logging.info(f"Finished mapping. Updated {updated_rows} vendor_gallery rows.")
    conn.close()

if __name__ == '__main__':
    remap_minio_files()
