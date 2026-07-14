<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import type { PageData } from './$types';

  // Get SSR-safe translations context
  const i18n = getI18n();

  let { data }: { data: PageData } = $props();

  // Selected template ID state
  let selectedId = $state<string | null>(null);
  let activeTemplate = $derived(
    (data.templates || []).find((t: any) => t.id === selectedId) || (data.templates || [])[0]
  );

  // Editor states (temporary workspace before save)
  let editorName = $state('');
  let editorBodyAr = $state('');
  let editorBodyEn = $state('');
  let activeFocusField = $state<'ar' | 'en'>('ar');

  // Update editor values when active template changes
  $effect(() => {
    if (activeTemplate) {
      editorName = activeTemplate.template_name || '';
      editorBodyAr = activeTemplate.body_text_ar || '';
      editorBodyEn = activeTemplate.body_text_en || '';
    } else {
      editorName = '';
      editorBodyAr = '';
      editorBodyEn = '';
    }
  });

  // Preview variables replacement
  let previewText = $derived.by(() => {
    let text = i18n.locale === 'ar' ? editorBodyAr : editorBodyEn;
    const coupleSample = i18n.locale === 'ar' ? 'أحمد محمد' : 'Ahmed Mohamed';
    const dateSample = i18n.locale === 'ar' ? '١٢ أكتوبر ٢٠٢٦' : '12 Oct 2026';
    
    text = text.replace(/{CoupleName}/g, coupleSample);
    text = text.replace(/{EventDate}/g, dateSample);
    return text;
  });

  function insertVariable(variable: string) {
    if (activeFocusField === 'ar') {
      editorBodyAr += ` ${variable} `;
    } else {
      editorBodyEn += ` ${variable} `;
    }
  }
</script>

<svelte:head>
  <title>{i18n.t.nav?.templates || 'Templates'} – {i18n.t.common?.appName || 'Zafaf Portal'}</title>
</svelte:head>

<div class="templates-layout">
  <!-- Left panel: Templates list -->
  <div class="card templates-sidebar">
    <div class="sidebar-hdr">
      <h3>{i18n.t.templates?.title || 'Templates'}</h3>
      
      <form method="POST" action="?/create" use:enhance={() => {
        return async ({ result, update }) => {
          if (result.type === 'success') {
            if (result.data) {
              // Select the newly created template
              const created = (result.data as any).template;
              if (created && created.id) {
                selectedId = created.id;
              }
            }
            await invalidateAll();
          }
          await update();
        };
      }}>
        <input type="hidden" name="template_name" value={i18n.locale === 'ar' ? 'نموذج جديد' : 'New Template'} />
        <input type="hidden" name="body_text_ar" value={"مرحباً {CoupleName}، يسعدنا استفساركم لمناسبة تاريخ {EventDate}..."} />
        <input type="hidden" name="body_text_en" value={"Hello {CoupleName}, we are glad to receive your inquiry for {EventDate}..."} />
        <button type="submit" class="btn btn-primary btn-sm">
          + {i18n.t.common?.add || 'Add'}
        </button>
      </form>
    </div>

    <div class="templates-list-items">
      {#each data.templates || [] as item (item.id)}
        <div
          class="template-item-btn {activeTemplate?.id === item.id ? 'active' : ''}"
          role="button"
          tabindex="0"
          onclick={() => selectedId = item.id}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedId = item.id; } }}
        >
          <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
            <span class="item-title">{item.template_name}</span>
            
            {#if (data.templates || []).length > 1}
              <form method="POST" action="?/delete" use:enhance={() => {
                return async ({ result, update }) => {
                  if (result.type === 'success') {
                    // Reset selected ID to let it pick first template
                    selectedId = null;
                    await invalidateAll();
                  }
                  await update();
                };
              }}>
                <input type="hidden" name="id" value={item.id} />
                <button
                  type="submit"
                  class="item-delete-btn"
                  onclick={(e) => {
                    if (!confirm(i18n.locale === 'ar' ? 'هل أنت متأكد من حذف هذا النموذج؟' : 'Are you sure you want to delete this template?')) {
                      e.preventDefault();
                    }
                  }}
                  title={i18n.t.common?.delete || 'Delete'}
                  aria-label={i18n.t.common?.delete || 'Delete'}
                >
                  &times;
                </button>
              </form>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Right panel: Active Editor & Live Preview -->
  {#if activeTemplate}
    <div class="editor-preview-container">
      <!-- Editor Card -->
      <form method="POST" action="?/update" use:enhance={() => {
        return async ({ result, update }) => {
          if (result.type === 'success') await invalidateAll();
          await update();
        };
      }} class="card editor-card">
        <input type="hidden" name="id" value={activeTemplate.id} />
        
        <div class="form-group">
          <label class="form-label" for="temp-title">{i18n.t.templates?.tempTitle || 'Template Name'}</label>
          <input
            id="temp-title"
            name="template_name"
            type="text"
            class="form-input"
            bind:value={editorName}
            required
          />
        </div>

        <div class="form-group">
          <label class="form-label" style="display: flex; justify-content: space-between; align-items: center;">
            <span>{i18n.t.templates?.body || 'Message Body'}</span>
            
            <div class="variable-chips">
              <button type="button" class="chip" onclick={() => insertVariable('{CoupleName}')}>
                + {i18n.t.couples?.name || 'Couple Name'}
              </button>
              <button type="button" class="chip" onclick={() => insertVariable('{EventDate}')}>
                + {i18n.t.couples?.eventDate || 'Wedding Date'}
              </button>
            </div>
          </label>

          <!-- Bilingual inputs stacked with localization directions -->
          <div class="bilingual-editors">
            <div class="editor-block">
              <span class="lang-label">العربية (Arabic)</span>
              <textarea
                name="body_text_ar"
                class="form-textarea rtl-direction"
                rows="4"
                bind:value={editorBodyAr}
                onfocus={() => activeFocusField = 'ar'}
              ></textarea>
            </div>

            <div class="editor-block">
              <span class="lang-label">English</span>
              <textarea
                name="body_text_en"
                class="form-textarea ltr-direction"
                rows="4"
                bind:value={editorBodyEn}
                onfocus={() => activeFocusField = 'en'}
              ></textarea>
            </div>
          </div>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 16px;">
          <button type="submit" class="btn btn-primary">
            {i18n.t.common?.save || 'Save'}
          </button>
        </div>
      </form>

      <!-- Live Mock WhatsApp Preview (Visual Wow) -->
      <div class="card preview-card">
        <div class="whatsapp-hdr">
          <div class="wa-avatar">📞</div>
          <div>
            <div class="wa-name">Zafaf Auto Reply</div>
            <div class="wa-status">Online</div>
          </div>
        </div>
        <div class="whatsapp-body">
          <div class="wa-message-bubble">
            <p>{previewText || '...'}</p>
            <span class="wa-time">10:42 AM ✓✓</span>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="empty-state" style="flex: 2;">
      <div class="empty-icon">📋</div>
      <h3>{i18n.t.nav?.templates || 'Templates'}</h3>
      <p>{i18n.t.templates?.empty || 'No templates configured.'}</p>
    </div>
  {/if}
</div>

<style>
  .templates-layout {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 20px;
    margin-bottom: 24px;
  }

  .templates-sidebar {
    padding: 20px;
    display: flex;
    flex-direction: column;
    height: 600px;
    overflow-y: auto;
  }

  .sidebar-hdr {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .templates-list-items {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .template-item-btn {
    text-align: var(--text-align);
    background: none;
    border: 1px solid var(--border);
    padding: 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font);
    font-size: 13.5px;
    color: var(--text-sec);
    transition: all 0.15s;
    outline: none;
  }

  .template-item-btn:hover {
    border-color: var(--teal);
    background: var(--teal-light);
    color: var(--teal);
  }

  .template-item-btn.active {
    border-color: var(--teal);
    background: var(--teal);
    color: #fff;
  }

  .item-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 150px;
  }

  .item-delete-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    color: inherit;
    opacity: 0.6;
    transition: opacity 0.15s;
    padding: 0 4px;
  }

  .item-delete-btn:hover {
    opacity: 1;
  }

  .editor-preview-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .editor-card {
    padding: 24px;
  }

  .bilingual-editors {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-top: 10px;
  }

  .editor-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .lang-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-sec);
    text-transform: uppercase;
  }

  .rtl-direction {
    direction: rtl;
    text-align: right;
  }

  .ltr-direction {
    direction: ltr;
    text-align: left;
  }

  .variable-chips {
    display: flex;
    gap: 8px;
  }

  .chip {
    background: var(--teal-light);
    color: var(--teal);
    border: 1px solid var(--teal-mid);
    border-radius: 999px;
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    font-family: var(--font);
    transition: all 0.15s;
  }

  .chip:hover {
    background: var(--teal);
    color: #fff;
  }

  /* ── WhatsApp Mockup Styles ── */
  .preview-card {
    padding: 0;
    overflow: hidden;
    background: #efeae2; /* WhatsApp chat background */
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .whatsapp-hdr {
    background: #075e54;
    color: #fff;
    padding: 10px 16px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .wa-avatar {
    width: 32px;
    height: 32px;
    background: rgba(255,255,255,0.2);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
  }

  .wa-name {
    font-weight: 700;
    font-size: 13.5px;
  }

  .wa-status {
    font-size: 11px;
    opacity: 0.8;
  }

  .whatsapp-body {
    padding: 20px;
    min-height: 160px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: flex-end;
  }

  .wa-message-bubble {
    background: #d9fdd3; /* WhatsApp sent bubble color */
    border-radius: 8px;
    padding: 10px 14px;
    max-width: 80%;
    box-shadow: 0 1px 1px rgba(0,0,0,0.1);
    position: relative;
  }

  .wa-message-bubble p {
    color: #303030;
    font-size: 13px;
    margin: 0;
    white-space: pre-wrap;
    text-align: var(--text-align);
  }

  .wa-time {
    display: block;
    text-align: end;
    font-size: 9.5px;
    color: #667781;
    margin-top: 4px;
  }

  @media (max-width: 900px) {
    .templates-layout {
      grid-template-columns: 1fr;
    }
    .templates-sidebar {
      height: 250px;
    }
  }
</style>
