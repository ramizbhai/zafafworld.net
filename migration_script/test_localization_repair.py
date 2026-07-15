import re
import psycopg2
from psycopg2.extras import DictCursor

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
    'nader': 'نادر', 'ader': 'نادر'
}

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

def run_dry_run():
    conn = psycopg2.connect(DB_DSN)
    cur = conn.cursor(cursor_factory=DictCursor)
    
    total_desync_fields = 0
    total_repaired_fields = 0
    
    for table, pairs in TABLES_LOCALIZATION.items():
        print(f"Checking table '{table}':")
        pk = 'slug' if table == 'categories' else 'id'
        for col_ar, col_en in pairs:
            cur.execute(f"SELECT {pk}, {col_ar}, {col_en} FROM {table}")
            rows = cur.fetchall()
            
            pair_desyncs = 0
            pair_repairs = 0
            
            for row in rows:
                val_ar = row[col_ar]
                val_en = row[col_en]
                row_id = row[pk]
                
                # Check for desync
                is_desynced = False
                if val_ar is None or str(val_ar).strip() == '' or is_english_only(str(val_ar)):
                    is_desynced = True
                    pair_desyncs += 1
                
                if is_desynced:
                    # Determine repaired values
                    new_ar = val_ar
                    new_en = val_en
                    
                    if val_ar and is_english_only(str(val_ar)) and (not val_en or str(val_en).strip() == '' or str(val_en).strip() == str(val_ar).strip()):
                        new_en = str(val_ar).strip()
                        new_ar = translate_to_arabic(new_en)
                    elif (not val_ar or str(val_ar).strip() == '') and val_en and str(val_en).strip() != '':
                        new_ar = translate_to_arabic(str(val_en).strip())
                    elif val_ar and is_english_only(str(val_ar)) and val_en and str(val_en).strip() != '':
                        new_ar = translate_to_arabic(str(val_en).strip())
                    
                    # If we generated new_ar/new_en that changed from original
                    if new_ar != val_ar or new_en != val_en:
                        pair_repairs += 1
                        
            print(f"  - Pair {col_ar}/{col_en}: {pair_desyncs} desynced rows, {pair_repairs} would be repaired.")
            total_desync_fields += pair_desyncs
            total_repaired_fields += pair_repairs
            
    print(f"\nTotal desynced fields checked: {total_desync_fields}")
    print(f"Total fields that would be repaired: {total_repaired_fields}")
    conn.close()

if __name__ == '__main__':
    run_dry_run()
