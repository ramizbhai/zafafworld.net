import glob

for filepath in glob.glob('/opt/zafafworld.net/backend-rust/migrations/*.sql'):
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    with open(filepath, 'w') as f:
        for line in lines:
            if line.strip().upper() not in ['BEGIN;', 'COMMIT;']:
                f.write(line)
print("Stripped transactions from all SQL files.")
