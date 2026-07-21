<script lang="ts">
  import { navigating } from "$app/stores";
  import { onDestroy } from "svelte";

  let progress = $state(0);
  let visible = $state(false);
  let timer: any = null;
  let fadeTimer: any = null;

  $effect(() => {
    if ($navigating) {
      if (fadeTimer) clearTimeout(fadeTimer);
      if (timer) clearInterval(timer);

      visible = true;
      progress = 20;

      timer = setInterval(() => {
        progress = Math.min(progress + (85 - progress) * 0.15, 88);
      }, 150);
    } else {
      if (timer) clearInterval(timer);
      if (visible) {
        progress = 100;
        fadeTimer = setTimeout(() => {
          visible = false;
          progress = 0;
        }, 300);
      }
    }
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
    if (fadeTimer) clearTimeout(fadeTimer);
  });
</script>

{#if visible}
  <div 
    class="admin-nav-progress-bar" 
    style="width: {progress}%; opacity: {progress === 100 ? 0 : 1};"
    aria-hidden="true"
  ></div>
{/if}

<style>
  .admin-nav-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    z-index: 999999;
    background: linear-gradient(90deg, #d97706 0%, #f59e0b 50%, #b45309 100%);
    transition: width 0.2s ease-out, opacity 0.3s ease-out;
    pointer-events: none;
    box-shadow: 0 0 10px rgba(217, 119, 6, 0.6);
  }
</style>
