import os
import sys
import uuid
import logging
import argparse
import psycopg2
from psycopg2.extras import DictCursor, execute_values
from PIL import Image
from minio import Minio
from minio.error import S3Error
from dotenv import load_dotenv
import io

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout)
    ]
)
error_logger = logging.getLogger('error_logger')
error_logger.setLevel(logging.ERROR)
error_handler = logging.FileHandler('migration_errors.log')
error_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))
error_logger.addHandler(error_handler)

# Load environment
load_dotenv('/opt/zafafworld.net/.env')

# Database configs
DB_LEGACY_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world_legacy"
DB_NEW_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"

# MinIO configs
MINIO_ENDPOINT = os.getenv('MINIO_ENDPOINT', '127.0.0.1:9000').replace('http://', '').replace('https://', '')
if MINIO_ENDPOINT.startswith("minio:9000"):
    MINIO_ENDPOINT = "127.0.0.1:9000"
    
MINIO_ACCESS_KEY = os.getenv('MINIO_APP_USER', 'zafaf_minio_admin')
MINIO_SECRET_KEY = os.getenv('MINIO_APP_PASSWORD', 'zafaf_minio_secret')
MINIO_BUCKET = os.getenv('MINIO_BUCKET', 'zafafworld-media')

UPLOADS_DIR = "/opt/zafafworld.net/zafaf-backups/uploads"

BATCH_SIZE = 50

def get_minio_client():
    return Minio(
        MINIO_ENDPOINT,
        access_key=MINIO_ACCESS_KEY,
        secret_key=MINIO_SECRET_KEY,
        secure=False
    )

def ensure_bucket_exists(client):
    try:
        if not client.bucket_exists(MINIO_BUCKET):
            client.make_bucket(MINIO_BUCKET)
    except S3Error as e:
        error_logger.error(f"MinIO Bucket Error: {e}")
        raise

def process_image(file_path):
    try:
        with Image.open(file_path) as img:
            if img.mode in ("RGBA", "P"):
                img = img.convert("RGB")
            
            img_io = io.BytesIO()
            img.save(img_io, format='WEBP', quality=85)
            img_bytes = img_io.getvalue()
            
            img.thumbnail((400, 400))
            thumb_io = io.BytesIO()
            img.save(thumb_io, format='WEBP', quality=85)
            thumb_bytes = thumb_io.getvalue()
            
            return img_bytes, len(img_bytes), thumb_bytes, len(thumb_bytes)
    except Exception as e:
        error_logger.error(f"Image processing failed for {file_path}: {str(e)}")
        return None, 0, None, 0

def process_video(file_path):
    try:
        with open(file_path, 'rb') as f:
            data = f.read()
        return data, len(data)
    except Exception as e:
        error_logger.error(f"Video processing failed for {file_path}: {str(e)}")
        return None, 0

def migrate_vendor_products(conn_legacy, conn_new, is_dry_run):
    logging.info("Migrating vendor_products...")
    cur_legacy = conn_legacy.cursor(cursor_factory=DictCursor)
    cur_new = conn_new.cursor()
    
    cur_legacy.execute("SELECT * FROM vendor_products")
    
    insert_query = """
    INSERT INTO vendor_products (
        id, vendor_id, slug, product_category, base_price_sar, deposit_percentage,
        coordinator_phone, coordinator_whatsapp, coordinator_avatar, coordinator_gender,
        crm_product_id, city_id, status, rejection_reason, pre_suspension_status,
        is_available, is_featured, version, created_at, updated_at,
        title_ar, title_en, description_ar, description_en, coordinator_name,
        attributes, featured_until, title
    ) VALUES %s
    ON CONFLICT (id) DO NOTHING
    """
    
    batch = []
    total_migrated = 0
    
    for row in cur_legacy:
        title = row.get('title') or ''
        desc = row.get('description') or ''
        
        batch.append((
            row['id'], row['vendor_id'], row['slug'], row['product_category'], row['base_price_sar'], row['deposit_percentage'],
            row['coordinator_phone'], row['coordinator_whatsapp'], row['coordinator_avatar'], row['coordinator_gender'],
            row['crm_product_id'], row['city_id'], row['status'], row['rejection_reason'], row['pre_suspension_status'],
            row['is_available'], row['is_featured'], row['version'], row['created_at'], row['updated_at'],
            title, None, desc, None, row['coordinator_name'],
            psycopg2.extras.Json(row['attributes']) if row['attributes'] is not None else None, row['featured_until'], title
        ))
        
        if len(batch) >= BATCH_SIZE:
            if not is_dry_run:
                execute_values(cur_new, insert_query, batch)
            total_migrated += len(batch)
            batch = []
            
    if batch:
        if not is_dry_run:
            execute_values(cur_new, insert_query, batch)
        total_migrated += len(batch)
        
    logging.info(f"Migrated {total_migrated} vendor_products")

def build_file_map(uploads_dir):
    file_map = {}
    for root, dirs, files in os.walk(uploads_dir):
        for f in files:
            file_map[f] = os.path.join(root, f)
    return file_map

def migrate_vendor_gallery(conn_legacy, conn_new, minio_client, file_map, is_dry_run):
    logging.info("Migrating vendor_gallery...")
    cur_legacy = conn_legacy.cursor(cursor_factory=DictCursor)
    cur_new = conn_new.cursor()
    
    cur_legacy.execute("SELECT * FROM vendor_gallery")
    
    insert_query = """
    INSERT INTO vendor_gallery (
        id, vendor_id, product_id, image_url, file_path, is_cover, sort_order,
        caption, created_at, media_type, file_url, thumbnail_url, file_size, duration_seconds
    ) VALUES %s
    ON CONFLICT (id) DO NOTHING
    """
    
    batch = []
    total_migrated = 0
    not_found = 0
    
    for row in cur_legacy:
        old_image_url = row.get('image_url')
        if not old_image_url:
            continue
            
        basename = os.path.basename(old_image_url)
        local_path = file_map.get(basename)
        
        if not local_path:
            error_logger.error(f"File not found in uploads mapping: {old_image_url} (row id: {row['id']})")
            not_found += 1
            new_file_url = old_image_url
            new_image_url = old_image_url
            new_thumbnail_url = None
            file_size = 0
            duration = 0
        else:
            hotel_name = os.path.basename(os.path.dirname(local_path))
            file_ext = os.path.splitext(local_path)[1].lower()
            media_type = row.get('media_type', 'image')
            
            file_id = str(uuid.uuid4())
            duration = 0
            
            if media_type == 'image' or file_ext in ['.jpg', '.jpeg', '.png', '.webp']:
                img_bytes, img_size, thumb_bytes, thumb_size = process_image(local_path)
                if img_bytes:
                    minio_filename = f"ZWI_{file_id}.webp"
                    minio_thumb = f"ZWI_{file_id}_thumb.webp"
                    minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                    minio_thumb_path = f"assets/uploads/vendors/{hotel_name}/{minio_thumb}"
                    
                    if not is_dry_run:
                        minio_client.put_object(
                            MINIO_BUCKET, minio_path, io.BytesIO(img_bytes), img_size, content_type="image/webp"
                        )
                        minio_client.put_object(
                            MINIO_BUCKET, minio_thumb_path, io.BytesIO(thumb_bytes), thumb_size, content_type="image/webp"
                        )
                    
                    new_file_url = f"/{minio_path}"
                    new_image_url = f"/{minio_path}"
                    new_thumbnail_url = f"/{minio_thumb_path}"
                    file_size = img_size
                else:
                    new_file_url = old_image_url
                    new_image_url = old_image_url
                    new_thumbnail_url = None
                    file_size = 0
            else:
                vid_bytes, vid_size = process_video(local_path)
                if vid_bytes:
                    minio_filename = f"ZWV_{file_id}{file_ext}"
                    minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                    
                    if not is_dry_run:
                        minio_client.put_object(
                            MINIO_BUCKET, minio_path, io.BytesIO(vid_bytes), vid_size, content_type="video/mp4"
                        )
                    
                    new_file_url = f"/{minio_path}"
                    new_image_url = f"/{minio_path}"
                    new_thumbnail_url = None
                    file_size = vid_size
                else:
                    new_file_url = old_image_url
                    new_image_url = old_image_url
                    new_thumbnail_url = None
                    file_size = 0
                    
        batch.append((
            row['id'], row['vendor_id'], row['product_id'], new_image_url, row['file_path'],
            row['is_cover'], row['sort_order'], row['caption'], row['created_at'],
            row['media_type'], new_file_url, new_thumbnail_url, file_size, duration
        ))
        
        if len(batch) >= BATCH_SIZE:
            if not is_dry_run:
                execute_values(cur_new, insert_query, batch)
            total_migrated += len(batch)
            batch = []
            
    if batch:
        if not is_dry_run:
            execute_values(cur_new, insert_query, batch)
        total_migrated += len(batch)
        
    logging.info(f"Migrated {total_migrated} vendor_gallery rows. {not_found} files not found locally.")

def migrate_standalone_uploads(conn_new, minio_client, file_map, is_dry_run):
    logging.info("Processing any standalone files in uploads folder...")
    processed_count = 0
    for basename, local_path in file_map.items():
        hotel_name = os.path.basename(os.path.dirname(local_path))
        file_ext = os.path.splitext(local_path)[1].lower()
        file_id = str(uuid.uuid4())
        
        try:
            if file_ext in ['.jpg', '.jpeg', '.png', '.webp']:
                img_bytes, img_size, thumb_bytes, thumb_size = process_image(local_path)
                if img_bytes:
                    minio_filename = f"ZWI_{file_id}.webp"
                    minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                    if not is_dry_run:
                        minio_client.put_object(
                            MINIO_BUCKET, minio_path, io.BytesIO(img_bytes), img_size, content_type="image/webp"
                        )
                    processed_count += 1
            elif file_ext in ['.mp4', '.mov', '.avi']:
                vid_bytes, vid_size = process_video(local_path)
                if vid_bytes:
                    minio_filename = f"ZWV_{file_id}{file_ext}"
                    minio_path = f"assets/uploads/vendors/{hotel_name}/{minio_filename}"
                    if not is_dry_run:
                        minio_client.put_object(
                            MINIO_BUCKET, minio_path, io.BytesIO(vid_bytes), vid_size, content_type="video/mp4"
                        )
                    processed_count += 1
        except Exception as e:
            error_logger.error(f"Failed standalone processing for {local_path}: {str(e)}")
            
    logging.info(f"Processed {processed_count} standalone files to MinIO (No DB linking possible without IDs).")

def get_target_columns(conn, table_name):
    cur = conn.cursor()
    cur.execute("SELECT column_name FROM information_schema.columns WHERE table_name = %s", (table_name,))
    return [row[0] for row in cur.fetchall()]

def migrate_table(conn_legacy, conn_new, table_name):
    logging.info(f"Migrating {table_name}...")
    target_cols = get_target_columns(conn_new, table_name)
    if not target_cols:
        logging.error(f"Table {table_name} not found in target DB")
        return

    cur_legacy = conn_legacy.cursor(cursor_factory=DictCursor)
    cur_new = conn_new.cursor()
    
    cur_legacy.execute(f"SELECT * FROM {table_name}")
    rows = cur_legacy.fetchall()
    if not rows:
        return
        
    cols = [col for col in list(rows[0].keys()) if col in target_cols]
    col_str = ', '.join(cols)
    val_str = ', '.join(['%s'] * len(cols))
    
    insert_query = f"INSERT INTO {table_name} ({col_str}) VALUES ({val_str}) ON CONFLICT DO NOTHING"
    
    batch = []
    total_migrated = 0
    for row in rows:
        def adapt(col_name, val):
            if isinstance(val, dict):
                return psycopg2.extras.Json(val)
            if col_name == 'features' and isinstance(val, list):
                return psycopg2.extras.Json(val)
            return val
        batch.append(tuple(adapt(col, row[col]) for col in cols))
        if len(batch) >= BATCH_SIZE:
            try:
                execute_values(cur_new, insert_query.replace(f"({val_str})", "%s"), batch)
            except psycopg2.Error as e:
                logging.error(f"Error migrating batch in {table_name}: {e}")
                conn_new.rollback()
                return
            total_migrated += len(batch)
            batch = []
            
    if batch:
        try:
            execute_values(cur_new, insert_query.replace(f"({val_str})", "%s"), batch)
        except psycopg2.Error as e:
            logging.error(f"Error migrating batch in {table_name}: {e}")
            conn_new.rollback()
            return
        total_migrated += len(batch)
        
    logging.info(f"Migrated {total_migrated} rows for {table_name}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--dry-run', action='store_true', help='Run without committing to DB and without MinIO uploads')
    parser.add_argument('--execute', action='store_true', help='Execute the migration')
    args = parser.parse_args()
    
    if not args.dry_run and not args.execute:
        print("Please specify --dry-run or --execute")
        sys.exit(1)
        
    is_dry_run = args.dry_run
    
    logging.info(f"Starting Migration Script (Dry Run: {is_dry_run})")
    
    try:
        conn_legacy = psycopg2.connect(DB_LEGACY_DSN)
        conn_new = psycopg2.connect(DB_NEW_DSN)
        
        conn_new.autocommit = False
        
        minio_client = get_minio_client()
        if not is_dry_run:
            ensure_bucket_exists(minio_client)
            
    except Exception as e:
        error_logger.error(f"Connection setup failed: {str(e)}")
        sys.exit(1)
        
    try:
        file_map = build_file_map(UPLOADS_DIR)
        logging.info(f"Discovered {len(file_map)} files in local uploads directory.")
        
        for table in ['countries', 'cities', 'categories', 'global_users', 'subscription_tiers', 'vendors']:
            try:
                migrate_table(conn_legacy, conn_new, table)
                if not is_dry_run:
                    conn_new.commit()
            except Exception as e:
                logging.error(f"Error migrating {table}: {e}")
                conn_new.rollback()
                
        migrate_vendor_products(conn_legacy, conn_new, is_dry_run)
        
        migrate_vendor_gallery(conn_legacy, conn_new, minio_client, file_map, is_dry_run)
        
        migrate_standalone_uploads(conn_new, minio_client, file_map, is_dry_run)
        
        if is_dry_run:
            logging.info("Dry-Run completed. Rolling back transaction.")
            conn_new.rollback()
        else:
            logging.info("Execution completed. Committing transaction.")
            conn_new.commit()
            
    except Exception as e:
        logging.error("Migration failed. Rolling back transaction.")
        error_logger.error(f"Migration fatal error: {str(e)}")
        conn_new.rollback()
        sys.exit(1)
    finally:
        conn_legacy.close()
        conn_new.close()

if __name__ == "__main__":
    main()
