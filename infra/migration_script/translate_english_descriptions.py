import psycopg2
import json
import re
import logging
from deep_translator import GoogleTranslator

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"

def translate_block_content(text):
    if not text:
        return text
    # If the text has no English letters, no need to translate
    if not re.search(r'[a-zA-Z]', text):
        return text
    try:
        translated = GoogleTranslator(source='en', target='ar').translate(text)
        return translated
    except Exception as e:
        logging.error(f"Translation failed for '{text[:30]}...': {e}")
        return text

def translate_descriptions():
    conn = psycopg2.connect(DB_DSN)
    cur = conn.cursor()
    
    # We want to identify any row where description_ar contains English text in the JSON content
    cur.execute("SELECT id, description_en, description_ar FROM vendor_products WHERE description_ar ~ '[a-zA-Z]'")
    rows = cur.fetchall()
    
    logging.info(f"Found {len(rows)} descriptions to translate.")
    
    updated_count = 0
    for r_id, desc_en, desc_ar in rows:
        # If desc_en is not available, we translate desc_ar itself
        source_desc = desc_en if desc_en else desc_ar
        if not source_desc:
            continue
            
        try:
            blocks = json.loads(source_desc)
            changed = False
            for b in blocks:
                if 'content' in b:
                    orig = b['content']
                    translated = translate_block_content(orig)
                    if translated != orig:
                        b['content'] = translated
                        changed = True
            
            if changed:
                new_desc_ar = json.dumps(blocks, ensure_ascii=False)
                cur.execute("UPDATE vendor_products SET description_ar = %s WHERE id = %s", (new_desc_ar, r_id))
                updated_count += 1
                if updated_count % 10 == 0:
                    logging.info(f"Translated {updated_count} descriptions...")
        except Exception as e:
            logging.error(f"Error processing description for {r_id}: {e}")
            
    conn.commit()
    logging.info(f"Completed! Translated and updated {updated_count} description fields in database.")
    conn.close()

if __name__ == '__main__':
    translate_descriptions()
