import os
import sys
import re
import gzip
import json
import logging
import argparse
import psycopg2
from psycopg2.extras import DictCursor

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"

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
    'nader': 'نادر', 'ader': 'نادر',
    
    # Realignment updates
    'venue coordinator': 'منسق الموقع',
    'Venue coordinator': 'منسق الموقع',
    'Rabia': 'ربيعة',
    'raza': 'رضا',
    'Ramiz': 'رامز',
    'Ramiz96': 'رامز',
    'Refal Homes Suites': 'أجنحة بيوت رفال',
    'Test Upload Vendor': 'مورد تجريبي',
    'Test Vendor': 'مورد تجربة',
    'Sunday Hotel Ballroom': 'قاعة صنداي الفندقية للاحتفالات'
}

def unescape_copy_val(val):
    if val == r'\N' or val == '\\N':
        return None
    def repl(m):
        char = m.group(1)
        if char == '\\':
            return '\\'
        elif char == 'n':
            return '\n'
        elif char == 'r':
            return '\r'
        elif char == 't':
            return '\t'
        return char
    return re.sub(r'\\(.)', repl, val)

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

def extract_and_repair_intro_arabic(text):
    match = re.search(r'([a-zA-Z]{4,}.*)$', text, re.DOTALL)
    if match:
        english_section = match.group(1).strip()
        if is_english_only(english_section):
            new_ar = translate_to_arabic(english_section)
            return new_ar, english_section
    return text, None

def fix_truncated_json(val):
    if not val or not val.startswith('['):
        return val
    try:
        json.loads(val)
        return val
    except json.JSONDecodeError:
        idx = val.rfind('"content":"')
        if idx != -1:
            content_start = idx + len('"content":"')
            content_part = val[content_start:]
            content_part = content_part.rstrip('\\')
            repaired = val[:content_start] + content_part + '..."}]'
            try:
                json.loads(repaired)
                return repaired
            except:
                pass
    return val

def translate_json_description(val_ar, val_en):
    # Fix truncated JSON first
    val_ar = fix_truncated_json(val_ar)
    val_en = fix_truncated_json(val_en)
    
    if not val_ar or val_ar.strip() == '' or val_ar == '\\N':
        if val_en and val_en.strip() != '' and val_en != '\\N':
            try:
                blocks = json.loads(val_en)
                for b in blocks:
                    if 'content' in b:
                        b['content'] = translate_to_arabic(b['content'])
                return json.dumps(blocks, ensure_ascii=False), val_en
            except:
                return translate_to_arabic(val_en), val_en
        return val_ar, val_en
        
    if val_ar.startswith('['):
        try:
            blocks = json.loads(val_ar)
            has_both_versions = False
            for b in blocks:
                if 'content' in b:
                    c = b['content']
                    if 'النسخة العربية' in c and 'النسخة الإنجليزية' in c:
                        has_both_versions = True
                        break
                        
            if has_both_versions:
                ar_blocks = []
                en_blocks = []
                for b in blocks:
                    if 'content' in b:
                        c = b['content']
                        ar_match = re.search(r'(?:النسخة العربية|\bArabic Version\b)(.*?)(?:النسخة الإنجليزية|\bEnglish Version\b|$)', c, re.DOTALL | re.IGNORECASE)
                        en_match = re.search(r'(?:النسخة الإنجليزية|\bEnglish Version\b)(.*?)$', c, re.DOTALL | re.IGNORECASE)
                        
                        ar_text = ar_match.group(1).strip() if ar_match else c
                        en_text = en_match.group(1).strip() if en_match else c
                        
                        ar_text = re.sub(r'^[:\-\s\n]+', '', ar_text)
                        en_text = re.sub(r'^[:\-\s\n]+', '', en_text)
                        
                        b_ar = b.copy()
                        b_ar['content'] = ar_text
                        ar_blocks.append(b_ar)
                        
                        b_en = b.copy()
                        b_en['content'] = en_text
                        en_blocks.append(b_en)
                    else:
                        ar_blocks.append(b.copy())
                        en_blocks.append(b.copy())
                return json.dumps(ar_blocks, ensure_ascii=False), json.dumps(en_blocks, ensure_ascii=False)
                
            # Check if any block has intro Arabic + English
            has_intro_arabic = False
            for b in blocks:
                if 'content' in b:
                    c = b['content']
                    if not is_english_only(c) and re.search(r'[a-zA-Z]{4,}', c):
                        has_intro_arabic = True
                        break
                        
            if has_intro_arabic:
                ar_blocks = []
                en_blocks = []
                for b in blocks:
                    if 'content' in b:
                        c = b['content']
                        new_ar_txt, new_en_txt = extract_and_repair_intro_arabic(c)
                        if new_en_txt:
                            b_ar = b.copy()
                            b_ar['content'] = new_ar_txt
                            ar_blocks.append(b_ar)
                            
                            b_en = b.copy()
                            b_en['content'] = new_en_txt
                            en_blocks.append(b_en)
                        else:
                            ar_blocks.append(b.copy())
                            b_en = b.copy()
                            b_en['content'] = translate_to_arabic(c) # fallback
                            en_blocks.append(b_en)
                    else:
                        ar_blocks.append(b.copy())
                        en_blocks.append(b.copy())
                return json.dumps(ar_blocks, ensure_ascii=False), json.dumps(en_blocks, ensure_ascii=False)
                
            is_eng = True
            for b in blocks:
                if 'content' in b and not is_english_only(b['content']):
                    is_eng = False
                    break
                    
            if is_eng:
                new_en = val_ar
                for b in blocks:
                    if 'content' in b:
                        b['content'] = translate_to_arabic(b['content'])
                new_ar = json.dumps(blocks, ensure_ascii=False)
                return new_ar, new_en
        except Exception as e:
            logging.error(f"JSON description parser failed: {e}")
            
    if is_english_only(val_ar) and (not val_en or val_en.strip() == '' or val_en == '\\N'):
        new_en = val_ar
        new_ar = translate_to_arabic(val_ar)
        return new_ar, new_en
    if (not val_ar or val_ar.strip() == '' or val_ar == '\\N') and val_en and val_en.strip() != '' and val_en != '\\N':
        new_ar = translate_to_arabic(val_en)
        return new_ar, val_en
    if is_english_only(val_ar) and val_en and val_en.strip() != '' and val_en != '\\N':
        new_ar = translate_to_arabic(val_en)
        return new_ar, val_en
        
    return val_ar, val_en

def restore_descriptions_from_backup(cur):
    logging.info("Restoring original descriptions from SQL backup...")
    backup_path = '/var/lib/zafafworld/backups/zafaf_zafaf_world_20260715_070755.sql.gz'
    copy_started = False
    restore_count = 0
    
    with gzip.open(backup_path, 'rt', encoding='utf-8') as f:
        for line in f:
            if line.startswith('COPY public.vendor_products '):
                copy_started = True
                continue
            if copy_started:
                if line.startswith(r'\.'):
                    break
                parts = line.split('\t')
                v_id = parts[0]
                desc_ar = parts[27]
                desc_en = parts[28]
                
                val_ar = unescape_copy_val(desc_ar)
                val_en = unescape_copy_val(desc_en)
                
                cur.execute("""
                    UPDATE vendor_products 
                    SET description_ar = %s, description_en = %s 
                    WHERE id = %s
                """, (val_ar, val_en, v_id))
                restore_count += 1
                
    logging.info(f"Restored original descriptions for {restore_count} vendor_products rows.")

def perform_realignment(execute=False):
    conn = psycopg2.connect(DB_DSN)
    conn.autocommit = False
    cur = conn.cursor(cursor_factory=DictCursor)
    
    try:
        # Step 1: Restore original descriptions from backup
        restore_descriptions_from_backup(cur)
        
        # Step 2: Apply high-priority localization realignment
        logging.info("Starting Localization Realignment...")
        
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
        
        total_fields_updated = 0
        
        for table, pairs in TABLES_LOCALIZATION.items():
            pk = 'slug' if table == 'categories' else 'id'
            
            for col_ar, col_en in pairs:
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
                    
                    # Special check: is this a description column that could be JSON?
                    if col_ar == 'description_ar':
                        new_ar, new_en = translate_json_description(val_ar, val_en)
                    else:
                        # Standard translation logic
                        is_desynced = False
                        if val_ar is None or str(val_ar).strip() == '' or is_english_only(str(val_ar)):
                            is_desynced = True
                            
                        if is_desynced:
                            if table == 'vendor_products' and col_en == 'coordinator_name_en':
                                coord_val = row['coordinator_name']
                                if (not val_en or str(val_en).strip() == '') and coord_val and str(coord_val).strip() != '':
                                    new_en = str(coord_val).strip()
                                    new_ar = translate_to_arabic(new_en)
                                    
                            elif val_ar and is_english_only(str(val_ar)) and (not val_en or str(val_en).strip() == '' or str(val_en).strip() == str(val_ar).strip()):
                                new_en = str(val_ar).strip()
                                new_ar = translate_to_arabic(new_en)
                                
                            elif (not val_ar or str(val_ar).strip() == '') and val_en and str(val_en).strip() != '':
                                new_ar = translate_to_arabic(str(val_en).strip())
                                
                            elif val_ar and is_english_only(str(val_ar)) and val_en and str(val_en).strip() != '':
                                new_ar = translate_to_arabic(str(val_en).strip())
                                
                        # Extra check: If val_ar is Arabic and val_en is NULL, translate it to English if possible (like Sunday Hotel Ballroom)
                        if val_ar and not is_english_only(val_ar) and (not val_en or str(val_en).strip() == ''):
                            if val_ar == 'قاعة صنداي الفندقية للاحتفالات':
                                new_en = 'Sunday Hotel Ballroom'
                                
                    if new_ar != val_ar or new_en != val_en:
                        cur.execute(f"""
                            UPDATE {table} 
                            SET {col_ar} = %s, {col_en} = %s 
                            WHERE {pk} = %s
                        """, (new_ar, new_en, row_id))
                        total_fields_updated += 1
                        
        logging.info(f"Localization Realignment completed. Realigned {total_fields_updated} fields.")
        
        # Validation Check
        desync_left = 0
        for table, pairs in TABLES_LOCALIZATION.items():
            pk = 'slug' if table == 'categories' else 'id'
            for col_ar, col_en in pairs:
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
                    if table == 'vendor_products' and col_ar == 'coordinator_name_ar' and not val_ar and not val_en:
                        continue
                    # Skip check for description_ar if we know both are empty
                    if not val_ar and not val_en:
                        continue
                    if val_ar is None or str(val_ar).strip() == '' or is_english_only(str(val_ar)):
                        desync_left += 1
                        
        logging.info(f"Final desynced fields remaining: {desync_left}")
        
        if execute:
            logging.info("Committing database transactions...")
            conn.commit()
            logging.info("Localization realignment successfully committed!")
        else:
            logging.info("Dry-run. Rolling back database transaction...")
            conn.rollback()
            logging.info("Database rolled back successfully.")
            
    except Exception as e:
        logging.error(f"Realignment failed: {e}")
        conn.rollback()
        sys.exit(1)
        
    conn.close()
    return total_fields_updated, desync_left

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true', help='Execute and commit realignment to the database')
    args = parser.parse_args()
    
    perform_realignment(execute=args.execute)
