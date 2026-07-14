<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { MapPin, Phone, Share2 } from 'lucide-svelte';
  import type { BuilderState } from '../../../features/admin/builder/builderState.svelte.js';
  import { getI18n } from "$lib/i18n/i18n.svelte";
  import { resolveMediaUrl } from "$lib/shared/utils/media.js";

  let { state }: { state: BuilderState } = $props();
  const i18n = getI18n();
  let currentLang = $derived(i18n.locale);

  let allBlocks = $derived([...state.featureBlocks, ...state.mediaBlocks]);

  function getGalleryUrls(url: string) {
      if (!url) return [];
      return url.split(/[\n,]/).map(u => u.trim()).filter(Boolean);
  }

  function getEmbedUrl(url: string) {
      if (!url) return "";
      if (url.includes("<iframe")) {
          const match = url.match(/src=["']([^"']+)["']/);
          if (match && match[1]) return match[1];
      }
      return url;
  }

  function isEmbeddableUrl(url: string) {
      if (!url) return false;
      return url.includes("google.com/maps/embed") || url.includes("google.com/maps/d/embed") || url.includes("openstreetmap.org/export/embed");
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
          <span class="step-icon-badge">👀</span>
          <div>
              <h3>{currentLang === "ar" ? "معاينة الإعلان" : "Listing Preview"}</h3>
              <p class="subtitle-desc">{currentLang === "ar" ? "هكذا سيبدو إعلانك للزوار" : "This is how your listing will appear to visitors"}</p>
          </div>
      </div>
  </div>

  <div class="prose max-w-none text-slate-800 space-y-6 text-start bg-white p-8 rounded-2xl border border-slate-200">
      {#each allBlocks as block}
          {@const content =
              (currentLang === "ar" ? block.contentAr : block.contentEn) ||
              (currentLang === "ar" ? block.contentEn : block.contentAr) ||
              ""}
          {#if block.type === "heading" && content}
              <h2 class="text-3xl font-extrabold text-slate-900 tracking-tight mt-8 mb-4">{content}</h2>
          {:else if block.type === "subheading" && content}
              <h3 class="text-2xl font-bold text-slate-800 tracking-tight mt-6 mb-3">{content}</h3>
          {:else if block.type === "text" && content}
              <p class="text-base text-slate-600 leading-relaxed whitespace-pre-wrap">{content}</p>
          {:else if block.type === "list" && content}
              <ul class="list-disc pl-5 rtl:pl-0 rtl:pr-5 space-y-2 my-4">
                  {#each content.split("\n") as item}
                      {#if item.trim()}
                          <li class="text-base text-slate-600">{item}</li>
                      {/if}
                  {/each}
              </ul>
          {:else if block.type === "image" && block.url}
              <img src={resolveMediaUrl(block.url)} alt="" loading="lazy" class="rounded-2xl max-w-full h-auto my-6 shadow-sm border border-gray-100/80" />
          {:else if block.type === "gallery" && block.url}
              {@const urls = getGalleryUrls(block.url)}
              {#if urls.length > 1}
                  <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 my-6">
                      {#each urls as imageUrl}
                          <div class="relative aspect-video rounded-2xl overflow-hidden shadow-sm group/gallery border border-gray-100">
                              <img src={resolveMediaUrl(imageUrl)} alt="" loading="lazy" class="w-full h-full object-cover transition-transform duration-500 group-hover/gallery:scale-105" />
                          </div>
                      {/each}
                  </div>
              {:else if urls.length === 1}
                  <img src={resolveMediaUrl(urls[0])} alt="" loading="lazy" class="rounded-2xl max-w-full h-auto my-6 shadow-sm border border-gray-100/80" />
              {/if}
          {:else if block.type === "video" && block.url}
              {@const embedUrl = getVideoEmbedUrl(block.url)}
              {#if embedUrl}
                  <div class="my-6 rounded-2xl overflow-hidden border border-gray-100 shadow-sm aspect-video w-full bg-slate-100">
                      <iframe src={embedUrl} title="Video" class="w-full h-full border-0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen loading="lazy"></iframe>
                  </div>
              {:else}
                  <!-- svelte-ignore a11y_media_has_caption -->
                  <video src={resolveMediaUrl(block.url)} controls playsinline preload="metadata" class="w-full rounded-2xl my-6 shadow-sm border border-gray-100"></video>
              {/if}
          {:else if block.type === "map" && block.url}
              {@const embedUrl = getEmbedUrl(block.url)}
              {#if isEmbeddableUrl(embedUrl)}
                  <div class="my-6 rounded-2xl overflow-hidden border border-gray-100 shadow-sm aspect-video w-full bg-slate-100">
                      <iframe src={embedUrl} title="Map" class="w-full h-full border-0" allowfullscreen loading="lazy"></iframe>
                  </div>
              {:else}
                  <div class="my-6 p-6 bg-slate-50 border border-slate-200/60 rounded-2xl flex flex-col sm:flex-row items-center justify-between gap-4 shadow-sm">
                      <div class="flex items-center gap-4 text-start w-full sm:w-auto">
                          <div class="p-3 bg-red-50 text-red-500 rounded-xl shrink-0"><MapPin size={24} /></div>
                          <div class="flex-1">
                              <h4 class="font-semibold text-slate-800">{currentLang === 'ar' ? 'موقع الإعلان على الخريطة' : 'Listing Location on Map'}</h4>
                          </div>
                      </div>
                      <a href={block.url} target="_blank" rel="noopener noreferrer" class="w-full sm:w-auto text-center bg-slate-900 text-white px-5 py-2.5 rounded-lg font-medium hover:bg-slate-800 transition-colors text-sm shadow-sm">{currentLang === 'ar' ? 'عرض على الخريطة' : 'View on Map'}</a>
                  </div>
              {/if}
          {:else if block.type === "contact" && content}
              <div class="my-6 p-6 bg-emerald-50/40 border border-emerald-100/60 rounded-2xl flex items-start gap-4 shadow-sm text-start">
                  <div class="p-3 bg-emerald-100/80 text-emerald-600 rounded-xl mt-1 shrink-0"><Phone size={20} /></div>
                  <div class="flex-1">
                      <h4 class="font-semibold text-slate-800 mb-1">{currentLang === 'ar' ? 'تفاصيل الاتصال' : 'Contact Details'}</h4>
                      <p class="text-slate-600 whitespace-pre-wrap leading-relaxed text-sm">{content}</p>
                  </div>
              </div>
          {:else if block.type === "social" && content}
              <div class="my-6 p-6 bg-blue-50/40 border border-blue-100/60 rounded-2xl flex items-start gap-4 shadow-sm text-start">
                  <div class="p-3 bg-blue-100/80 text-blue-600 rounded-xl mt-1 shrink-0"><Share2 size={20} /></div>
                  <div class="flex-1">
                      <h4 class="font-semibold text-slate-800 mb-1">{currentLang === 'ar' ? 'قنوات التواصل الاجتماعي' : 'Social Media Channels'}</h4>
                      <p class="text-slate-600 whitespace-pre-wrap leading-relaxed text-sm">{content}</p>
                  </div>
              </div>
          {:else if block.type === "divider"}
              <hr class="my-8 border-gray-200" />
          {:else if block.type === "image_text"}
              <div class="flex flex-col sm:flex-row gap-6 my-8 items-start bg-slate-50/50 p-6 rounded-3xl border border-slate-100">
                  {#if block.layout === 'right'}
                      <div class="flex-1 text-slate-700 leading-relaxed text-sm whitespace-pre-wrap order-2 sm:order-1">{content}</div>
                      {#if block.url}
                          <div class="w-full sm:w-1/2 rounded-2xl overflow-hidden shadow-sm border border-slate-100 order-1 sm:order-2 shrink-0">
                              <img src={block.url} alt="" class="w-full h-full object-cover" />
                          </div>
                      {/if}
                  {:else}
                      {#if block.url}
                          <div class="w-full sm:w-1/2 rounded-2xl overflow-hidden shadow-sm border border-slate-100 shrink-0">
                              <img src={block.url} alt="" class="w-full h-full object-cover" />
                          </div>
                      {/if}
                      <div class="flex-1 text-slate-700 leading-relaxed text-sm whitespace-pre-wrap">{content}</div>
                  {/if}
              </div>
          {:else if block.type === "button" && block.url && content}
              <a href={block.url} target="_blank" rel="noopener noreferrer" class="inline-block bg-slate-900 text-white px-6 py-3 rounded-lg font-medium hover:bg-slate-800 transition-colors my-4">{content}</a>
          {/if}
      {/each}
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
</style>
