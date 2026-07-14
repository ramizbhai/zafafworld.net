<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';

  let {
    open = false,
    onClose = () => {},
    onConfirm = () => {}
  } = $props<{
    open: boolean;
    onClose: () => void;
    onConfirm: () => void;
  }>();

  const i18n = getI18n();

  let selected = $state('visa');

  let methods = $derived([
    { id: 'visa', label: i18n.t.pricing.creditCard, logo: 'VISA', logoBg: '#1a1f71', logoColor: '#fff' },
    { id: 'apple', label: i18n.t.pricing.applePay, logo: 'Apple Pay', logoBg: '#000', logoColor: '#fff' },
    { id: 'paypal', label: i18n.t.pricing.paypal, logo: 'PayPal', logoBg: '#003087', logoColor: '#fff' },
  ]);

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
    aria-labelledby="pay-modal-title"
    tabindex="-1"
  >
    <div class="modal">
      <div class="modal-header">
        <h3 id="pay-modal-title" class="modal-title">{i18n.t.pricing.paymentTitle}</h3>
        <button class="modal-close" onclick={onClose} aria-label={i18n.t.common.close}>&times;</button>
      </div>

      <div class="modal-body">
        {#each methods as method}
          <button
            class="payment-option {selected === method.id ? 'selected' : ''}"
            onclick={() => selected = method.id}
          >
            <div class="payment-check {selected === method.id ? 'checked' : ''}">
              {#if selected === method.id}✓{/if}
            </div>
            <span class="payment-label">{method.label}</span>
            <span class="payment-logo" style="background:{method.logoBg};color:{method.logoColor}">
              {method.logo}
            </span>
          </button>
        {/each}
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={onClose}>{i18n.t.common.back}</button>
        <button class="btn btn-primary" onclick={onConfirm}>{i18n.t.common.confirm}</button>
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
  .modal-body { padding: 20px; display: flex; flex-direction: column; gap: 10px; }
  .modal-footer { display: flex; gap: 10px; padding: 16px 20px; border-top: 1px solid var(--border); justify-content: flex-end; }

  .payment-option {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px; border: 1.5px solid var(--border);
    border-radius: 10px; cursor: pointer; background: none;
    font-family: var(--font); width: 100%; transition: all 0.15s;
    gap: 12px;
  }
  .payment-option:hover { border-color: var(--teal); background: var(--teal-light); }
  .payment-option.selected { border-color: var(--teal); background: var(--teal-light); }

  .payment-check {
    width: 22px; height: 22px; border-radius: 50%;
    border: 1.5px solid var(--border); display: flex;
    align-items: center; justify-content: center;
    font-size: 12px; color: #fff; flex-shrink: 0; transition: all 0.15s;
  }
  .payment-check.checked { background: var(--teal); border-color: var(--teal); }

  .payment-label { font-size: 14px; font-weight: 600; color: var(--text); flex: 1; text-align: var(--text-align); }

  .payment-logo {
    padding: 4px 10px; border-radius: 4px; font-size: 12px;
    font-weight: 700; font-style: italic; flex-shrink: 0; white-space: nowrap;
  }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @keyframes slideUp { from { transform: translateY(10px) scale(0.98); } to { transform: translateY(0) scale(1); } }
</style>
