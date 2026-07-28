import os
import sys
import hashlib
import logging
from minio import Minio
from minio.error import S3Error
import psycopg2
from psycopg2.extras import DictCursor
from dotenv import load_dotenv

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout)
    ]
)

# Load environment
dotenv_path = '/opt/zafafworld.net/.env'
load_dotenv(dotenv_path)

# MinIO configs
MINIO_ENDPOINT = os.getenv('MINIO_ENDPOINT', 'http://minio:9000')
# Translate container endpoint to host endpoint
MINIO_ENDPOINT_HOST = MINIO_ENDPOINT.replace('http://minio:9000', '127.0.0.1:9000').replace('http://', '').replace('https://', '')

MINIO_ACCESS_KEY = os.getenv('MINIO_APP_USER', 'zafaf_minio_admin')
MINIO_SECRET_KEY = os.getenv('MINIO_APP_PASSWORD', 'zafaf_minio_secret')
MINIO_BUCKET = os.getenv('MINIO_BUCKET', 'zafafworld-media')

# Postgres configs
DATABASE_URL = os.getenv('DATABASE_URL', 'postgres://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world')
# Translate container endpoint/port to host endpoint
DB_URL_HOST = DATABASE_URL.replace('@postgres:5432', '@127.0.0.1:5434').replace('@pgbouncer:5432', '@127.0.0.1:5434')

LOCAL_UPLOADS_DIR = "/var/lib/zafafworld/uploads"

def get_md5(file_path):
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def get_mime_type(file_name):
    ext = os.path.splitext(file_name)[1].lower()
    if ext in ['.jpg', '.jpeg']:
        return 'image/jpeg'
    elif ext == '.png':
        return 'image/png'
    elif ext == '.webp':
        return 'image/webp'
    elif ext == '.mp4':
        return 'video/mp4'
    elif ext == '.webm':
        return 'video/webm'
    elif ext == '.mov':
        return 'video/quicktime'
    else:
        return 'application/octet-stream'

def migrate():
    logging.info("Starting MinIO Media Migration...")
    
    # 1. Connect to MinIO
    logging.info(f"Connecting to MinIO at {MINIO_ENDPOINT_HOST}...")
    try:
        minio_client = Minio(
            MINIO_ENDPOINT_HOST,
            access_key=MINIO_ACCESS_KEY,
            secret_key=MINIO_SECRET_KEY,
            secure=False
        )
        if not minio_client.bucket_exists(MINIO_BUCKET):
            logging.info(f"Creating bucket {MINIO_BUCKET}...")
            minio_client.make_bucket(MINIO_BUCKET)
    except Exception as e:
        logging.error(f"Failed to connect to MinIO: {e}")
        sys.exit(1)

    # 2. Connect to Postgres
    logging.info("Connecting to Postgres...")
    try:
        conn = psycopg2.connect(DB_URL_HOST)
        cur = conn.cursor(cursor_factory=DictCursor)
    except Exception as e:
        logging.error(f"Failed to connect to Postgres: {e}")
        sys.exit(1)

    if not os.path.exists(LOCAL_UPLOADS_DIR):
        logging.error(f"Local uploads directory does not exist at: {LOCAL_UPLOADS_DIR}")
        sys.exit(1)

    files_to_migrate = []
    for root, dirs, files in os.walk(LOCAL_UPLOADS_DIR):
        for f in files:
            # Skip temp staging files that are actively being processed (ending in .tmp or .tmp.mp4)
            if f.endswith('.tmp') or f.endswith('.tmp.mp4') or f.startswith('.'):
                continue
            full_path = os.path.join(root, f)
            rel_path = os.path.relpath(full_path, LOCAL_UPLOADS_DIR)
            files_to_migrate.append((full_path, rel_path, f))

    logging.info(f"Found {len(files_to_migrate)} files to migrate.")

    success_count = 0
    verification_failed = 0

    for full_path, rel_path, filename in files_to_migrate:
        logging.info(f"Processing: {rel_path}")
        local_size = os.path.getsize(full_path)
        local_md5 = get_md5(full_path)
        mime_type = get_mime_type(filename)
        
        # We check if it exists in MinIO
        exists = False
        try:
            stat = minio_client.stat_object(MINIO_BUCKET, rel_path)
            exists = True
            minio_size = stat.size
            logging.info(f"  Already exists in MinIO. Size={minio_size} bytes (Local={local_size} bytes).")
            if minio_size != local_size:
                logging.warning(f"  Size mismatch! Re-uploading...")
                exists = False
        except S3Error as e:
            if e.code == 'NoSuchKey':
                pass
            else:
                logging.error(f"  Error checking MinIO: {e}")
                verification_failed += 1
                continue

        if not exists:
            logging.info(f"  Uploading to MinIO key='{rel_path}'...")
            try:
                minio_client.fput_object(
                    MINIO_BUCKET,
                    rel_path,
                    full_path,
                    content_type=mime_type
                )
                logging.info(f"  Upload completed successfully.")
            except Exception as e:
                logging.error(f"  Upload failed: {e}")
                verification_failed += 1
                continue

        # Double check verification
        try:
            stat = minio_client.stat_object(MINIO_BUCKET, rel_path)
            if stat.size == local_size:
                logging.info(f"  ✓ Verified in MinIO: size matches ({stat.size} bytes).")
                success_count += 1
                # Once verified, delete local copy
                try:
                    os.remove(full_path)
                    logging.info(f"  ✓ Deleted local copy: {full_path}")
                except Exception as e:
                    logging.error(f"  ✗ Failed to delete local copy {full_path}: {e}")
            else:
                logging.error(f"  ✗ Verification failed: size mismatch in MinIO ({stat.size} vs local {local_size})")
                verification_failed += 1
                continue
        except Exception as e:
            logging.error(f"  ✗ Verification check failed: {e}")
            verification_failed += 1
            continue

        # 3. Check and insert DB record in uploaded_files if missing
        try:
            cur.execute("SELECT id, status FROM uploaded_files WHERE object_key = %s", (rel_path,))
            db_row = cur.fetchone()
            if not db_row:
                logging.info(f"  DB record missing for key '{rel_path}'. Inserting...")
                import uuid
                new_id = uuid.uuid4()
                cur.execute("""
                    INSERT INTO uploaded_files (id, bucket_name, object_key, file_name, file_size, mime_type, status)
                    VALUES (%s, %s, %s, %s, %s, %s, 'ready')
                """, (str(new_id), MINIO_BUCKET, rel_path, filename, local_size, mime_type))
                conn.commit()
                logging.info(f"  ✓ DB record inserted (id: {new_id}).")
            else:
                logging.info(f"  ✓ DB record exists (id: {db_row['id']}, status: {db_row['status']}).")
        except Exception as e:
            conn.rollback()
            logging.error(f"  Database error: {e}")
            verification_failed += 1

    conn.close()
    
    logging.info("=========================================")
    logging.info(f"Migration Summary:")
    logging.info(f"  Total files scanned: {len(files_to_migrate)}")
    logging.info(f"  Successfully verified in MinIO: {success_count}")
    logging.info(f"  Failed: {verification_failed}")
    logging.info("=========================================")

    if verification_failed > 0:
        logging.error("Migration finished with errors.")
        sys.exit(1)
    else:
        logging.info("Migration completed successfully with zero errors!")
        sys.exit(0)

if __name__ == '__main__':
    migrate()
