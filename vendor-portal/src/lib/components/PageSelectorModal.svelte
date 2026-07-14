<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';

  let {
    open = false,
    pages = [],
    onClose = () => {},
    onNext = () => {}
  } = $props<{
    open: boolean;
    pages?: any[];
    onClose: () => void;
    onNext: () => void;
  }>();

  const i18n = getI18n();

  let selectedPage = $state('');

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }
</script>

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdrop}
    onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
    role="dialog"
    aria-modal="true"
    aria-labelledby="ps-modal-title"
    tabindex="-1"
  >
    <div class="modal">
      <div class="modal-header">
        <h3 id="ps-modal-title" class="modal-title">{i18n.t.pricing.pageSelectTitle}</h3>
        <button class="modal-close" onclick={onClose} aria-label={i18n.t.common.close}>&times;</button>
      </div>

      <div class="modal-body">
        <label class="field-label" for="page-selector-select">{i18n.t.pages.pageTitle}</label>
        <div class="select-wrap">
          <select
            id="page-selector-select"
            class="page-select"
            bind:value={selectedPage}
          >
            <option value="" disabled selected>{i18n.t.pricing.selectPlaceholder}</option>
            {#each pages as p}
              <option value={p.title}>{p.title}</option>
            {/each}
          </select>
          <svg class="select-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="6 9 12 15 18 9"/>
          </svg>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={onClose}>{i18n.t.common.back}</button>
        <button class="btn btn-primary" onclick={onNext} disabled={!selectedPage}>{i18n.t.common.next}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.45);
    z-index: 200; display: flex; align-items: center;
    justify-content: center; padding: 20px;
    animation: fadeIn 0.15s ease;
    backdrop-filter: blur(4px);
  }
  .modal {
    background: var(--white); border-radius: var(--radius);
    width: 100%; max-width: 480px; box-shadow: var(--shadow-lg);
    animation: slideUp 0.2s ease;
  }
  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 18px 20px; border-bottom: 1px solid var(--border);
  }
  .modal-title { font-size: 16px; font-weight: 700; color: var(--text); }
  .modal-close {
    background: none; border: none; cursor: pointer; color: var(--text-sec);
    font-size: 24px; font-weight: 300; width: 30px; height: 30px;
    border-radius: 6px; display: flex; align-items: center; justify-content: center;
    line-height: 1; transition: background 0.15s; font-family: var(--font);
  }
  .modal-close:hover { background: var(--bg); }
  .modal-body { padding: 20px; }
  .modal-footer { display: flex; gap: 10px; padding: 16px 20px; border-top: 1px solid var(--border); justify-content: flex-end; }

  .field-label { display: block; font-size: 13px; font-weight: 600; color: var(--text); margin-bottom: 8px; text-align: var(--text-align); }
  .select-wrap { position: relative; }
  .page-select {
    width: 100%; height: 44px; border: 1.5px solid var(--border);
    border-radius: var(--radius-sm); padding: 0 16px;
    font-size: 14px; font-family: var(--font); color: var(--text);
    background: var(--white); outline: none; appearance: none;
    cursor: pointer; transition: border 0.15s;
    text-align: var(--text-align);
  }
  .page-select:focus { border-color: var(--teal); box-shadow: 0 0 0 3px rgba(26,158,122,0.1); }
  
  .select-arrow {
    position: absolute;
    top: 50%;
    transform: translateY(-50%) var(--arrow-rotation);
    color: var(--text-light);
    pointer-events: none;
  }
  :global(html[dir="rtl"]) .select-arrow { left: 16px; }
  :global(html[dir="ltr"]) .select-arrow { right: 16px; }

  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @keyframes slideUp { from { transform: translateY(10px) scale(0.98); } to { transform: translateY(0) scale(1); } }
</style>
