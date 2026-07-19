import os
import re

directory = "/opt/zafafworld.net"

# We only want to replace ' — ' with ' - ' inside <title> tags.
# This regex will match `<title>... — ...</title>`
def process_file(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except:
        return
        
    original = content
    # Find all <title>...</title>
    def replacer(match):
        title_content = match.group(1)
        # Replace em-dash and en-dash with hyphen
        new_title = title_content.replace('—', '-').replace('–', '-')
        return f'<title>{new_title}</title>'
        
    content = re.sub(r'<title>(.*?)</title>', replacer, content, flags=re.DOTALL)
    
    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Fixed titles in {filepath}")

for root, dirs, files in os.walk(directory):
    if '.git' in dirs:
        dirs.remove('.git')
    if 'node_modules' in dirs:
        dirs.remove('node_modules')
    if '.svelte-kit' in dirs:
        dirs.remove('.svelte-kit')
    for file in files:
        if file.endswith(('.svelte', '.html', '.js', '.ts')):
            process_file(os.path.join(root, file))
