import os
import re
from typing import Dict, List, Optional
import pypdf

# Default list of binary or non-text file extensions to skip when scanning directories
IGNORED_EXTENSIONS = {
    '.png', '.jpg', '.jpeg', '.gif', '.ico', '.pdf', '.zip', '.tar', '.gz',
    '.exe', '.dll', '.so', '.dylib', '.db', '.sqlite', '.woff', '.woff2',
    '.ttf', '.eot', '.mp4', '.mp3', '.wav', '.flac', '.pyc', '.class', '.o'
}

# Default list of directories to ignore during recursive scanning
IGNORED_DIRS = {
    '.git', '.svn', '.hg', 'node_modules', 'venv', '.venv', 'env', '__pycache__',
    'target', 'dist', 'build', 'out', '.idea', '.vscode', 'tmp', 'temp', 'vendor'
}

def clean_html(html_content: str) -> str:
    """Simple regex-based HTML tag stripper to get clean text for the LLM."""
    # Remove script and style elements
    clean_content = re.sub(r'<(script|style).*?>.*?</\1>', '', html_content, flags=re.DOTALL | re.IGNORECASE)
    # Remove all other HTML tags
    clean_content = re.sub(r'<[^>]*>', ' ', clean_content)
    # Normalize whitespaces
    clean_content = re.sub(r'\s+', ' ', clean_content).strip()
    return clean_content

def parse_pdf(file_path: str) -> str:
    """Extracts text content from a PDF file using pypdf."""
    text_content = []
    try:
        reader = pypdf.PdfReader(file_path)
        for page_num, page in enumerate(reader.pages):
            page_text = page.extract_text()
            if page_text:
                text_content.append(f"--- Page {page_num + 1} ---\n{page_text}")
    except Exception as e:
        raise ValueError(f"Failed to parse PDF file at {file_path}: {str(e)}")
    return "\n\n".join(text_content)

def parse_text_file(file_path: str) -> str:
    """Reads a text-based file (MD, HTML, TXT, source, etc.) with fallback encoding."""
    encodings = ['utf-8', 'latin-1', 'utf-16', 'utf-32']
    for encoding in encodings:
        try:
            with open(file_path, 'r', encoding=encoding) as f:
                content = f.read()
                if file_path.lower().endswith(('.html', '.htm')):
                    return clean_html(content)
                return content
        except UnicodeDecodeError:
            continue
    raise ValueError(f"Could not decode text file {file_path} with common encodings.")

def scan_directory(dir_path: str, max_depth: int = 5, current_depth: int = 0) -> List[Dict[str, str]]:
    """
    Recursively scans a directory for source code and documentation files.
    Returns a list of dictionaries containing file paths (relative) and content.
    """
    if current_depth > max_depth:
        return []

    files_data = []
    try:
        for entry in os.scandir(dir_path):
            if entry.is_dir():
                if entry.name in IGNORED_DIRS:
                    continue
                files_data.extend(scan_directory(entry.path, max_depth, current_depth + 1))
            elif entry.is_file():
                ext = os.path.splitext(entry.name)[1].lower()
                if ext in IGNORED_EXTENSIONS:
                    continue
                try:
                    content = parse_text_file(entry.path)
                    # Get relative path for cleaner output and less token usage
                    rel_path = os.path.relpath(entry.path, start=os.path.dirname(dir_path))
                    files_data.append({
                        "path": rel_path,
                        "content": content
                    })
                except Exception:
                    # Skip files that fail to parse (e.g. binary without matching extension)
                    continue
    except Exception as e:
        print(f"Warning: Failed to scan directory {dir_path}: {str(e)}")
    
    return files_data

def get_context_from_path(path: str) -> str:
    """
    Main entrance function. Detects if path is file or directory,
    parses, and returns a single formatted context string.
    """
    if not os.path.exists(path):
        raise FileNotFoundError(f"Path does not exist: {path}")

    if os.path.isfile(path):
        ext = os.path.splitext(path)[1].lower()
        if ext == '.pdf':
            return f"=== FILE: {os.path.basename(path)} ===\n\n{parse_pdf(path)}"
        else:
            return f"=== FILE: {os.path.basename(path)} ===\n\n{parse_text_file(path)}"
    
    elif os.path.isdir(path):
        files = scan_directory(path)
        formatted_files = []
        for f in files:
            formatted_files.append(f"=== FILE PATH: {f['path']} ===\n{f['content']}\n=== END OF FILE ===")
        return "\n\n".join(formatted_files)
    
    else:
        raise ValueError(f"Unknown path type at {path}")
