<script lang="ts">
    import { Bold, List, ListOrdered, RemoveFormatting } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';

    const i18n = getI18n();

    let { 
        value = $bindable(''), 
        id, 
        name, 
        placeholder = '', 
        dir = 'ltr' 
    } = $props<{
        value?: string;
        id?: string;
        name?: string;
        placeholder?: string;
        dir?: 'ltr' | 'rtl';
    }>();

    let editorRef = $state<HTMLDivElement | null>(null);

    // Track active formatting states
    let isBold = $state(false);
    let isUl = $state(false);
    let isOl = $state(false);

    // Strip HTML tags to get pure text word count
    function getWordCount(html: string): number {
        if (!html) return 0;
        const text = html.replace(/<[^>]*>/g, ' ');
        return text.trim().split(/\s+/).filter(w => w.length > 0).length;
    }

    let wordCount = $derived(getWordCount(value));

    // Handle formatting actions
    function format(command: string) {
        document.execCommand(command, false);
        updateValue();
        checkActiveStates();
        if (editorRef) {
            editorRef.focus();
        }
    }

    function updateValue() {
        if (editorRef) {
            value = editorRef.innerHTML;
            // Treat empty/placeholder markup as empty string
            if (value === '<br>' || value === '<p><br></p>' || value === '<div><br></div>') {
                value = '';
            }
        }
    }

    function checkActiveStates() {
        isBold = document.queryCommandState('bold');
        isUl = document.queryCommandState('insertUnorderedList');
        isOl = document.queryCommandState('insertOrderedList');
    }

    // Sync initial value to editor element
    $effect(() => {
        if (editorRef && editorRef.innerHTML !== value) {
            editorRef.innerHTML = value || '';
        }
    });
</script>

<div class="rich-editor-container" {dir}>
    <!-- Toolbar -->
    <div class="rich-editor-toolbar">
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isBold}
            onclick={() => format('bold')}
            title={i18n.locale === 'ar' ? 'خط عريض' : 'Bold'}
        >
            <Bold size={16} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isUl}
            onclick={() => format('insertUnorderedList')}
            title={i18n.locale === 'ar' ? 'قائمة نقطية' : 'Bullet List'}
        >
            <List size={16} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isOl}
            onclick={() => format('insertOrderedList')}
            title={i18n.locale === 'ar' ? 'قائمة رقمية' : 'Numbered List'}
        >
            <ListOrdered size={16} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            onclick={() => format('removeFormat')}
            title={i18n.locale === 'ar' ? 'إزالة التنسيق' : 'Clear Formatting'}
        >
            <RemoveFormatting size={16} />
        </button>
    </div>

    <!-- Editable Area -->
    <div 
        bind:this={editorRef}
        contenteditable="true"
        role="textbox"
        aria-multiline="true"
        tabindex="0"
        class="rich-editor-area"
        oninput={updateValue}
        onkeyup={checkActiveStates}
        onmouseup={checkActiveStates}
        {placeholder}
    ></div>

    <!-- Word Counter -->
    <div class="rich-editor-footer">
        <span class="word-counter" class:near-limit={wordCount > 1800} class:at-limit={wordCount >= 2000}>
            {wordCount}/2000 {i18n.locale === 'ar' ? 'كلمة' : 'words'}
        </span>
    </div>

    <!-- Hidden input to submit with the standard HTML form action -->
    <input type="hidden" {id} {name} value={value} />
</div>

<style>
    .rich-editor-container {
        display: flex;
        flex-direction: column;
        border: 1px solid var(--border-color, #e2e8f0);
        border-radius: 8px;
        background: #ffffff;
        overflow: hidden;
        font-family: inherit;
        width: 100%;
    }
    
    .rich-editor-toolbar {
        display: flex;
        gap: 6px;
        padding: 8px 12px;
        background: #f8fafc;
        border-bottom: 1px solid #e2e8f0;
    }
    
    .toolbar-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border: none;
        background: transparent;
        color: #475569;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    
    .toolbar-btn:hover {
        background: #f1f5f9;
        color: #0f172a;
    }
    
    .toolbar-btn.active {
        background: #e2e8f0;
        color: #0f172a;
    }
    
    .rich-editor-area {
        min-height: 160px;
        max-height: 400px;
        overflow-y: auto;
        padding: 14px;
        outline: none;
        line-height: 1.6;
        font-size: 14px;
        color: #1e293b;
        background: #ffffff;
    }
    
    /* Contenteditable placeholder styling */
    .rich-editor-area:empty:before {
        content: attr(placeholder);
        color: #94a3b8;
        pointer-events: none;
    }
    
    .rich-editor-footer {
        display: flex;
        justify-content: flex-end;
        padding: 6px 12px;
        background: #f8fafc;
        border-top: 1px solid #e2e8f0;
    }
    
    .word-counter {
        font-size: 11px;
        color: #64748b;
        font-weight: 600;
    }
    
    .word-counter.near-limit {
        color: #d97706; /* amber */
    }
    
    .word-counter.at-limit {
        color: #dc2626; /* red */
    }
</style>
