<script lang="ts">
  import { getI18n } from "$lib/i18n/i18n.svelte";
  import { ClientBuilderState } from "$lib/components/listing/clientBuilderState.svelte.js";
  import StepBaseInfo from "$lib/shared/builder/StepBaseInfo.svelte";
  import StepFeatures from "$lib/shared/builder/StepFeatures.svelte";
  import StepMedia from "$lib/shared/builder/StepMedia.svelte";
  import StepPreview from "$lib/shared/builder/StepPreview.svelte";

  const i18n = getI18n();
  let currentLang = $derived(i18n.locale);

  let {
      descriptionAr = $bindable(""),
      descriptionEn = $bindable(""),
      titleAr = "",
      titleEn = "",
      metaTitleAr = $bindable(""),
      metaTitleEn = $bindable(""),
      metaDescriptionAr = $bindable(""),
      metaDescriptionEn = $bindable(""),
      readonly = false,
      hideSeo = false,
      locale = undefined,
  } = $props<{
      descriptionAr?: string;
      descriptionEn?: string;
      titleAr?: string;
      titleEn?: string;
      metaTitleAr?: string;
      metaTitleEn?: string;
      metaDescriptionAr?: string;
      metaDescriptionEn?: string;
      readonly?: boolean;
      locale?: "ar" | "en";
      hideSeo?: boolean;
  }>();

  const state = new ClientBuilderState({
      titleAr,
      titleEn,
      metaTitleAr,
      metaTitleEn,
      metaDescriptionAr,
      metaDescriptionEn,
      descriptionAr,
      descriptionEn
  });

  state.onSync = (data) => {
      descriptionAr = data.descriptionAr;
      descriptionEn = data.descriptionEn;
      metaTitleAr = data.metaTitleAr;
      metaTitleEn = data.metaTitleEn;
      metaDescriptionAr = data.metaDescriptionAr;
      metaDescriptionEn = data.metaDescriptionEn;
  };
</script>

<div class="unified-description-builder">
  {#if readonly}
      <StepPreview {state} />
  {:else}
      <div class="wizard-header">
          <div class="step-progress">
              {#each Array(state.totalSteps) as _, i}
                  <button
                      type="button"
                      class="step-dot"
                      class:active={state.currentStep === i + 1}
                      class:completed={state.currentStep > i + 1}
                      onclick={() => state.goToStep(i + 1)}
                  >
                      {i + 1}
                  </button>
                  {#if i < state.totalSteps - 1}
                      <div class="step-line" class:completed={state.currentStep > i + 1}></div>
                  {/if}
              {/each}
          </div>
      </div>

      <div class="wizard-content">
          {#if state.currentStep === 1}
              <StepBaseInfo {state} {hideSeo} />
          {:else if state.currentStep === 2}
              <StepFeatures {state} />
          {:else if state.currentStep === 3}
              <StepMedia {state} />
          {:else if state.currentStep === 4}
              <StepPreview {state} />
          {/if}
      </div>

      <div class="wizard-footer">
          <button
              type="button"
              class="btn-nav btn-prev"
              disabled={state.currentStep === 1}
              onclick={() => state.prevStep()}
          >
              {currentLang === "ar" ? "السابق" : "Previous"}
          </button>
          <button
              type="button"
              class="btn-nav btn-next"
              disabled={state.currentStep === state.totalSteps}
              onclick={() => state.nextStep()}
          >
              {currentLang === "ar" ? "التالي" : "Next"}
          </button>
      </div>
  {/if}
</div>

<style>
  .unified-description-builder {
      display: flex;
      flex-direction: column;
      width: 100%;
      background: #fff;
      border: 1px solid #e2e8f0;
      border-radius: 16px;
      overflow: hidden;
      box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
  }

  .wizard-header {
      padding: 24px;
      border-bottom: 1px solid #e2e8f0;
      background: #f8fafc;
  }

  .step-progress {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 12px;
  }

  .step-dot {
      width: 32px;
      height: 32px;
      border-radius: 50%;
      background: #fff;
      border: 2px solid #e2e8f0;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 600;
      color: #64748b;
      cursor: pointer;
      transition: all 0.2s;
  }

  .step-dot.active {
      border-color: #6366f1;
      background: #6366f1;
      color: #fff;
  }

  .step-dot.completed {
      border-color: #10b981;
      background: #10b981;
      color: #fff;
  }

  .step-line {
      flex: 1;
      max-width: 60px;
      height: 2px;
      background: #e2e8f0;
  }

  .step-line.completed {
      background: #10b981;
  }

  .wizard-content {
      padding: 32px 24px;
      min-height: 400px;
  }

  .wizard-footer {
      padding: 16px 24px;
      border-top: 1px solid #e2e8f0;
      background: #f8fafc;
      display: flex;
      justify-content: space-between;
      align-items: center;
  }

  .btn-nav {
      padding: 10px 24px;
      border-radius: 8px;
      font-weight: 600;
      font-size: 14px;
      cursor: pointer;
      transition: all 0.2s;
  }

  .btn-prev {
      background: #fff;
      border: 1px solid #e2e8f0;
      color: #64748b;
  }

  .btn-prev:disabled {
      opacity: 0.5;
      cursor: not-allowed;
  }

  .btn-next {
      background: #6366f1;
      border: 1px solid #6366f1;
      color: #fff;
  }

  .btn-next:disabled {
      opacity: 0.5;
      cursor: not-allowed;
  }
</style>
