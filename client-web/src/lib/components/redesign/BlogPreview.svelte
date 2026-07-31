<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import Button from "./Button.svelte";
  import Card from "./Card.svelte";
  import type { BlogPost } from "$lib/lib/blog/BlogRepository";

  interface Props {
    posts?: BlogPost[];
    loading?: boolean;
  }

  let { posts = [], loading = false }: Props = $props();

  const isRtl = $derived(getLocale() === "ar");
</script>

<section
  class="py-zw-16 md:py-zw-24 bg-zw-surface relative overflow-hidden"
  aria-labelledby="blog-preview-title"
>
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-zw-6 mb-zw-12">
      <div class="text-start">
        <span class="text-zw-primary text-zw-xs font-bold tracking-widest uppercase mb-zw-2 block">
          {isRtl ? "مجلة زفاف وورلد" : "ZafafWorld Magazine"}
        </span>
        <h2 id="blog-preview-title" class="font-display text-zw-3xl sm:text-zw-4xl font-bold text-zw-secondary leading-tight">
          {isRtl ? "أحدث النصائح والأفكار لزفافك" : "Latest Wedding Inspiration"}
        </h2>
        <p class="text-zw-muted text-zw-sm max-w-xl mt-zw-2 leading-relaxed">
          {isRtl
            ? "نصائح الخبراء، أحدث صيحات الموضة، وأفكار مبتكرة لمساعدتك في التخطيط لليلة العمر بكل سهولة."
            : "Expert guides, style trends, and creative ideas curated by wedding professionals to help you plan effortlessly."}
        </p>
      </div>

      <div class="text-start">
        <Button href="/discover" variant="outline" size="md" class="font-bold">
          <span>{isRtl ? "تصفح جميع المقالات" : "Browse All Articles"}</span>
          <span class="ms-zw-2 rtl:rotate-180" aria-hidden="true">→</span>
        </Button>
      </div>
    </div>

    <!-- Cards Grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-zw-6">
      {#if loading || posts.length === 0}
        <!-- Skeletons (EXACTLY matching the card height/layout to prevent CLS) -->
        {#each Array(3) as _}
          <div
            class="bg-zw-surface border border-zw-border rounded-zw-2xl overflow-hidden shadow-zw-sm animate-pulse h-[460px] flex flex-col justify-between p-zw-4"
            aria-hidden="true"
          >
            <div class="w-full aspect-[16/10] bg-zw-border/30 rounded-zw-xl animate-pulse"></div>
            <div class="flex flex-col gap-zw-3 mt-zw-4 flex-1">
              <div class="w-1/3 h-3 bg-zw-border/30 rounded animate-pulse"></div>
              <div class="w-full h-5 bg-zw-border/30 rounded animate-pulse mt-zw-2"></div>
              <div class="w-5/6 h-5 bg-zw-border/30 rounded animate-pulse"></div>
              <div class="w-full h-3 bg-zw-border/20 rounded animate-pulse mt-zw-3"></div>
              <div class="w-2/3 h-3 bg-zw-border/20 rounded animate-pulse"></div>
            </div>
            <div class="w-1/4 h-4 bg-zw-border/30 rounded animate-pulse mt-zw-4"></div>
          </div>
        {/each}
      {:else}
        <!-- Real Blog Cards -->
        {#each posts as post}
          <article
            class="bg-zw-surface border border-zw-border hover:border-zw-border-hover rounded-zw-2xl overflow-hidden shadow-zw-sm hover:shadow-zw-md transition-all duration-300 flex flex-col h-full justify-between"
          >
            <div class="p-zw-4">
              <!-- Article cover image with intrinsic sizes -->
              <div class="relative w-full aspect-[16/10] rounded-zw-xl overflow-hidden bg-zw-secondary/15 group">
                <img
                  src={post.cover_image}
                  alt={post.title}
                  class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                  loading="lazy"
                  decoding="async"
                  sizes="(max-width: 768px) 100vw, 33vw"
                  width="400"
                  height="250"
                />
              </div>

              <!-- Metadata row -->
              <div class="flex flex-wrap items-center gap-zw-2 text-[10px] text-zw-muted font-bold uppercase tracking-wider mt-zw-4 mb-zw-3 text-start">
                <span class="text-zw-primary-contrast">{isRtl ? post.category.ar : post.category.en}</span>
                <span>•</span>
                <span>{isRtl ? post.reading_time.ar : post.reading_time.en}</span>
                <span>•</span>
                <span>{isRtl ? post.publish_date.ar : post.publish_date.en}</span>
              </div>

              <!-- Title & excerpt -->
              <div class="text-start">
                <h3 class="font-display text-zw-lg font-bold text-zw-secondary line-clamp-2 hover:text-zw-primary transition-colors mb-zw-2 leading-tight">
                  <a href={`/discover/${post.slug}`}>
                    {post.title}
                  </a>
                </h3>
                <p class="text-zw-muted text-zw-xs line-clamp-3 leading-relaxed">
                  {post.excerpt}
                </p>
              </div>
            </div>

            <!-- Bottom CTA Link -->
            <div class="px-zw-4 pb-zw-5 pt-zw-2 text-start">
              <a
                href={`/discover/${post.slug}`}
                class="inline-flex items-center text-zw-primary-contrast hover:text-zw-primary text-zw-xs font-bold transition-colors group/link"
              >
                <span>{isRtl ? "اقرأ المقال الكامل" : "Read Full Article"}</span>
                <span class="ms-zw-1.5 transition-transform group-hover/link:translate-x-1 rtl:group-hover/link:-translate-x-1 rtl:rotate-180" aria-hidden="true">→</span>
              </a>
            </div>
          </article>
        {/each}
      {/if}
    </div>
  </div>
</section>
