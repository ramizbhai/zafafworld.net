import psycopg2
import json
import re
import logging

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
    
    'address': 'العنوان', 'alhamra': 'الحمراء', 'alramala': 'الرمال', 'apartments': 'شقق',
    'aren': 'أرين', 'art': 'آرت', 'banqueting': 'للحفلات', 'bilad': 'البلاد', 'boutique': 'بوتيك',
    'casa': 'كازا', 'center': 'المركز', 'city': 'سيتي', 'club': 'نادي', 'concorde': 'كونكورد',
    'continental': 'كونتيننتال', 'coordinator': 'منسق', 'crowne': 'كراون', 'crystal': 'كريستال',
    'd': 'دي', 'dareen': 'دارين', 'dawadmi': 'الدوادمي', 'dawasir': 'الدواسر', 'details': 'تفاصيل',
    'dfgjhgfjdhsgwfeqdert': '', 'dhabab': 'الضباب', 'di': 'دي', 'diplomacy': 'الدبلوماسي',
    'donatello': 'دوناتيلو', 'doubletree': 'دبل تري', 'dylan': 'ديلان', 'elegance': 'إيليغانس',
    'event': 'إيفنت', 'faf': '', 'fahd': 'فهد', 'financial': 'المالي', 'foco': 'فوكو',
    'fog': 'الضباب', 'four': 'فور', 'fursan': 'الفرسان', 'galleria': 'غاليريا', 'hala': 'هلا',
    'hamra': 'الحمراء', 'hayat': 'حياة', 'hedge': 'هيدج', 'hilton': 'هيلتون', 'holiday': 'هوليدي',
    'homes': 'بيوت', 'hudub': 'هدب', 'ihg': 'آي إتش جي', 'ikleel': 'إكليل', 'ilyas': 'إلياس',
    'izdihar': 'الازدهار', 'jaddah': 'جدة', 'jwar': 'جوار', 'khaldia': 'الخالدية', 'king': 'الملك',
    'la': 'لا', 'lamir': 'لامير', 'lani': 'لاني', 'layalina': 'ليالينا', 'laylat': 'ليلة',
    'le': 'لو', 'louvian': 'لوفيان', 'loville': 'لوفيل', 'luxury': 'فاخر', 'madarim': 'مداريم',
    'maison': 'ميزون', 'majmaah': 'المجمعة', 'makhmaliah': 'المخملية', 'mandarin': 'ماندارين',
    'manhal': 'المنهل', 'marina': 'مارينا', 'masaya': 'مسايا', 'mashreq': 'المشرق', 'masi': 'الماسي',
    'mawasim': 'مواسم', 'melenia': 'ميلينيا', 'mer': 'مير', 'milan': 'ميلان', 'momayaz': 'مميز',
    'monamore': 'مونامور', 'monasbti': 'مناسبتي', 'mothhalah': 'مذهلة', 'move': 'موف',
    'munsiyah': 'المونسية', 'muon': 'ميون', 'naseem': 'النسيم', 'nayyara': 'نيارة', 'nights': 'ليالي',
    'oasis': 'واحة', 'obhur': 'أبحر', 'omar': 'عمر', 'ora': 'أورا', 'oriental': 'الشرقي',
    'plaz': 'بلازا', 'points': 'بوانت', 'press': 'برس', 'rabia': 'ربيعة', 'rabwa': 'الربوة',
    'ramiz': 'رامز', 'rawasi': 'رواسي', 'raza': 'رضا', 'rdc': 'آر دي سي', 'red': 'ريد',
    'redwaves': 'ريد ويفز', 'reem': 'ريم', 'refal': 'رفال', 'regency': 'ريجنسي', 'rihanna': 'ريحانا',
    'rikaz': 'ركاز', 'riyad': 'الرياض', 'romance': 'رومانس', 'rose': 'روز', 'rosemond': 'روزموند',
    's': '', 'sahel': 'الساحل', 'samar': 'سمر', 'seafront': 'الواجهة البحرية', 'senior': 'سينيور',
    'sharq': 'الشرق', 'sharqi': 'الشرقي', 'sky': 'سكاي', 'space': 'الفضاء', 'star': 'ستار',
    'suites': 'أجنحة', 'sukoon': 'سكون', 'sulaimania': 'السليمانية', 'sunday': 'صنداي', 'sy': '',
    'taeesh': 'تعايش', 'tahlia': 'التحلية', 'tamayoz': 'التميز', 'tanfeethiyoon': 'التنفيذيين',
    'tara': 'تارا', 'tashrifat': 'التشريفات', 'temandra': 'تيماندرا', 'test': 'تجريبي',
    'the': '', 'trident': 'ترايدنت', 'tuwaiq': 'طويق', 'ulya': 'العليا', 'upload': 'تحميل',
    'vendor': 'مورد', 'venue': 'الموقع', 'view': 'فيو', 'wa': 'و', 'wadi': 'وادي',
    'waha': 'الواحة', 'waves': 'ويفز',
    
    'al': 'ال', 'queen': 'الملكة', 'spa': 'سبا', 'ihg': 'آي إتش جي', 'in': 'إن',
    'resort': 'منتجع', 'palace': 'قصر', 'gala': 'قاعات', 'hotel': 'فندق',
    'ballroom': 'قاعة', 'hall': 'قاعة', 'concorde': 'كونكورد', 'dhabab': 'الضباب',
    'donatello': 'دوناتيلو', 'jaddah': 'جدة', 'momayaz': 'مميز', 'oriental': 'الشرقي',
    'plaz': 'بلازا', 'rosemond': 'روزموند', 'sulaimania': 'السليمانية', 'convention': 'المؤتمرات',
    'residences': 'شقق سكنية', 'mandarin': 'ماندارين', 'bilad': 'البلاد', 'halls': 'قاعات',
    'ballrooms': 'قاعات', 'suites': 'أجنحة', 'mövenpick': 'موفنبيك'
}

def clean_mixed_arabic(text):
    if not text:
        return text
    def repl(m):
        word = m.group(0).lower()
        return TRANSLATIONS.get(word, m.group(0))
    cleaned = re.sub(r'[a-zA-ZöÖ]+', repl, text)
    cleaned = re.sub(r'\s+', ' ', cleaned).strip()
    return cleaned

def clean_descriptions():
    conn = psycopg2.connect(DB_DSN)
    cur = conn.cursor()
    
    cur.execute("SELECT id, description_ar FROM vendor_products WHERE description_ar ~ '[a-zA-Z]'")
    rows = cur.fetchall()
    
    logging.info(f"Found {len(rows)} JSON descriptions with mixed English characters.")
    
    updated_count = 0
    for r_id, desc_ar in rows:
        if not desc_ar:
            continue
            
        try:
            blocks = json.loads(desc_ar)
            changed = False
            for b in blocks:
                if 'content' in b:
                    orig = b['content']
                    cleaned = clean_mixed_arabic(orig)
                    if cleaned != orig:
                        b['content'] = cleaned
                        changed = True
            
            if changed:
                new_desc_ar = json.dumps(blocks, ensure_ascii=False)
                cur.execute("UPDATE vendor_products SET description_ar = %s WHERE id = %s", (new_desc_ar, r_id))
                updated_count += 1
        except Exception as e:
            logging.error(f"Error parsing description JSON for {r_id}: {e}")
            
    conn.commit()
    logging.info(f"Cleaned {updated_count} description fields in database.")
    conn.close()

if __name__ == '__main__':
    clean_descriptions()
