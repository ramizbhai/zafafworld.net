<script lang="ts">
  interface Props {
    class?: string;
    height?: string;
    width?: string;
    rounded?: boolean;
    circle?: boolean;
  }

  let { class: extraClass = '', height = 'h-4', width = 'w-full', rounded = false, circle = false }: Props = $props();
</script>

<div
  class="skeleton {height} {width} {circle ? 'rounded-full' : rounded ? 'rounded-lg' : 'rounded'} {extraClass}"
  aria-hidden="true"
  role="presentation"
></div>
