<script lang="ts">
  import BlogEditorForm from '$lib/components/admin/blog/BlogEditorForm.svelte';
  import { AlertCircle } from 'lucide-svelte';

  let { data, form } = $props<{ data: any; form: any }>();
</script>

{#if form?.error}
  <div class="notice-banner error max-w-6xl mx-auto mb-6 px-4">
    <AlertCircle size={18} class="notice-icon" />
    <div class="notice-text">{form.error}</div>
  </div>
{/if}

<BlogEditorForm
  bind:blog={data.blog}
  bind:allCategories={data.allCategories}
  bind:allTags={data.allTags}
  action="?/save"
  isNew={true}
/>
