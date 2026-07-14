<script lang="ts">
  import '../app.css';
  import { lang, dir } from '$lib/i18n/index.js';
  import { onMount } from 'svelte';
  import { navigating } from '$app/stores';
  import { ui } from '$lib/stores/ui.store.js';
  import Loading from '$lib/components/Loading.svelte';

  let { children } = $props();

  onMount(() => {
    // Apply stored language direction on first mount
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem('zafaf_lang') as 'ar' | 'en' | null;
      if (stored && stored !== 'ar') {
        lang.set(stored);
      }
    }
  });
</script>

<svelte:head>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<Loading show={!!$navigating || $ui.globalLoading} />

{@render children()}
