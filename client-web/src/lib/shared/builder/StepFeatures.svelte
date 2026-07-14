<script lang="ts">
  import { Plus, ArrowUp, ArrowDown, Copy, Trash2, Link as LinkIcon, Globe } from 'lucide-svelte';
  import type { ClientBuilderState } from '../../components/listing/clientBuilderState.svelte.js';
  import { BLOCK_TYPES, getBlockDef } from './builder.constants.js';
  import { getI18n } from "$lib/i18n/i18n.svelte";
  import { vendorStore } from "$lib/stores/vendorStore.js";

  let { state }: { state: ClientBuilderState } = $props();
  const i18n = getI18n();
  let currentLang = $derived(i18n.locale);

  let isDiamond = $derived($vendorStore.tier_id === "diamond");
  let maxBlocks = $derived($vendorStore.policy_limits?.description_blocks ?? 5);

  const featureTypes = BLOCK_TYPES.filter(b => ['heading', 'subheading', 'text', 'list', 'button', 'contact', 'social', 'divider'].includes(b.id));

  function handleAdd(type: string, index: number) {
      state.addBlock(type, false, index, maxBlocks, isDiamond, currentLang);
  }
</script>

<div class="step-container animate-slide-in">
  <div class="builder-header-banner">
      <div class="header-left">
          <span class="step-icon-badge">✍️</span>
          <div>
              <h3>{currentLang === "ar" ? "تفاصيل ومميزات الإعلان" : "Listing Features & Text"}</h3>
              <p class="subtitle-desc">{currentLang === "ar" ? "أضف النصوص، العناوين والقوائم لتوضيح مميزاتك" : "Add text, headings, and lists to describe your features"}</p>
          </div>
      </div>
      <div class="lang-requirement-badge">
          <Globe size={13} />
          <span>{currentLang === "ar" ? "يدعم تعدد اللغات" : "Multi-language supported"}</span>
      </div>
  </div>

  <div class="blocks-stack">
      <!-- Top Add Button -->
      {#if state.featureBlocks.length > 0}
          <div class="inline-add-zone" class:active={state.showAddMenuAtIndex?.index === 0 && state.showAddMenuAtIndex?.type === 'feature'}>
              <button type="button" class="inline-add-btn" onclick={() => state.showAddMenuAtIndex = state.showAddMenuAtIndex?.index === 0 ? null : {type: 'feature', index: 0}}>
                  <Plus size={16} />
              </button>
              {#if state.showAddMenuAtIndex?.index === 0 && state.showAddMenuAtIndex?.type === 'feature'}
                  <div class="inline-add-menu">
                      {#each featureTypes as bt}
                          {@const Icon = bt.icon}
                          <button type="button" class="menu-item-btn" onclick={() => handleAdd(bt.id, 0)}>
                              <Icon size={14} />
                              <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                          </button>
                      {/each}
                  </div>
              {/if}
          </div>
      {/if}

      {#each state.featureBlocks as block, i}
          {@const DefIcon = getBlockDef(block.type).icon}
          <div class="block-card-wrapper animate-slide-in">
              <div class="block-card-head">
                  <span class="block-index-label">
                      <DefIcon size={16} />
                      {currentLang === "ar" ? getBlockDef(block.type).labelAr : getBlockDef(block.type).labelEn}
                  </span>
                  <div class="block-actions">
                      <button type="button" class="action-btn" onclick={() => state.moveUp(false, i)} disabled={i === 0}><ArrowUp size={14} /></button>
                      <button type="button" class="action-btn" onclick={() => state.moveDown(false, i)} disabled={i === state.featureBlocks.length - 1}><ArrowDown size={14} /></button>
                      <button type="button" class="action-btn" onclick={() => state.duplicateBlock(false, i, maxBlocks, isDiamond, currentLang)}><Copy size={14} /></button>
                      <button type="button" class="action-btn delete-btn" onclick={() => state.removeBlock(false, i)}><Trash2 size={14} /></button>
                  </div>
              </div>

              <!-- Content Areas -->
              <div class="block-card-inputs grid-side-by-side">
                  {#if block.type !== 'divider'}
                      <div class="lang-input-pane ltr-pane">
                          <div class="pane-header">
                              <span class="lang-flag-indicator">🇬🇧</span>
                              <span class="pane-title">English</span>
                          </div>
                          {#if block.type === "button"}
                              <input type="text" class="pane-text-input mb-2" bind:value={block.contentEn} oninput={() => state.sync()} placeholder="Button Label" />
                              <div class="pane-url-input">
                                  <LinkIcon size={14} class="pane-icon" />
                                  <input type="url" bind:value={block.url} oninput={() => state.sync()} placeholder="Button URL" />
                              </div>
                          {:else}
                              <textarea class="pane-textarea" bind:value={block.contentEn} oninput={() => state.sync()} placeholder="Write in English..." rows="3"></textarea>
                          {/if}
                      </div>

                      <div class="lang-input-pane rtl-pane">
                          <div class="pane-header">
                              <span class="lang-flag-indicator">🇸🇦</span>
                              <span class="pane-title">العربية</span>
                          </div>
                          {#if block.type === "button"}
                              <input type="text" class="pane-text-input mb-2" bind:value={block.contentAr} oninput={() => state.sync()} placeholder="نص الزر" />
                              <div class="pane-url-input">
                                  <LinkIcon size={14} class="pane-icon" />
                                  <input type="url" bind:value={block.url} oninput={() => state.sync()} placeholder="رابط الزر" />
                              </div>
                          {:else}
                              <textarea class="pane-textarea" bind:value={block.contentAr} oninput={() => state.sync()} placeholder="اكتب بالعربية..." rows="3"></textarea>
                          {/if}
                      </div>
                  {:else}
                      <div class="divider-pane" style="grid-column: 1 / -1;">
                          <hr class="preview-divider" />
                      </div>
                  {/if}
              </div>
          </div>

          <div class="inline-add-zone" class:active={state.showAddMenuAtIndex?.index === i + 1 && state.showAddMenuAtIndex?.type === 'feature'}>
              <button type="button" class="inline-add-btn" onclick={() => state.showAddMenuAtIndex = state.showAddMenuAtIndex?.index === i + 1 ? null : {type: 'feature', index: i + 1}}>
                  <Plus size={16} />
              </button>
              {#if state.showAddMenuAtIndex?.index === i + 1 && state.showAddMenuAtIndex?.type === 'feature'}
                  <div class="inline-add-menu">
                      {#each featureTypes as bt}
                          {@const Icon = bt.icon}
                          <button type="button" class="menu-item-btn" onclick={() => handleAdd(bt.id, i + 1)}>
                              <Icon size={14} />
                              <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                          </button>
                      {/each}
                  </div>
              {/if}
          </div>
      {/each}

      {#if state.featureBlocks.length === 0}
          <div class="empty-blocks-state">
              <p class="empty-txt">{currentLang === "ar" ? "لا يوجد مميزات مضافة بعد." : "No features added yet."}</p>
              <div class="add-actions-group center-actions mt-4">
                  {#each featureTypes as bt}
                      {@const Icon = bt.icon}
                      <button type="button" class="btn-add-block" onclick={() => handleAdd(bt.id, 0)}>
                          <Icon size={14} />
                          <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                      </button>
                  {/each}
              </div>
          </div>
      {/if}
  </div>
</div>

<style>
  .step-container { animation: slideIn 300ms ease forwards; }
  @keyframes slideIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

  .builder-header-banner {
      display: flex; align-items: center; justify-content: space-between;
      background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 12px;
      padding: 16px 20px; gap: 12px; margin-bottom: 24px;
  }
  .header-left { display: flex; align-items: center; gap: 12px; }
  .step-icon-badge { font-size: 24px; }
  .header-left h3 { margin: 0; font-size: 1.1rem; font-weight: 700; color: #0f172a; }
  .subtitle-desc { margin: 2px 0 0 0; font-size: 0.85rem; color: #64748b; }

  /* Reused classes from UnifiedDescriptionBuilder */
  .blocks-stack { display: flex; flex-direction: column; }
  .inline-add-zone { position: relative; height: 24px; display: flex; align-items: center; justify-content: center; margin: 8px 0; }
  .inline-add-btn { background: #fff; border: 1px solid #e2e8f0; border-radius: 50%; width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; color: #94a3b8; cursor: pointer; z-index: 2; transition: all 0.2s; }
  .inline-add-btn:hover { background: #6366f1; color: #fff; border-color: #6366f1; }
  .inline-add-zone::before { content: ''; position: absolute; left: 0; right: 0; top: 50%; height: 1px; background: #e2e8f0; z-index: 1; }
  
  .inline-add-menu { position: absolute; top: 100%; left: 50%; transform: translateX(-50%); background: #fff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 8px; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.1); z-index: 10; display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; width: max-content; }
  .menu-item-btn { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: none; background: transparent; cursor: pointer; border-radius: 6px; font-size: 13px; color: #475569; transition: all 0.2s; }
  .menu-item-btn:hover { background: #f8fafc; color: #0f172a; }

  .block-card-wrapper { background: #fff; border: 1px solid #e2e8f0; border-radius: 12px; margin-bottom: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.02); }
  .block-card-head { display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid #f1f5f9; background: #f8fafc; border-radius: 12px 12px 0 0; }
  .block-index-label { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 600; color: #475569; }
  .block-actions { display: flex; gap: 4px; }
  .action-btn { background: transparent; border: none; padding: 6px; cursor: pointer; border-radius: 6px; color: #64748b; transition: all 0.2s; }
  .action-btn:hover:not(:disabled) { background: #f1f5f9; color: #0f172a; }
  .action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .delete-btn:hover { background: #fef2f2 !important; color: #ef4444 !important; }

  .block-card-inputs { padding: 16px; }
  .grid-side-by-side { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
  
  .lang-input-pane { display: flex; flex-direction: column; gap: 12px; }
  .pane-header { display: flex; align-items: center; gap: 8px; }
  .pane-title { font-size: 13px; font-weight: 600; color: #475569; }
  
  .pane-textarea, .pane-text-input { width: 100%; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px; font-size: 14px; color: #1e293b; transition: all 0.2s; }
  .pane-textarea:focus, .pane-text-input:focus { outline: none; border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,0.1); }
  
  .pane-url-input { display: flex; align-items: center; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; background: #fff; }
  .pane-url-input .pane-icon { margin: 0 12px; color: #94a3b8; }
  .pane-url-input input { border: none; padding: 10px 10px 10px 0; width: 100%; font-size: 14px; outline: none; }
  
  .empty-blocks-state { text-align: center; padding: 48px 20px; background: #f8fafc; border: 1px dashed #cbd5e1; border-radius: 12px; }
  .empty-txt { color: #64748b; font-size: 14px; margin-bottom: 20px; }
  .add-actions-group { display: flex; flex-wrap: wrap; justify-content: center; gap: 8px; }
  .btn-add-block { display: flex; align-items: center; gap: 8px; padding: 8px 16px; background: #fff; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 13px; font-weight: 500; color: #475569; cursor: pointer; transition: all 0.2s; }
  .btn-add-block:hover { background: #f8fafc; border-color: #cbd5e1; color: #0f172a; }

  @media (max-width: 768px) {
      .grid-side-by-side { grid-template-columns: 1fr; }
  }
</style>
