<script lang="ts">
    import { 
        Bold, 
        Italic, 
        Underline, 
        Strikethrough,
        List, 
        ListOrdered, 
        AlignLeft,
        AlignCenter,
        AlignRight,
        Minus,
        Image as ImageIcon,
        RemoveFormatting 
    } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { apiClient } from '$lib/api/client';
    import { resolveMediaUrl } from '$lib/shared/utils/media';

    const i18n = getI18n();

    let { 
        value = $bindable(''), 
        id, 
        name, 
        placeholder = '', 
        dir = 'ltr',
        token = ''
    } = $props<{
        value?: string;
        id?: string;
        name?: string;
        placeholder?: string;
        dir?: 'ltr' | 'rtl';
        token?: string;
    }>();

    let editorRef = $state<HTMLDivElement | null>(null);
    let fileInputRef = $state<HTMLInputElement | null>(null);

    // Track active formatting states
    let isBold = $state(false);
    let isItalic = $state(false);
    let isUnderline = $state(false);
    let isStrike = $state(false);
    let isUl = $state(false);
    let isOl = $state(false);
    let isAlignLeft = $state(false);
    let isAlignCenter = $state(false);
    let isAlignRight = $state(false);
    let currentFormatBlock = $state('p');
    let isUploading = $state(false);

    // Strip HTML tags to get pure text word count
    function getWordCount(html: string): number {
        if (!html) return 0;
        const text = html.replace(/<[^>]*>/g, ' ');
        return text.trim().split(/\s+/).filter(w => w.length > 0).length;
    }

    let wordCount = $derived(getWordCount(value));

    // Handle formatting actions
    function format(command: string, valueArgument: string = '') {
        document.execCommand(command, false, valueArgument);
        updateValue();
        checkActiveStates();
        if (editorRef) {
            editorRef.focus();
        }
    }

    function handleHeadingChange(e: Event) {
        const target = e.target as HTMLSelectElement;
        const val = target.value;
        if (val === 'p') {
            format('formatBlock', 'p');
        } else {
            format('formatBlock', val);
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
        isItalic = document.queryCommandState('italic');
        isUnderline = document.queryCommandState('underline');
        isStrike = document.queryCommandState('strikeThrough');
        isUl = document.queryCommandState('insertUnorderedList');
        isOl = document.queryCommandState('insertOrderedList');
        isAlignLeft = document.queryCommandState('justifyLeft');
        isAlignCenter = document.queryCommandState('justifyCenter');
        isAlignRight = document.queryCommandState('justifyRight');

        // Check format block heading state
        try {
            const block = document.queryCommandValue('formatBlock');
            currentFormatBlock = block ? block.toLowerCase() : 'p';
            if (currentFormatBlock !== 'h1' && currentFormatBlock !== 'h2' && currentFormatBlock !== 'h3') {
                currentFormatBlock = 'p';
            }
        } catch {
            currentFormatBlock = 'p';
        }
    }

    // Trigger file chooser for image upload
    function triggerImageUpload() {
        if (fileInputRef) {
            fileInputRef.click();
        }
    }

    // Handle image uploading
    async function handleFileChange(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) return;
        const file = input.files[0];

        // Save selection range before async operation to avoid losing cursor focus position
        const selection = window.getSelection();
        let savedRange: Range | null = null;
        if (selection && selection.rangeCount > 0) {
            savedRange = selection.getRangeAt(0).cloneRange();
        }

        isUploading = true;
        try {
            if (token) {
                const formData = new FormData();
                formData.append('file', file);
                
                const res = await apiClient.vendor.uploadMedia(token, formData);
                if (res.error) {
                    throw new Error(res.error.message || 'Image upload failed');
                }
                const { url } = res.data || {};
                if (!url) {
                    throw new Error('No URL returned from upload');
                }

                const imageUrl = resolveMediaUrl(url);
                insertImageHtml(imageUrl, savedRange, selection);
            } else {
                // Fallback: Read local file as base64 data URL
                const reader = new FileReader();
                reader.onload = (event) => {
                    const base64Url = event.target?.result as string;
                    insertImageHtml(base64Url, savedRange, selection);
                };
                reader.readAsDataURL(file);
            }
        } catch (err: any) {
            console.error(err);
            alert(err.message || 'Failed to upload image');
        } finally {
            isUploading = false;
            input.value = ''; // Reset input element
        }
    }

    function insertImageHtml(imageUrl: string, savedRange: Range | null, selection: Selection | null) {
        const imgHtml = `<img src="${imageUrl}" alt="Image" style="max-width: 100%; height: auto; border-radius: 8px; margin: 16px 0; display: block;" />`;
        
        if (editorRef) {
            editorRef.focus();
            if (savedRange && selection) {
                selection.removeAllRanges();
                selection.addRange(savedRange);
            }
            
            if (selection && selection.rangeCount > 0) {
                const range = selection.getRangeAt(0);
                range.deleteContents();
                
                const div = document.createElement('div');
                div.innerHTML = imgHtml;
                const frag = document.createDocumentFragment();
                let node;
                while ((node = div.firstChild)) {
                    frag.appendChild(node);
                }
                range.insertNode(frag);
                // Collapse cursor after inserted image
                range.collapse(false);
            } else {
                editorRef.innerHTML += imgHtml;
            }
            updateValue();
        }
    }

    // Sync initial value to editor element
    $effect(() => {
        if (editorRef && editorRef.innerHTML !== value) {
            editorRef.innerHTML = value || '';
        }
    });
</script>

<div class="rich-editor-container" {dir}>
    <!-- Hidden file input for uploading images -->
    <input 
        type="file" 
        accept="image/*" 
        bind:this={fileInputRef} 
        onchange={handleFileChange} 
        style="display: none;" 
    />

    <!-- Toolbar -->
    <div class="rich-editor-toolbar">
        <!-- Formatting Block select -->
        <select 
            value={currentFormatBlock} 
            onchange={handleHeadingChange}
            class="toolbar-select"
            title={i18n.locale === 'ar' ? 'نمط النص' : 'Text Style'}
        >
            <option value="p">{i18n.locale === 'ar' ? 'فقرة عادية' : 'Normal Text'}</option>
            <option value="h1">{i18n.locale === 'ar' ? 'عنوان رئيسي 1' : 'Heading 1'}</option>
            <option value="h2">{i18n.locale === 'ar' ? 'عنوان رئيسي 2' : 'Heading 2'}</option>
            <option value="h3">{i18n.locale === 'ar' ? 'عنوان رئيسي 3' : 'Heading 3'}</option>
        </select>

        <div class="toolbar-divider"></div>

        <!-- Inline formatting buttons -->
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isBold}
            onclick={() => format('bold')}
            title={i18n.locale === 'ar' ? 'خط عريض' : 'Bold'}
        >
            <Bold size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isItalic}
            onclick={() => format('italic')}
            title={i18n.locale === 'ar' ? 'خط مائل' : 'Italic'}
        >
            <Italic size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isUnderline}
            onclick={() => format('underline')}
            title={i18n.locale === 'ar' ? 'تحته خط' : 'Underline'}
        >
            <Underline size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isStrike}
            onclick={() => format('strikeThrough')}
            title={i18n.locale === 'ar' ? 'يتوسطه خط' : 'Strikethrough'}
        >
            <Strikethrough size={15} />
        </button>

        <div class="toolbar-divider"></div>

        <!-- Alignments -->
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isAlignLeft}
            onclick={() => format('justifyLeft')}
            title={i18n.locale === 'ar' ? 'محاذاة لليسار' : 'Align Left'}
        >
            <AlignLeft size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isAlignCenter}
            onclick={() => format('justifyCenter')}
            title={i18n.locale === 'ar' ? 'محاذاة للوسط' : 'Align Center'}
        >
            <AlignCenter size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isAlignRight}
            onclick={() => format('justifyRight')}
            title={i18n.locale === 'ar' ? 'محاذاة لليمين' : 'Align Right'}
        >
            <AlignRight size={15} />
        </button>

        <div class="toolbar-divider"></div>

        <!-- Lists -->
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isUl}
            onclick={() => format('insertUnorderedList')}
            title={i18n.locale === 'ar' ? 'قائمة نقطية' : 'Bullet List'}
        >
            <List size={15} />
        </button>
        <button 
            type="button" 
            class="toolbar-btn" 
            class:active={isOl}
            onclick={() => format('insertOrderedList')}
            title={i18n.locale === 'ar' ? 'قائمة رقمية' : 'Numbered List'}
        >
            <ListOrdered size={15} />
        </button>

        <div class="toolbar-divider"></div>

        <!-- Divider and Image Insertion -->
        <button 
            type="button" 
            class="toolbar-btn" 
            onclick={() => format('insertHorizontalRule')}
            title={i18n.locale === 'ar' ? 'إدراج فاصل خطي' : 'Insert Divider'}
        >
            <Minus size={15} />
        </button>
        
        <button 
            type="button" 
            class="toolbar-btn" 
            onclick={triggerImageUpload}
            disabled={isUploading}
            title={i18n.locale === 'ar' ? 'إدراج صورة' : 'Insert Image'}
        >
            {#if isUploading}
                <div class="mini-spinner"></div>
            {:else}
                <ImageIcon size={15} />
            {/if}
        </button>

        <div class="toolbar-divider"></div>

        <!-- Remove format -->
        <button 
            type="button" 
            class="toolbar-btn" 
            onclick={() => format('removeFormat')}
            title={i18n.locale === 'ar' ? 'إزالة التنسيق' : 'Clear Formatting'}
        >
            <RemoveFormatting size={15} />
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

    <!-- Hidden input to submit with standard HTML form action if needed -->
    <input type="hidden" {id} {name} value={value} />
</div>

<style>
    .rich-editor-container {
        display: flex;
        flex-direction: column;
        border: 1px solid var(--border-color, #e2e8f0);
        border-radius: 12px;
        background: #ffffff;
        overflow: hidden;
        font-family: inherit;
        width: 100%;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    }
    
    .rich-editor-toolbar {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 4px;
        padding: 8px 12px;
        background: #f8fafc;
        border-bottom: 1px solid #e2e8f0;
    }

    .toolbar-select {
        padding: 4px 8px;
        border-radius: 6px;
        border: 1px solid #cbd5e1;
        background-color: white;
        font-size: 0.85rem;
        color: #334155;
        outline: none;
        cursor: pointer;
    }
    
    .toolbar-divider {
        width: 1px;
        height: 18px;
        background-color: #cbd5e1;
        margin: 0 4px;
    }

    .toolbar-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        border: none;
        background: transparent;
        color: #475569;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.15s ease;
    }
    
    .toolbar-btn:hover:not(:disabled) {
        background: #e2e8f0;
        color: #0f172a;
    }
    
    .toolbar-btn.active {
        background: #cbd5e1;
        color: #0f172a;
    }

    .toolbar-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    
    .rich-editor-area {
        min-height: 220px;
        max-height: 480px;
        overflow-y: auto;
        padding: 16px;
        outline: none;
        line-height: 1.7;
        font-size: 15px;
        color: #1e293b;
        background: #ffffff;
    }

    /* Editor inner styling content */
    :global(.rich-editor-area h1) {
        font-size: 1.8rem;
        font-weight: 700;
        margin-top: 16px;
        margin-bottom: 8px;
        color: #0f172a;
    }
    :global(.rich-editor-area h2) {
        font-size: 1.45rem;
        font-weight: 600;
        margin-top: 14px;
        margin-bottom: 6px;
        color: #1e293b;
    }
    :global(.rich-editor-area h3) {
        font-size: 1.25rem;
        font-weight: 600;
        margin-top: 12px;
        margin-bottom: 4px;
        color: #334155;
    }
    :global(.rich-editor-area p) {
        margin-bottom: 12px;
    }
    :global(.rich-editor-area ul) {
        list-style-type: disc;
        margin-left: 24px;
        margin-bottom: 12px;
    }
    :global(.rich-editor-area ol) {
        list-style-type: decimal;
        margin-left: 24px;
        margin-bottom: 12px;
    }
    :global(.rich-editor-area img) {
        max-width: 100%;
        height: auto;
        border-radius: 8px;
        margin: 16px 0;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    }
    :global(.rich-editor-area hr) {
        border: 0;
        border-top: 1px solid #e2e8f0;
        margin: 20px 0;
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

    .mini-spinner {
        width: 14px;
        height: 14px;
        border: 2px solid #cbd5e1;
        border-top-color: #6366f1;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }
</style>
