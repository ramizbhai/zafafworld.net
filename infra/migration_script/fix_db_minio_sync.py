import os
import uuid
import logging
import psycopg2
from psycopg2.extras import DictCursor, execute_values
from PIL import Image
from minio import Minio
import io

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"
MINIO_ENDPOINT = '127.0.0.1:9000'
MINIO_ACCESS_KEY = 'zafaf_minio_admin'
MINIO_SECRET_KEY = 'zafaf_minio_secret'
MINIO_BUCKET = 'zafafworld-media'
UPLOADS_DIR = "/opt/zafafworld.net/zafaf-backups/uploads"

def get_minio_client():
    return Minio(MINIO_ENDPOINT, access_key=MINIO_ACCESS_KEY, secret_key=MINIO_SECRET_KEY, secure=False)

def process_image(file_path):
    try:
        with Image.open(file_path) as img:
            if img.mode in ("RGBA", "P"): img = img.convert("RGB")
            
            img_io = io.BytesIO()
            img.save(img_io, format='WEBP', quality=85)
            img_bytes = img_io.getvalue()
            
            img.thumbnail((400, 400))
            thumb_io = io.BytesIO()
            img.save(thumb_io, format='WEBP', quality=85)
            thumb_bytes = thumb_io.getvalue()
            
            return img_bytes, len(img_bytes), thumb_bytes, len(thumb_bytes)
    except Exception as e:
        logging.error(f"Image processing failed for {file_path}: {str(e)}")
        return None, 0, None, 0

def process_video(file_path):
    try:
        with open(file_path, 'rb') as f:
            data = f.read()
        return data, len(data)
    except Exception as e:
        return None, 0

def build_file_map(uploads_dir):
    file_map = {}
    for root, dirs, files in os.walk(uploads_dir):
        for f in files:
            file_map[f] = os.path.join(root, f)
    return file_map

def fix_vendor_gallery():
    minio_client = get_minio_client()
    file_map = build_file_map(UPLOADS_DIR)
    
    conn = psycopg2.connect(DB_DSN)
    conn.autocommit = False
    cur = conn.cursor(cursor_factory=DictCursor)
    
    cur.execute("SELECT id, vendor_id, image_url, file_path, media_type FROM vendor_gallery")
    rows = cur.fetchall()
    
    updated = 0
    missing = 0
    
    for row in rows:
        row_id = row['id']
        old_image_url = row.get('image_url')
        if not old_image_url: continue
        
        basename = os.path.basename(old_image_url)
        local_path = file_map.get(basename)
        
        if not local_path:
            missing += 1
            continue
            
        hotel_name = os.path.basename(os.path.dirname(local_path))
        file_ext = os.path.splitext(local_path)[1].lower()
        media_type = row.get('media_type', 'image')
        
        new_file_url = None
        new_thumb_url = None
        file_size = 0
        minio_path = ""
        
        if media_type == 'image' or file_ext in ['.jpg', '.jpeg', '.png', '.webp']:
            img_bytes, img_size, thumb_bytes, thumb_size = process_image(local_path)
            if img_bytes:
                minio_filename = f"ZWI_{row_id}.webp"
                minio_thumb = f"ZWI_{row_id}_thumb.webp"
                minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                minio_thumb_path = f"assets/uploads/vendors/{hotel_name}/{minio_thumb}"
                
                minio_client.put_object(MINIO_BUCKET, minio_path, io.BytesIO(img_bytes), img_size, content_type="image/webp")
                minio_client.put_object(MINIO_BUCKET, minio_thumb_path, io.BytesIO(thumb_bytes), thumb_size, content_type="image/webp")
                
                new_file_url = f"/{minio_path}"
                new_thumb_url = f"/{minio_thumb_path}"
                file_size = img_size
        else:
            vid_bytes, vid_size = process_video(local_path)
            if vid_bytes:
                minio_filename = f"ZWV_{row_id}{file_ext}"
                minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                minio_client.put_object(MINIO_BUCKET, minio_path, io.BytesIO(vid_bytes), vid_size, content_type="video/mp4")
                new_file_url = f"/{minio_path}"
                file_size = vid_size

        if new_file_url:
            try:
                # Insert into uploaded_files
                cur.execute("""
                    INSERT INTO uploaded_files (id, bucket_name, object_key, file_name, file_size, mime_type, status)
                    VALUES (%s, %s, %s, %s, %s, %s, 'ready')
                    ON CONFLICT (object_key) DO NOTHING
                """, (row_id, MINIO_BUCKET, minio_path, basename, file_size, 'image/webp' if media_type == 'image' else 'video/mp4'))
                
                # Update vendor_gallery
                cur.execute("""
                    UPDATE vendor_gallery 
                    SET image_url = %s, file_url = %s, thumbnail_url = %s 
                    WHERE id = %s
                """, (new_file_url, new_file_url, new_thumb_url, row_id))
                
                updated += 1
                if updated % 50 == 0:
                    conn.commit()
                    logging.info(f"Processed {updated} rows...")
            except psycopg2.Error as e:
                logging.error(f"Error processing row {row_id}: {e}")
                conn.rollback()

    conn.commit()
    logging.info(f"Finished updating {updated} rows. Missing: {missing}")

if __name__ == '__main__':
    fix_vendor_gallery()
