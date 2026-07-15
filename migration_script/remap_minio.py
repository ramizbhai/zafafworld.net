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

TRANSLATIONS = {
    'reef': 'الريف',
    'aber': 'عابر',
    'anoud': 'العنود',
    'ared': 'العارض',
    'aseela': 'الاصيلة',
    'al-aseela': 'الاصيلة',
    'darah': 'الدارة',
    'al-darah': 'الدارة',
    'fakhamah': 'الفخامه',
    'al-fakhamah': 'الفخامه',
    'fareeda': 'الفريدة',
    'al-fareeda': 'الفريدة',
    'ferdous': 'الفردوس',
    'jawsak': 'الجوسق',
    'mamlaka': 'المملكة',
    'al-mamlaka': 'المملكة',
    'multaqal': 'الملتقى',
    'al-multaqal': 'الملتقى',
    'qasr': 'القصر',
    'al-qasr': 'القصر',
    'riyada': 'الريادة',
    'thuraya': 'الثريا',
    'al-thuraya': 'الثريا',
    'manara': 'المناره',
    'almanara': 'المناره',
    'astura': 'الاسطوره',
    'alastura': 'الاسطوره',
    'areen': 'أرين',
    'aroma': 'أروما',
    'art view': 'ارت فيو',
    'asdaa': 'اصداء',
    'raha': 'الراحة',
    'rozana': 'الروزانا',
    'awfad': 'أوفاد',
    'azizia': 'العزيزيه',
    'braira': 'بريرا',
    'nakheel': 'النخيل',
    'olaya': 'العليا',
    'qurtubah': 'قرطبة',
    'casa di ora': 'كازا دي اورا',
    'centro': 'سنترو',
    'chalet': 'شالي',
    'residence': 'ريزيدنس',
    'khobar': 'الخبر',
    'courtyard': 'كورتيارد',
    'marriott': 'ماريوت',
    'diplomatic': 'الدبلوماسي',
    'quarter': 'الحي',
    'crown': 'تاج',
    'night': 'ليلة',
    'radisson': 'راديسون',
    'blu': 'بلو',
    'salam': 'السلام',
    'plaza': 'بلازا',
    'corniche': 'الكورنيش',
    'park inn': 'بارك ان',
    'madinah': 'المدينة',
    'road': 'طريق',
    'faisaliah': 'الفيصلية',
    'sheraton': 'شيراتون',
    'ewa': 'ايوا',
    'express': 'اكسبريس',
    'airport': 'المطار',
    'monsiah': 'المونسية',
    'grand': 'جراند',
    'ship': 'السفينة',
    'golden': 'الذهبية',
    'narcissus': 'نارسيس',
    'gulf': 'الخليج',
    'delmon': 'دلمون',
    'delmon hall': 'دلمون',
    'warwick': 'وارويك',
    'holiday inn': 'هوليدي ان',
    'gateway': 'بوابة',
    'movenpick': 'موفنبيك',
    'city star': 'سيتي ستار',
    'crowne plaza': 'كراون بلازا',
    'nora': 'نورا',
    'ramada': 'رمادا',
    'hyatt': 'حياة',
    'place': 'بلايس',
    'sulaymaniyah': 'السليمانية',
    'white diamond': 'وايت دايموند',
    'diamond': 'دايموند',
    'palace': 'قصر',
    'arab': 'العرب',
    'kharj': 'الخرج',
    'shatel': 'شتيل',
    'khayal': 'خيال',
    'milano': 'ميلانو',
    'sands': 'الرمال',
    'danat': 'دانات',
    'gemini': 'جميني',
    'afif': 'عفيف',
    'palazzo': 'بلاتسو',
    'sultan': 'سلطان',
    'kamarah': 'قمره',
    'lafara': 'لفارا',
    'resort': 'منتجع',
    'dream': 'الحلم',
    'white': 'الابيض',
    'jeddah': 'جدة',
    'riyadh': 'الرياض',
    'dammam': 'الدمام',
    'hotel': 'فندق',
    'hall': 'قاعة',
    'ballroom': 'قاعة',
    'celebration': 'احتفالات',
    'celebrations': 'الاحتفالات',
    'conference': 'مؤتمرات',
    'conferences': 'المؤتمرات',
    'wedding': 'افراح',
    'weddings': 'الافراح',
    'gala': 'قاعات',
    'events': 'مناسبات',
}

STRIP_WORDS = {
    'hotel', 'hall', 'ballroom', 'celebration', 'celebrations', 'conference', 'conferences',
    'wedding', 'weddings', 'gala', 'events', 'resort', 'palace', 'center', 'express', 'suites',
    'for', 'and', 'with', 'by', 'of', 'in', 'at',
    'فندق', 'قاعة', 'الاحتفالات', 'المؤتمرات', 'الافراح', 'الاحتفال', 'احتفالات', 'مؤتمرات',
    'افراح', 'مناسبات', 'جناح', 'أجنحة', 'أبحر', 'طريق', 'المدينة', 'قاعات', 'منتجع', 'قصر',
    'مركز', 'الرياض', 'جدة', 'الدمام', 'الخبر', 'في', 'من', 'و', 'مع', 'للاحتفالات', 'للمؤتمرات',
    'للأفراح', 'والمناسبات', 'بوابة', 'شبه', 'سابقا', 'ملفات', 'files', 'الأمواج', 'الحمراء',
    'الذهبية', 'الشرقي'
}

def get_minio_client():
    return Minio(MINIO_ENDPOINT, access_key=MINIO_ACCESS_KEY, secret_key=MINIO_SECRET_KEY, secure=False)

def clean_and_translate(text):
    text = text.lower()
    for char in [',', '-', '–', '.', '(', ')', '_', '/', ':', '‎']:
        text = text.replace(char, ' ')
    
    words = text.split()
    translated_words = []
    for w in words:
        if w in STRIP_WORDS:
            continue
        if w in TRANSLATIONS:
            translated_words.append(TRANSLATIONS[w])
        else:
            translated_words.append(w)
            
    filtered = []
    for w in translated_words:
        if w in STRIP_WORDS or not w.strip():
            continue
        filtered.append(w)
    return ' '.join(filtered)

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
    
    # 3. Match vendors to MinIO folders using our clean matching logic
    matches = []
    for v in vendors:
        name_en = v['name_en'] or ""
        name_ar = v['name_ar'] or ""
        
        translated_en = clean_and_translate(name_en)
        translated_ar = clean_and_translate(name_ar)
        
        for f in minio_folder_names:
            f_clean = clean_and_translate(f)
            
            if not translated_en and not translated_ar:
                continue
            if not f_clean:
                continue
                
            score_en = difflib.SequenceMatcher(None, translated_en, f_clean).ratio()
            score_ar = difflib.SequenceMatcher(None, translated_ar, f_clean).ratio()
            
            words_v = set(translated_en.split() + translated_ar.split())
            words_f = set(f_clean.split())
            overlap = words_v.intersection(words_f)
            overlap_score = len(overlap) / max(len(words_v), len(words_f), 1)
            
            sub_bonus = 0
            if f_clean in translated_en or f_clean in translated_ar:
                sub_bonus = 0.3
            for w in words_f:
                if len(w) > 3 and (w in translated_en or w in translated_ar):
                    sub_bonus = max(sub_bonus, 0.2)
                    
            score = max(score_en, score_ar) * 0.5 + overlap_score * 0.5 + sub_bonus
            
            if score > 0.70:
                matches.append((v, f, score))
                
    # Sort matches descending to assign the highest quality mappings first (1-to-1)
    matches.sort(key=lambda x: x[2], reverse=True)
    
    final_mappings = {}
    used_folders = set()
    for v, f, score in matches:
        if v['id'] not in final_mappings and f not in used_folders:
            final_mappings[v['id']] = (v, f)
            used_folders.add(f)
            
    logging.info(f"Established {len(final_mappings)} unique vendor-to-folder mappings.")
    
    updated_rows = 0
    
    # 4. Map the files for each successfully matched vendor
    for vendor_id, (vendor, best_match) in final_mappings.items():
        name_en = vendor['name_en']
        logging.info(f"Processing vendor '{name_en}' mapping to folder '{best_match}'")
        
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
                    # Insert into uploaded_files
                    cur.execute("""
                        INSERT INTO uploaded_files (id, bucket_name, object_key, file_name, file_size, mime_type, status)
                        VALUES (%s, %s, %s, %s, %s, %s, 'ready')
                        ON CONFLICT (object_key) DO NOTHING
                    """, (row_id, MINIO_BUCKET, minio_path, basename, file_size, mime_type))
                    
                    # Update vendor_gallery including file_id link
                    cur.execute("""
                        UPDATE vendor_gallery 
                        SET image_url = %s, file_url = %s, thumbnail_url = %s, file_id = %s
                        WHERE id = %s
                    """, (file_url, file_url, thumb_url, row_id, row_id))
                    
                    updated_rows += 1
                except Exception as e:
                    logging.error(f"Error updating DB for row {row_id}: {e}")
                    conn.rollback()
                    raise
                    
    conn.commit()
    logging.info(f"Finished mapping. Updated {updated_rows} vendor_gallery rows.")
    conn.close()

if __name__ == '__main__':
    remap_minio_files()
