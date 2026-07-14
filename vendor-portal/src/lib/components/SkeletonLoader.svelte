<script lang="ts">
  interface Props {
    width?: string;
    height?: string;
    radius?: string;
    lines?: number;
    variant?: 'text' | 'rect' | 'circle';
    class?: string;
  }

  let {
    width = '100%',
    height = '18px',
    radius = '4px',
    lines = 1,
    variant = 'rect',
    class: className = ''
  }: Props = $props();

  let lineCount = $derived(Array.from({ length: lines }));

  function getStyle(index: number) {
    let finalWidth = width;
    if (variant === 'text' && lines > 1 && index === lines - 1) {
      finalWidth = '60%'; // last text line shorter for realistic text shape
    }
    
    let finalRadius = radius;
    if (variant === 'circle') {
      finalRadius = '50%';
    }

    return `
      width: ${finalWidth};
      height: ${variant === 'circle' ? width : height};
      border-radius: ${finalRadius};
    `;
  }
</script>

<div class="skeleton-wrapper {className}">
  {#each lineCount as _, i}
    <div
      class="skeleton-pulse {variant}"
      style={getStyle(i)}
      aria-hidden="true"
    ></div>
  {/each}
</div>

<style>
  .skeleton-wrapper {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .skeleton-pulse {
    background: linear-gradient(
      90deg,
      #e2e8f0 25%,
      #f1f5f9 37%,
      #e2e8f0 63%
    );
    background-size: 400% 100%;
    animation: skeleton-glow 1.5s ease-in-out infinite;
  }

  /* Support dark-theme values slightly if user has custom theme classes */
  :global(.dark) .skeleton-pulse {
    background: linear-gradient(
      90deg,
      #1e293b 25%,
      #334155 37%,
      #1e293b 63%
    );
    background-size: 400% 100%;
  }

  .circle {
    aspect-ratio: 1 / 1;
    flex-shrink: 0;
  }

  @keyframes skeleton-glow {
    0% {
      background-position: 100% 50%;
    }
    100% {
      background-position: 0% 50%;
    }
  }
</style>
