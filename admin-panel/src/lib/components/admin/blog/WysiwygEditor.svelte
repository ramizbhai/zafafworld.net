<script lang="ts">
  import { onMount } from 'svelte';
  import { Bold, Italic, Underline, AlignLeft, AlignCenter, AlignRight, List, ListOrdered, Link, Heading2, Heading3, RefreshCw } from 'lucide-svelte';

  let { 
    value = $bindable(''), 
    placeholder = 'Start writing...',
    direction = 'ltr' 
  } = $props<{ 
    value?: string; 
    placeholder?: string;
    direction?: 'ltr' | 'rtl';
  }>();

  let editorRef = $state<HTMLDivElement | null>(null);

  // Sync state from value prop (only on initialization or if it changes externally)
  onMount(() => {
    if (editorRef && editorRef.innerHTML !== value) {
      editorRef.innerHTML = value || '';
    }
  });

  // Track edits and update value
  function handleInput() {
    if (editorRef) {
      value = editorRef.innerHTML;
    }
  }

  // Exec command wrapper
  function execCmd(command: string, argument: string = '') {
    document.execCommand(command, false, argument);
    handleInput();
  }

  // Prompt for link insertion
  function insertLink() {
    const url = prompt('Enter URL (e.g. https://example.com):', 'https://');
    if (url) {
      execCmd('createLink', url);
    }
  }
</script>

<div class="wysiwyg-editor border rounded-xl overflow-hidden bg-white shadow-sm flex flex-col">
  <!-- Toolbar -->
  <div class="editor-toolbar flex flex-wrap items-center gap-1 p-2 bg-slate-50 border-b border-slate-200">
    <button type="button" class="btn-tool" onclick={() => execCmd('bold')} title="Bold">
      <Bold size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('italic')} title="Italic">
      <Italic size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('underline')} title="Underline">
      <Underline size={16} />
    </button>

    <div class="w-px h-6 bg-slate-200 mx-1"></div>

    <button type="button" class="btn-tool" onclick={() => execCmd('formatBlock', '<h2>')} title="Heading 2">
      <Heading2 size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('formatBlock', '<h3>')} title="Heading 3">
      <Heading3 size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('formatBlock', '<p>')} title="Paragraph">
      P
    </button>

    <div class="w-px h-6 bg-slate-200 mx-1"></div>

    <button type="button" class="btn-tool" onclick={() => execCmd('insertUnorderedList')} title="Bullet List">
      <List size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('insertOrderedList')} title="Numbered List">
      <ListOrdered size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={insertLink} title="Insert Link">
      <Link size={16} />
    </button>

    <div class="w-px h-6 bg-slate-200 mx-1"></div>

    <button type="button" class="btn-tool" onclick={() => execCmd('justifyLeft')} title="Align Left">
      <AlignLeft size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('justifyCenter')} title="Align Center">
      <AlignCenter size={16} />
    </button>
    <button type="button" class="btn-tool" onclick={() => execCmd('justifyRight')} title="Align Right">
      <AlignRight size={16} />
    </button>

    <button type="button" class="btn-tool ml-auto" onclick={() => execCmd('removeFormat')} title="Clear Formatting">
      <RefreshCw size={14} />
    </button>
  </div>

  <!-- Content editable area -->
  <div 
    bind:this={editorRef}
    contenteditable="true"
    oninput={handleInput}
    dir={direction}
    class="editor-content p-4 min-h-[300px] max-h-[500px] overflow-y-auto focus:outline-none prose prose-slate max-w-none"
    style="font-family: {direction === 'rtl' ? '\"Noto Sans Arabic\", sans-serif' : 'inherit'};"
    placeholder={placeholder}
  ></div>
</div>

<style>
  .btn-tool {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    color: #475569;
    cursor: pointer;
    transition: all 0.2s;
  }
  .btn-tool:hover {
    background: #f1f5f9;
    color: #0f172a;
    border-color: #e2e8f0;
  }
  .editor-content[contenteditable=true]:empty:before {
    content: attr(placeholder);
    color: #94a3b8;
    pointer-events: none;
    display: block;
  }
  
  /* Reset bullet/numbered lists inside contenteditable to appear properly */
  :global(.editor-content ul) {
    list-style-type: disc !important;
    padding-left: 1.5rem !important;
    margin-top: 0.5rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(.editor-content ol) {
    list-style-type: decimal !important;
    padding-left: 1.5rem !important;
    margin-top: 0.5rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(.editor-content h2) {
    font-size: 1.5rem !important;
    font-weight: 700 !important;
    margin-top: 1rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(.editor-content h3) {
    font-size: 1.25rem !important;
    font-weight: 600 !important;
    margin-top: 1rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(.editor-content p) {
    margin-top: 0.5rem !important;
    margin-bottom: 0.5rem !important;
  }
  :global(.editor-content a) {
    color: #5b21b6 !important;
    text-decoration: underline !important;
  }
</style>
