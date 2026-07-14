<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { getContext } from "svelte";
    import { WizardFinalState } from "../../features/vendor/wizard/wizardFinalState.svelte";
    import Step9Terms from "./Step9Terms.svelte";
    import Step9SubmitBar from "./Step9SubmitBar.svelte";
    import { listingStore } from "$lib/stores/listingStore";

    let { data } = $props<{ data: { sessionToken: string } }>();
    const i18n = getI18n();

    const state = new WizardFinalState();
    
    const wizard = getContext<{ setCanContinue: (val: boolean) => void }>('wizard');

    $effect(() => {
        state.setSessionToken(data.sessionToken);
        wizard.setCanContinue(true);
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">🚀</div>
        <div>
            <h1>{i18n.locale === "ar" ? "إرسال للمراجعة والتفعيل" : "Submit for Approval"}</h1>
            <p>{i18n.locale === "ar" ? "سيتم مراجعة قائمتك من قبل فريقنا في غضون 24 ساعة." : "Your listing will be reviewed by our team within 24 hours."}</p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">⚠️ {$listingStore.submitError}</div>
    {/if}

    <div class="form-card">
        <Step9Terms {state} />
        <Step9SubmitBar {state} />
    </div>
</div>
