<script lang="ts">
  import { Plus, ArrowUp, ArrowDown, Copy, Trash2, X, MapPin, Image as ImageIcon, Globe } from 'lucide-svelte';
  import type { ClientBuilderState } from '../../components/listing/clientBuilderState.svelte.js';
  import { BLOCK_TYPES, getBlockDef } from './builder.constants.js';
  import { getI18n } from "$lib/i18n/i18n.svelte";
  import { vendorStore } from "$lib/stores/vendorStore.js";
  import { resolveMediaUrl } from "$lib/shared/utils/media.js";

  let { state }: { state: ClientBuilderState } = $props();
  const i18n = getI18n();
  let currentLang = $derived(i18n.locale);

  let isDiamond = $derived($vendorStore.tier_id === "diamond");
  let maxBlocks = $derived($vendorStore.policy_limits?.description_blocks ?? 5);

  const mediaTypes = BLOCK_TYPES.filter(b => ['image', 'gallery', 'video', 'map', 'image_text'].includes(b.id));

  function handleAdd(type: string, index: number) {
      state.addBlock(type, true, index, maxBlocks, isDiamond, currentLang);
  }

  function getEmbedUrl(url: string) {
      if (!url) return "";
      if (url.includes("<iframe")) {
          const match = url.match(/src=["']([^"']+)["']/);
          if (match && match[1]) return match[1];
      }
      return url;
  }

  function getVideoEmbedUrl(url: string): string | null {
      if (!url) return null;
      if (url.includes("<iframe")) {
          const match = url.match(/src=["']([^"']+)["']/);
          if (match && match[1]) url = match[1];
      }
      const ytMatch = url.match(/^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|\&v=|shorts\/)([^#\&\?]*).*/);
      if (ytMatch && ytMatch[2].length === 11) return `https://www.youtube.com/embed/${ytMatch[2]}`;
      
      const vimeoMatch = url.match(/^.*(vimeo\.com\/|video\/)(clip\/)?([0-9]+).*/);
      if (vimeoMatch) return `https://player.vimeo.com/video/${vimeoMatch[3]}`;

      return null;
  }
</script>

<div class="step-container animate-slide-in">
  <div class="builder-header-banner">
      <div class="header-left">
          <span class="step-icon-badge">📸</span>
          <div>
              <h3>{currentLang === "ar" ? "وسائط الإعلان" : "Listing Media"}</h3>
              <p class="subtitle-desc">{currentLang === "ar" ? "أضف الصور، الفيديوهات والخريطة" : "Add images, videos, and map locations"}</p>
          </div>
      </div>
  </div>

  <div class="blocks-stack">
      <!-- Top Add Button -->
      {#if state.mediaBlocks.length > 0}
          <div class="inline-add-zone" class:active={state.showAddMenuAtIndex?.index === 0 && state.showAddMenuAtIndex?.type === 'media'}>
              <button type="button" class="inline-add-btn" onclick={() => state.showAddMenuAtIndex = state.showAddMenuAtIndex?.index === 0 ? null : {type: 'media', index: 0}}>
                  <Plus size={16} />
              </button>
              {#if state.showAddMenuAtIndex?.index === 0 && state.showAddMenuAtIndex?.type === 'media'}
                  <div class="inline-add-menu">
                      {#each mediaTypes as bt}
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

      {#each state.mediaBlocks as block, i}
          {@const DefIcon = getBlockDef(block.type).icon}
          <div class="block-card-wrapper animate-slide-in">
              <div class="block-card-head">
                  <span class="block-index-label">
                      <DefIcon size={16} />
                      {currentLang === "ar" ? getBlockDef(block.type).labelAr : getBlockDef(block.type).labelEn}
                  </span>
                  <div class="block-actions">
                      <button type="button" class="action-btn" onclick={() => state.moveUp(true, i)} disabled={i === 0}><ArrowUp size={14} /></button>
                      <button type="button" class="action-btn" onclick={() => state.moveDown(true, i)} disabled={i === state.mediaBlocks.length - 1}><ArrowDown size={14} /></button>
                      <button type="button" class="action-btn" onclick={() => state.duplicateBlock(true, i, maxBlocks, isDiamond, currentLang)}><Copy size={14} /></button>
                      <button type="button" class="action-btn delete-btn" onclick={() => state.removeBlock(true, i)}><Trash2 size={14} /></button>
                  </div>
              </div>

              <!-- Media Input Areas -->
              <div class="block-card-inputs" class:grid-side-by-side={block.type !== "image_text"}>
                  {#if block.type === "image_text"}
                      <div class="lang-input-pane" style="width: 100%;">
                          <div class="flex items-center gap-4 mb-4">
                              <span class="font-bold text-sm text-slate-700">{currentLang === 'ar' ? 'تخطيط الصورة' : 'Image Layout'}:</span>
                              <label class="flex items-center gap-2 cursor-pointer text-sm">
                                  <input type="radio" name="layout_{i}" value="left" bind:group={block.layout} onchange={() => state.sync()} class="text-emerald-600" />
                                  {currentLang === 'ar' ? 'الصورة يسار' : 'Image Left'}
                              </label>
                              <label class="flex items-center gap-2 cursor-pointer text-sm">
                                  <input type="radio" name="layout_{i}" value="right" bind:group={block.layout} onchange={() => state.sync()} class="text-emerald-600" />
                                  {currentLang === 'ar' ? 'الصورة يمين' : 'Image Right'}
                              </label>
                          </div>
                          
                          <div class="pane-url-input mb-6">
                              <ImageIcon size={14} class="pane-icon" />
                              <input type="url" bind:value={block.url} oninput={() => state.sync()} placeholder={currentLang === "ar" ? "أدخل رابط الصورة (URL)..." : "Enter Image URL..."} />
                          </div>

                          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                              <div>
                                  <div class="pane-header mb-2"><span class="lang-flag-indicator">🇬🇧</span> <span class="pane-title">English Text</span></div>
                                  <textarea class="pane-textarea" bind:value={block.contentEn} oninput={() => state.sync()} rows="6" placeholder="English Content..."></textarea>
                              </div>
                              <div dir="rtl">
                                  <div class="pane-header mb-2"><span class="lang-flag-indicator">🇸🇦</span> <span class="pane-title">النص العربي</span></div>
                                  <textarea class="pane-textarea" bind:value={block.contentAr} oninput={() => state.sync()} rows="6" placeholder="النص العربي..."></textarea>
                              </div>
                          </div>
                      </div>
                  {:else}
                      <!-- Media Left: Input Options -->
                      <div class="lang-input-pane media-input-pane">
                          <div class="pane-header">
                              <span class="pane-title">{currentLang === "ar" ? "إضافة محتوى" : "Add Content"}</span>
                          </div>
                          <div class="pane-url-input">
                              <DefIcon size={14} class="pane-icon" />
                              <input type="url" bind:value={block.url} oninput={() => state.sync()} placeholder={currentLang === "ar" ? "أدخل الرابط (URL)..." : "Enter URL..."} />
                          </div>
                          <p class="media-help-text">
                              {currentLang === "ar" ? "أدخل رابط الصورة، الفيديو، أو الخريطة وسيظهر العرض الجانبي تلقائياً." : "Enter the URL for the image, video, or map and the preview will appear automatically."}
                          </p>
                      </div>
                      
                      <!-- Media Right: Preview -->
                      <div class="lang-input-pane media-preview-pane">
                          {#if block.url}
                              <div class="media-preview-container">
                                  <button class="clear-media-btn" onclick={() => state.clearMedia(i)} title={currentLang === "ar" ? "إزالة المحتوى" : "Clear Content"}><X size={14} /></button>
                                  {#if block.type === "image" || block.type === "gallery"}
                                      <img src={resolveMediaUrl(block.url)} alt="Preview" class="preview-img" />
                                  {:else if block.type === "video"}
                                      {@const embedUrl = getVideoEmbedUrl(block.url)}
                                      {#if embedUrl}
                                          <iframe src={embedUrl} title="Preview Video" class="w-full h-full border-0 aspect-video" allowfullscreen></iframe>
                                      {:else}
                                          <!-- svelte-ignore a11y_media_has_caption -->
                                          <video src={resolveMediaUrl(block.url)} controls playsinline preload="metadata" class="w-full h-full max-h-[200px] object-contain rounded-lg"></video>
                                      {/if}
                                  {:else if block.type === "map"}
                                      <div class="preview-map-placeholder">
                                          <MapPin size={32} />
                                          <span>Map Preview</span>
                                      </div>
                                  {/if}
                              </div>
                          {:else}
                              <div class="empty-preview">
                                  <DefIcon size={24} />
                                  <span>{currentLang === "ar" ? "لا يوجد محتوى للعرض" : "No content to preview"}</span>
                              </div>
                          {/if}
                      </div>
                  {/if}
              </div>
          </div>

          <div class="inline-add-zone" class:active={state.showAddMenuAtIndex?.index === i + 1 && state.showAddMenuAtIndex?.type === 'media'}>
              <button type="button" class="inline-add-btn" onclick={() => state.showAddMenuAtIndex = state.showAddMenuAtIndex?.index === i + 1 ? null : {type: 'media', index: i + 1}}>
                  <Plus size={16} />
              </button>
              {#if state.showAddMenuAtIndex?.index === i + 1 && state.showAddMenuAtIndex?.type === 'media'}
                  <div class="inline-add-menu">
                      {#each mediaTypes as bt}
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

      {#if state.mediaBlocks.length === 0}
          <div class="empty-blocks-state">
              <p class="empty-txt">{currentLang === "ar" ? "لا يوجد وسائط مضافة بعد." : "No media added yet."}</p>
              <div class="add-actions-group center-actions mt-4">
                  {#each mediaTypes as bt}
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
  /* Base styles inherited or matched with StepFeatures */
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

  /* Reused blocks styles */
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
  
  .pane-url-input { display: flex; align-items: center; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; background: #fff; }
  .pane-url-input .pane-icon { margin: 0 12px; color: #94a3b8; }
  .pane-url-input input { border: none; padding: 10px 10px 10px 0; width: 100%; font-size: 14px; outline: none; }
  
  .pane-textarea { width: 100%; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px; font-size: 14px; color: #1e293b; transition: all 0.2s; }
  .pane-textarea:focus { outline: none; border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,0.1); }

  .empty-blocks-state { text-align: center; padding: 48px 20px; background: #f8fafc; border: 1px dashed #cbd5e1; border-radius: 12px; }
  .empty-txt { color: #64748b; font-size: 14px; margin-bottom: 20px; }
  .add-actions-group { display: flex; flex-wrap: wrap; justify-content: center; gap: 8px; }
  .btn-add-block { display: flex; align-items: center; gap: 8px; padding: 8px 16px; background: #fff; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 13px; font-weight: 500; color: #475569; cursor: pointer; transition: all 0.2s; }
  .btn-add-block:hover { background: #f8fafc; border-color: #cbd5e1; color: #0f172a; }

  /* Media Preview Styles */
  .media-input-pane { justify-content: center; }
  .media-help-text { font-size: 13px; color: #64748b; line-height: 1.5; margin: 0; }
  .media-preview-pane { display: flex; align-items: stretch; }
  .media-preview-container { position: relative; width: 100%; min-height: 120px; border-radius: 8px; overflow: hidden; background: #f1f5f9; display: flex; align-items: center; justify-content: center; }
  .preview-img { width: 100%; height: 100%; object-fit: contain; max-height: 200px; }
  .clear-media-btn { position: absolute; top: 8px; right: 8px; background: rgba(0,0,0,0.5); color: #fff; border: none; width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: pointer; z-index: 10; transition: all 0.2s; }
  .clear-media-btn:hover { background: #ef4444; }
  .empty-preview { width: 100%; height: 120px; border: 1px dashed #cbd5e1; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: #94a3b8; font-size: 13px; background: #f8fafc; }
  .preview-map-placeholder { display: flex; flex-direction: column; align-items: center; gap: 8px; color: #64748b; }

  @media (max-width: 768px) {
      .grid-side-by-side { grid-template-columns: 1fr; }
  }
</style>
