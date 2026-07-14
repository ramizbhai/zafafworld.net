<script lang="ts">
  interface Props {
    id?: string;
    name?: string;
    type?: string;
    value?: string;
    placeholder?: string;
    label?: string;
    error?: string;
    hint?: string;
    disabled?: boolean;
    required?: boolean;
    class?: string;
    leadingIcon?: string;
    trailingIcon?: string;
    oninput?: (e: Event) => void;
    onchange?: (e: Event) => void;
  }

  let {
    id,
    name,
    type = 'text',
    value = $bindable(''),
    placeholder,
    label,
    error,
    hint,
    disabled = false,
    required = false,
    class: extraClass = '',
    leadingIcon,
    trailingIcon,
    oninput,
    onchange,
  }: Props = $props();

  const randomId = `input-${Math.random().toString(36).slice(2, 8)}`;
  const inputId = $derived(id ?? randomId);
  const errorId = $derived(`${inputId}-error`);
  const hintId = $derived(`${inputId}-hint`);
</script>

<div class="flex flex-col gap-1.5 {extraClass}">
  {#if label}
    <label
      for={inputId}
      class="text-sm font-medium text-text"
    >
      {label}
      {#if required}
        <span class="text-error ms-0.5" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div class="relative">
    {#if leadingIcon}
      <span class="absolute inset-y-0 start-3 flex items-center pointer-events-none text-muted">
        {@html leadingIcon}
      </span>
    {/if}

    <input
      id={inputId}
      {name}
      {type}
      bind:value
      {placeholder}
      {disabled}
      {required}
      aria-invalid={!!error}
      aria-describedby={[error ? errorId : null, hint ? hintId : null].filter(Boolean).join(' ') || undefined}
      class="
        w-full rounded-lg border bg-white px-4 py-3
        text-text placeholder:text-muted
        transition-colors duration-150
        focus:outline-none focus:ring-2 focus:ring-primary focus:border-primary
        disabled:opacity-50 disabled:cursor-not-allowed
        {leadingIcon ? 'ps-10' : ''}
        {trailingIcon ? 'pe-10' : ''}
        {error
          ? 'border-error focus:ring-error'
          : 'border-border hover:border-primary-light'}
      "
      {oninput}
      {onchange}
    />

    {#if trailingIcon}
      <span class="absolute inset-y-0 end-3 flex items-center pointer-events-none text-muted">
        {@html trailingIcon}
      </span>
    {/if}
  </div>

  {#if error}
    <p id={errorId} class="text-sm text-error" role="alert">{error}</p>
  {:else if hint}
    <p id={hintId} class="text-sm text-muted">{hint}</p>
  {/if}
</div>
