<script lang="ts">
  import { navigating } from "$app/stores";
  import { onDestroy } from "svelte";

  let progress = $state(0);
  let visible = $state(false);
  let timer: any = null;
  let fadeTimer: any = null;

  $effect(() => {
    if ($navigating) {
      // Navigation started
      if (fadeTimer) clearTimeout(fadeTimer);
      if (timer) clearInterval(timer);

      visible = true;
      progress = 15;

      // Incrementally trickle progress up to ~85%
      timer = setInterval(() => {
        progress = Math.min(progress + (85 - progress) * 0.15, 88);
      }, 150);
    } else {
      // Navigation finished
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
    class="zw-nav-progress-bar" 
    style="width: {progress}%; opacity: {progress === 100 ? 0 : 1};"
    aria-hidden="true"
  >
    <div class="zw-nav-progress-glow"></div>
  </div>
{/if}

<style>
  .zw-nav-progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    z-index: 999999;
    background: linear-gradient(
      90deg,
      #9B7A42 0%,
      #C9A96E 35%,
      #F7E7B0 70%,
      #C9A96E 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.8s infinite linear;
    transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.3s ease-out;
    pointer-events: none;
    box-shadow: 0 0 10px rgba(201, 169, 110, 0.7), 0 0 4px rgba(201, 169, 110, 0.4);
  }

  .zw-nav-progress-glow {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 100px;
    box-shadow: 0 0 15px #F7E7B0, 0 0 8px #C9A96E;
    transform: rotate(3deg) translate(0px, -4px);
    opacity: 0.8;
  }

  @keyframes shimmer {
    0% { background-position: 100% 0; }
    100% { background-position: -100% 0; }
  }
</style>
