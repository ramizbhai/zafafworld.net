import os
import sys
import uuid
import logging
import argparse
import psycopg2
from psycopg2.extras import DictCursor
from minio import Minio
import re

# Setup Logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"
MINIO_ENDPOINT = '127.0.0.1:9000'
MINIO_ACCESS_KEY = 'zafaf_minio_admin'
MINIO_SECRET_KEY = 'zafaf_minio_secret'
MINIO_BUCKET = 'zafafworld-media'

# Import OVERRIDES from generate_audit_report
sys_path = '/home/noon/.gemini/antigravity-cli/brain/96361d8f-cb76-4d56-9b17-452bd178ac00/scratch'
sys.path.append(sys_path)
from generate_audit_report import OVERRIDES

TRANSLATIONS = {
    'reef': 'الريف', 'aber': 'عابر', 'anoud': 'العنود', 'ared': 'العارض', 'aseela': 'الاصيلة',
    'al-aseela': 'الاصيلة', 'darah': 'الدارة', 'al-darah': 'الدارة', 'fakhamah': 'الفخامه',
    'al-fakhamah': 'الفخامه', 'fareeda': 'الفريدة', 'al-fareeda': 'الفريدة', 'ferdous': 'الفردوس',
    'jawsak': 'الجوسق', 'mamlaka': 'المملكة', 'al-mamlaka': 'المملكة', 'multaqal': 'الملتقى',
    'al-multaqal': 'الملتقى', 'qasr': 'القصر', 'al-qasr': 'القصر', 'riyada': 'الريادة',
    'thuraya': 'الثريا', 'al-thuraya': 'الثريا', 'manara': 'المناره', 'almanara': 'المناره',
    'astura': 'الاسطوره', 'alastura': 'الاسطوره', 'areen': 'أرين', 'aroma': 'أروما',
    'art view': 'ارت فيو', 'asdaa': 'اصداء', 'raha': 'الراحة', 'rozana': 'الروزانا',
    'awfad': 'أوفاد', 'azizia': 'العزيزيه', 'braira': 'بريرا', 'nakheel': 'النخيل',
    'olaya': 'العليا', 'qurtubah': 'قرطبة', 'casa di ora': 'كازا دي اورا', 'centro': 'سنترو',
    'chalet': 'شالي', 'residence': 'ريزيدنس', 'khobar': 'الخبر', 'courtyard': 'كورتيارد',
    'marriott': 'ماريوت', 'diplomatic': 'الدبلوماسي', 'quarter': 'الحي', 'crown': 'تاج',
    'night': 'ليلة', 'radisson': 'راديسون', 'blu': 'بلو', 'salam': 'السلام', 'plaza': 'بلازا',
    'corniche': 'الكورنيش', 'park inn': 'بارك ان', 'madinah': 'المدينة', 'road': 'طريق',
    'faisaliah': 'الفيصلية', 'sheraton': 'شيراتون', 'ewa': 'ايوا', 'express': 'اكسبريس',
    'airport': 'المطار', 'monsiah': 'المونسية', 'grand': 'جراند', 'ship': 'السفينة',
    'golden': 'الذهبية', 'narcissus': 'نارسيس', 'gulf': 'الخليج', 'delmon': 'دلمون',
    'delmon hall': 'دلمون', 'warwick': 'وارويك', 'holiday inn': 'هوليدي ان', 'gateway': 'بوابة',
    'movenpick': 'موفنبيك', 'city star': 'سيتي ستار', 'crowne plaza': 'كراون بلازا', 'nora': 'نورا',
    'ramada': 'رمادا', 'hyatt': 'حياة', 'place': 'بلايس', 'sulaymaniyah': 'السليمانية',
    'white diamond': 'وايت دايموند', 'diamond': 'دايموند', 'palace': 'قصر', 'arab': 'العرب',
    'kharj': 'الخرج', 'shatel': 'شتيل', 'khayal': 'خيال', 'milano': 'ميلانو', 'sands': 'الرمال',
    'danat': 'دانات', 'gemini': 'جميني', 'afif': 'عفيف', 'palazzo': 'بلاتسو', 'sultan': 'سلطان',
    'kamarah': 'قمره', 'lafara': 'لفارا', 'resort': 'منتجع', 'dream': 'الحلم', 'white': 'الابيض',
    'jeddah': 'جدة', 'riyadh': 'الرياض', 'dammam': 'الدمام', 'hotel': 'فندق', 'hall': 'قاعة',
    'ballroom': 'قاعة', 'celebration': 'احتفالات', 'celebrations': 'الاحتفالات', 'conference': 'مؤتمرات',
    'conferences': 'المؤتمرات', 'wedding': 'افراح', 'weddings': 'الافراح', 'gala': 'قاعات',
    'events': 'مناسبات',
    
    'ritz': 'ريتز', 'carlton': 'كارلتون', 'intercontinental': 'انتركونتيننتال',
    'durrah': 'درة', 'durre': 'درة', 'durra': 'درة', 'dorra': 'درة', 'dorrah': 'درة',
    'bayat': 'بيوت', 'rafal': 'رفال', 'jouri': 'الجوري', 'juri': 'الجوري',
    'mora': 'مورا', 'mirtil': 'ميرتل', 'myrtle': 'ميرتل', 'sweet': 'سويت', 'inn': 'ان',
    'seasons': 'سيزونز', 'park': 'بارك', 'frontel': 'فرنتيل', 'tahliya': 'التحلية',
    'voco': 'فوكو', 'vittori': 'فيتوري', 'miltanya': 'ميلينيا', 'millenia': 'ميلينيا',
    'iridium': 'إريديوم', 'sabeen': 'السبعين', 'elias': 'إلياس', 'tanfeez': 'التنفيذيين',
    'executive': 'التنفيذيين', 'executives': 'التنفيذيين', 'durrat': 'درة',
    'jawharat': 'جوهرة', 'jawhara': 'جوهرة', 'shally': 'شاليه', 'shali': 'شاليه',
    'al-aseela': 'الاصيلة', 'aseelah': 'الاصيلة', 'al-darah': 'الدارة', 'darah': 'الدارة',
    'miral': 'ميرال', 'meyan': 'ميون', 'hadab': 'هدب', 'sahafa': 'الصحافة', 'sahafah': 'الصحافة',
    'nader': 'نادر', 'ader': 'نادر'
}

def is_english_only(text):
    if not text:
        return False
    arabic_pattern = re.compile(r'[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]')
    return not bool(arabic_pattern.search(text))

def translate_to_arabic(text):
    if not text or not is_english_only(text):
        return text
        
    text = text.lower().replace('&', 'and')
    text = re.sub(r'\s+', ' ', text)
    
    is_hotel = 'hotel' in text or 'resort' in text
    is_hall = 'hall' in text or 'ballroom' in text or 'gala' in text
    is_palace = 'palace' in text and 'hotel' not in text
    is_chalet = 'chalet' in text or 'shaly' in text or 'shally' in text
    
    text = text.replace('hotel', '').replace('resort', '').replace('hall', '').replace('ballroom', '').replace('gala', '').replace('palace', '').replace('chalet', '').replace('shally', '')
    
    words = re.findall(r'\b\w+\b', text)
    translated_words = []
    
    for w in words:
        if w in ['for', 'and', 'with', 'by', 'of', 'in', 'at']:
            if w == 'and':
                translated_words.append('و')
            continue
            
        if w in TRANSLATIONS:
            translated_words.append(TRANSLATIONS[w])
        else:
            translated_words.append(w.capitalize())
            
    body = ' '.join(translated_words)
    body = re.sub(r'\s+و\s+', ' و', body)
    
    if is_hotel:
        body = 'فندق ' + body
    elif is_hall:
        body = 'قاعة ' + body
    elif is_palace:
        body = 'قصر ' + body
    elif is_chalet:
        body = 'شاليه ' + body
        
    return body.strip()

def repair_database(execute=False):
    logging.info(f"Connecting to database (execute={execute})...")
    conn = psycopg2.connect(DB_DSN)
    conn.autocommit = False
    cur = conn.cursor(cursor_factory=DictCursor)
    
    # --- PART 1: ASSET RECOVERY ---
    logging.info("Starting Asset Recovery re-mapping...")
    
    # Get all vendors
    cur.execute("SELECT id, name_en FROM vendors")
    vendors_by_name = {v['name_en'].strip().replace('\u200e', '').replace('\u200f', ''): v['id'] for v in cur.fetchall()}
    
    minio_client = Minio(MINIO_ENDPOINT, access_key=MINIO_ACCESS_KEY, secret_key=MINIO_SECRET_KEY, secure=False)
    objects = list(minio_client.list_objects(MINIO_BUCKET, prefix="assets/uploads/vendors/", recursive=True))
    
    # Query already mapped object keys to avoid UniqueViolation
    cur.execute("SELECT object_key FROM uploaded_files")
    used_object_keys = set(row['object_key'] for row in cur.fetchall())
    
    minio_folders = {}
    for obj in objects:
        parts = obj.object_name.split('/')
        if len(parts) >= 5:
            folder = parts[3]
            file_name = parts[-1]
            if folder not in minio_folders:
                minio_folders[folder] = []
            minio_folders[folder].append((obj.object_name, obj.size, file_name))
            
    total_gallery_updated = 0
    total_uploaded_files_inserted = 0
    total_unmapped_left_for_overrides = 0
    
    # Group override folders by vendor id
    vendor_to_folders = {}
    for f_folder, v_name in OVERRIDES.items():
        norm_v = v_name.strip().replace('\u200e', '').replace('\u200f', '')
        if norm_v not in vendors_by_name:
            logging.warning(f"Override vendor '{v_name}' not found in database.")
            continue
        v_id = vendors_by_name[norm_v]
        if v_id not in vendor_to_folders:
            vendor_to_folders[v_id] = []
        vendor_to_folders[v_id].append(f_folder)
        
    for vendor_id, folders in vendor_to_folders.items():
        # Get unmapped vendor_gallery rows for this vendor
        cur.execute("""
            SELECT id, is_cover, media_type 
            FROM vendor_gallery 
            WHERE vendor_id = %s AND file_id IS NULL 
            ORDER BY is_cover DESC, created_at ASC
        """, (vendor_id,))
        gallery_rows = cur.fetchall()
        
        if not gallery_rows:
            continue
            
        # Get and combine MinIO files for all folders mapped to this vendor
        files = []
        for f_folder in folders:
            files.extend(minio_folders.get(f_folder, []))
            
        # Sort files alphabetically to ensure determinism
        files.sort(key=lambda x: x[2])
        
        main_files = [f for f in files if '_thumb' not in f[2] and 'ZWI_' in f[2] and f[0] not in used_object_keys]
        video_files = [f for f in files if 'ZWV_' in f[2] and f[0] not in used_object_keys]
        
        img_idx = 0
        vid_idx = 0
        
        for row in gallery_rows:
            row_id = row['id']
            media_type = row['media_type']
            
            assigned_file = None
            if media_type == 'image' and img_idx < len(main_files):
                assigned_file = main_files[img_idx]
                img_idx += 1
            elif media_type == 'video' and vid_idx < len(video_files):
                assigned_file = video_files[vid_idx]
                vid_idx += 1
                
            if assigned_file:
                minio_path = assigned_file[0]
                file_size = assigned_file[1]
                basename = assigned_file[2]
                file_url = f"/{minio_path}"
                mime_type = 'image/webp' if media_type == 'image' else 'video/mp4'
                
                # Insert metadata into uploaded_files
                cur.execute("""
                    INSERT INTO uploaded_files (id, bucket_name, object_key, file_name, file_size, mime_type, status)
                    VALUES (%s, %s, %s, %s, %s, %s, 'ready')
                    ON CONFLICT (id) DO UPDATE SET 
                        bucket_name = EXCLUDED.bucket_name, 
                        object_key = EXCLUDED.object_key, 
                        file_name = EXCLUDED.file_name, 
                        file_size = EXCLUDED.file_size, 
                        mime_type = EXCLUDED.mime_type, 
                        status = EXCLUDED.status
                """, (row_id, MINIO_BUCKET, minio_path, basename, file_size, mime_type))
                total_uploaded_files_inserted += 1
                
                # Update vendor_gallery link
                cur.execute("""
                    UPDATE vendor_gallery 
                    SET image_url = %s, file_url = %s, thumbnail_url = NULL, file_id = %s
                    WHERE id = %s
                """, (file_url, file_url, row_id, row_id))
                total_gallery_updated += 1
            else:
                total_unmapped_left_for_overrides += 1
                
    logging.info(f"Asset Recovery completed. Mapped {total_gallery_updated} rows. Rows left unmapped: {total_unmapped_left_for_overrides}.")
    
    # --- PART 2: LOCALIZATION REPAIR ---
    logging.info("Starting Localization Repair...")
    
    TABLES_LOCALIZATION = {
        'vendors': [
            ('name_ar', 'name_en'),
            ('description_ar', 'description_en'),
            ('address_ar', 'address_en'),
            ('coordinator_name_ar', 'coordinator_name_en')
        ],
        'vendor_products': [
            ('title_ar', 'title_en'),
            ('description_ar', 'description_en'),
            ('coordinator_name_ar', 'coordinator_name_en'),
            ('meta_title_ar', 'meta_title_en'),
            ('meta_description_ar', 'meta_description_en')
        ],
        'categories': [
            ('description_ar', 'description_en')
        ]
    }
    
    total_loc_updated = 0
    
    for table, pairs in TABLES_LOCALIZATION.items():
        pk = 'slug' if table == 'categories' else 'id'
        
        for col_ar, col_en in pairs:
            # For coordinator_name in vendor_products, select it too
            if table == 'vendor_products' and col_en == 'coordinator_name_en':
                cur.execute(f"SELECT {pk}, {col_ar}, {col_en}, coordinator_name FROM {table}")
            else:
                cur.execute(f"SELECT {pk}, {col_ar}, {col_en} FROM {table}")
                
            rows = cur.fetchall()
            
            for row in rows:
                row_id = row[pk]
                val_ar = row[col_ar]
                val_en = row[col_en]
                
                new_ar = val_ar
                new_en = val_en
                
                # Check for desync and fix
                is_desynced = False
                if val_ar is None or str(val_ar).strip() == '' or is_english_only(str(val_ar)):
                    is_desynced = True
                    
                if is_desynced:
                    # 1. Special case: coordinator_name_ar/coordinator_name_en in vendor_products
                    if table == 'vendor_products' and col_en == 'coordinator_name_en':
                        coord_val = row['coordinator_name']
                        if (not val_en or str(val_en).strip() == '') and coord_val and str(coord_val).strip() != '':
                            new_en = str(coord_val).strip()
                            new_ar = translate_to_arabic(new_en)
                            
                    # 2. General case: val_ar has English, val_en is empty or same
                    elif val_ar and is_english_only(str(val_ar)) and (not val_en or str(val_en).strip() == '' or str(val_en).strip() == str(val_ar).strip()):
                        new_en = str(val_ar).strip()
                        new_ar = translate_to_arabic(new_en)
                        
                    # 3. General case: val_ar is empty, val_en is populated
                    elif (not val_ar or str(val_ar).strip() == '') and val_en and str(val_en).strip() != '':
                        new_ar = translate_to_arabic(str(val_en).strip())
                        
                    # 4. General case: val_ar has English, val_en is populated
                    elif val_ar and is_english_only(str(val_ar)) and val_en and str(val_en).strip() != '':
                        new_ar = translate_to_arabic(str(val_en).strip())
                        
                # Update if changed
                if new_ar != val_ar or new_en != val_en:
                    cur.execute(f"""
                        UPDATE {table} 
                        SET {col_ar} = %s, {col_en} = %s 
                        WHERE {pk} = %s
                    """, (new_ar, new_en, row_id))
                    total_loc_updated += 1
                    
    logging.info(f"Localization Repair completed. Repaired {total_loc_updated} fields.")
    
    # --- PART 3: INTEGRITY CHECKS ---
    logging.info("Performing final integrity checks...")
    
    # Check 1: Count unmapped gallery rows for overrides vendors
    unmapped_left = 0
    for vendor_id in vendor_to_folders.keys():
        cur.execute("SELECT COUNT(*) FROM vendor_gallery WHERE vendor_id = %s AND file_id IS NULL", (vendor_id,))
        unmapped_left += cur.fetchone()[0]
            
    # Check 2: Count remaining desynced fields in the repaired columns
    desync_left = 0
    for table, pairs in TABLES_LOCALIZATION.items():
        pk = 'slug' if table == 'categories' else 'id'
        for col_ar, col_en in pairs:
            # We don't check description/address in vendors since both are NULL and cannot be repaired.
            # We also don't check description in categories since they are NULL.
            # We also don't check meta_title/meta_description in vendor_products since they are NULL.
            if table == 'vendors' and col_ar in ('description_ar', 'address_ar'):
                continue
            if table == 'categories' and col_ar == 'description_ar':
                continue
            if table == 'vendor_products' and col_ar in ('meta_title_ar', 'meta_description_ar'):
                continue
                
            cur.execute(f"SELECT {col_ar}, {col_en} FROM {table}")
            rows = cur.fetchall()
            for row in rows:
                val_ar = row[col_ar]
                val_en = row[col_en]
                # If coordinator name remains empty, it might be that both coordinator_name and coordinator_name_en were empty/NULL.
                # If so, we skip counting it if both are empty/NULL.
                if table == 'vendor_products' and col_ar == 'coordinator_name_ar' and not val_ar and not val_en:
                    continue
                if val_ar is None or str(val_ar).strip() == '' or is_english_only(str(val_ar)):
                    desync_left += 1
                    
    logging.info(f"Integrity check results:")
    logging.info(f"  - Unmapped vendor_gallery rows left for OVERRIDES vendors: {unmapped_left}")
    logging.info(f"  - Desynced fields left in repaired columns: {desync_left}")
    
    # Verification condition
    if unmapped_left == total_unmapped_left_for_overrides:
        logging.info("Asset Recovery integrity check PASSED.")
    else:
        logging.error("Asset Recovery integrity check FAILED!")
        conn.rollback()
        sys.exit(1)
        
    if execute:
        logging.info("Committing database transactions...")
        conn.commit()
        logging.info("Database updates committed successfully!")
    else:
        logging.info("Dry-run mode. Rolling back database transaction.")
        conn.rollback()
        logging.info("Database rolled back successfully.")
        
    conn.close()
    return total_gallery_updated, total_uploaded_files_inserted, total_loc_updated, unmapped_left, desync_left

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true', help='Execute and commit the repairs to the database')
    args = parser.parse_args()
    
    repair_database(execute=args.execute)
