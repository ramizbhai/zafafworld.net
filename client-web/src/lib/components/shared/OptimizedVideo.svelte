<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { resolveMediaUrl, getVideoPosterUrl } from '$lib/shared/utils/media.js';

  // Svelte 5 props definition
  let {
    src = '',
    controls = true,
    autoplay = false,
    loop = false,
    muted = false,
    playsinline = true,
    className = 'w-full rounded-2xl shadow-sm border border-slate-100',
    preload = 'metadata' as 'none' | 'metadata' | 'auto',
    aspectRatio = '16/9',
    style = ''
  } = $props();

  let containerEl = $state<HTMLDivElement | null>(null);
  let videoEl = $state<HTMLVideoElement | null>(null);
  let isIntersecting = $state(false);
  let isVisible = $state(false);
  let hasHlsError = $state(false);
  let hasSentStart = $state(false);

  let resolvedSrc = $derived(resolveMediaUrl(src));
  let webpPoster = $derived(getVideoPosterUrl(src, 'webp'));

  let computedStyle = $derived(
    [
      aspectRatio ? `aspect-ratio: ${aspectRatio};` : '',
      style
    ]
      .filter(Boolean)
      .join(' ')
  );

  let hlsInstance: any = null;

  async function reportTelemetry(event: 'start' | 'stop' | 'buffer' | 'error') {
    try {
      await fetch('/api/v1/telemetry/video', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ event })
      });
    } catch (e) {
      // Fail silently to avoid breaking UI on network drops
    }
  }

  function destroyHls() {
    if (hlsInstance) {
      hlsInstance.destroy();
      hlsInstance = null;
    }
  }

  function handleNativeHlsError() {
    if (videoEl && videoEl.error) {
      console.warn('Native HLS playback error, falling back to MP4:', videoEl.error);
      hasHlsError = true;
      reportTelemetry('error');
      videoEl.removeEventListener('error', handleNativeHlsError);
      videoEl.src = resolvedSrc;
      videoEl.load();
      if (autoplay && isVisible) {
        videoEl.play().catch(() => {});
      }
    }
  }

  function initPlayer() {
    if (!videoEl || !isIntersecting) return;

    let isUploadVideo = src.includes('/ZWV') && src.endsWith('.mp4');
    let hlsSrc = isUploadVideo ? resolvedSrc.replace(/\.mp4$/i, '_master.m3u8') : '';

    if (hlsSrc && !hasHlsError) {
      if (videoEl.canPlayType('application/vnd.apple.mpegurl')) {
        // Native Safari HLS support
        videoEl.src = hlsSrc;
        videoEl.addEventListener('error', handleNativeHlsError);
      } else {
        // Use hls.js fallback
        const scriptId = "hls-js-script";
        let script = document.getElementById(scriptId) as HTMLScriptElement | null;

        const initHlsJs = () => {
          const Hls = (window as any).Hls;
          if (Hls && Hls.isSupported()) {
            destroyHls();
            const hls = new Hls({
              maxBufferLength: 10,
              maxMaxBufferLength: 20,
              enableWorker: true,
              lowLatencyMode: false,
            });
            hlsInstance = hls;
            hls.loadSource(hlsSrc);
            hls.attachMedia(videoEl!);

            hls.on(Hls.Events.ERROR, (event: any, data: any) => {
              if (data.fatal) {
                console.warn('Fatal HLS.js error, falling back to MP4:', data);
                hasHlsError = true;
                reportTelemetry('error');
                destroyHls();
                if (videoEl) {
                  videoEl.src = resolvedSrc;
                  videoEl.load();
                  if (autoplay && isVisible) {
                    videoEl.play().catch(() => {});
                  }
                }
              }
            });

            hls.on(Hls.Events.BUFFER_STALLED, () => {
              reportTelemetry('buffer');
            });
          } else {
            videoEl!.src = resolvedSrc;
          }
        };

        if (!script) {
          script = document.createElement("script");
          script.id = scriptId;
          script.src = "https://cdn.jsdelivr.net/npm/hls.js@1.5.0/dist/hls.min.js";
          script.onload = initHlsJs;
          document.head.appendChild(script);
        } else if ((window as any).Hls) {
          initHlsJs();
        } else {
          script.addEventListener("load", initHlsJs);
        }
      }
    } else {
      videoEl.src = resolvedSrc;
    }
  }

  // Effect to re-initialize player once element mounts/intersects
  $effect(() => {
    if (videoEl && isIntersecting) {
      initPlayer();
    }
  });

  // Track active stream play duration
  $effect(() => {
    if (videoEl && isVisible && !videoEl.paused) {
      if (!hasSentStart) {
        hasSentStart = true;
        reportTelemetry('start');
      }
    } else if (hasSentStart) {
      hasSentStart = false;
      reportTelemetry('stop');
    }
  });

  // Visibility play/pause control
  $effect(() => {
    if (videoEl) {
      if (isVisible) {
        if (autoplay) {
          videoEl.play().catch(() => {});
        }
      } else {
        if (!videoEl.paused) {
          videoEl.pause();
        }
      }
    }
  });

  onMount(() => {
    if (!containerEl) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const [entry] = entries;
        isVisible = entry.isIntersecting;
        if (entry.isIntersecting) {
          isIntersecting = true;
        }
      },
      { rootMargin: '100px' }
    );

    observer.observe(containerEl);

    const handleVisibilityChange = () => {
      if (document.hidden && videoEl && !videoEl.paused) {
        videoEl.pause();
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      observer.disconnect();
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      destroyHls();
      if (hasSentStart) {
        reportTelemetry('stop');
      }
    };
  });

  onDestroy(() => {
    destroyHls();
  });
</script>

<div
  bind:this={containerEl}
  class="relative overflow-hidden bg-slate-950 {className}"
  style={computedStyle}
>
  {#if isIntersecting}
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      bind:this={videoEl}
      poster={webpPoster || undefined}
      {controls}
      {loop}
      {muted}
      {playsinline}
      {preload}
      class="w-full h-full object-cover"
    >
      <span class="text-xs text-slate-400">Your browser does not support HTML5 video.</span>
    </video>
  {:else}
    {#if webpPoster}
      <img
        src={webpPoster}
        alt="Video poster preview"
        class="w-full h-full object-cover brightness-[0.85] pointer-events-none"
        loading="lazy"
        decoding="async"
      />
    {/if}
  {/if}
</div>
