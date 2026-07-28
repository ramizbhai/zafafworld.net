import os
import psycopg2
from psycopg2.extras import DictCursor

DB_DSN = "postgresql://zafaf_db_admin:PASSWORD@127.0.0.1:5434/zafaf_world"

def check_legacy_urls():
    conn = psycopg2.connect(DB_DSN)
    cur = conn.cursor(cursor_factory=DictCursor)
    
    # Get all text/character columns in all user tables
    cur.execute("""
        SELECT table_name, column_name 
        FROM information_schema.columns 
        WHERE table_schema = 'public' 
          AND data_type IN ('text', 'character varying', 'character');
    """)
    columns = cur.fetchall()
    
    found_any = False
    for row in columns:
        table = row['table_name']
        col = row['column_name']
        
        # We check if there are any values containing '/uploads/' but not '/assets/uploads/'
        query = f"""
            SELECT COUNT(*) 
            FROM "{table}" 
            WHERE "{col}"::text LIKE '%/uploads/%' 
              AND "{col}"::text NOT LIKE '%/assets/uploads/%';
        """
        try:
            cur.execute(query)
            count = cur.fetchone()[0]
            if count > 0:
                print(f"Found {count} rows in table '{table}', column '{col}' with legacy '/uploads/' url.")
                # Print a few examples
                cur.execute(f'SELECT "{col}" FROM "{table}" WHERE "{col}"::text LIKE \'%/uploads/%\' AND "{col}"::text NOT LIKE \'%/assets/uploads/%\' LIMIT 3;')
                examples = cur.fetchall()
                for ex in examples:
                    print(f"  Example: {ex[0]}")
                found_any = True
        except Exception as e:
            conn.rollback()
            # print(f"Error checking {table}.{col}: {e}")
            
    if not found_any:
        print("No legacy '/uploads/' URLs found in any database table.")

if __name__ == '__main__':
    check_legacy_urls()
