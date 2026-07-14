<script lang="ts">
    import { page } from '$app/stores';
    import Loading from '$lib/components/Loading.svelte';
    import type { PageData } from './$types';

    let { data } = $props<{ data: PageData }>();

    let stepId = $derived(parseInt($page.params.stepId || '1'));

    // Dynamic imports map for steps to enable Vite code splitting
    const steps = {
        1: () => import('$lib/components/wizard/Step1.svelte'),
        2: () => import('$lib/components/wizard/Step2.svelte'),
        3: () => import('$lib/components/wizard/Step3.svelte'),
        4: () => import('$lib/components/wizard/Step4.svelte'),
        5: () => import('$lib/components/wizard/Step5.svelte'),
        6: () => import('$lib/components/wizard/Step6.svelte'),
        7: () => import('$lib/components/wizard/Step7.svelte'),
        8: () => import('$lib/components/wizard/Step8.svelte'),
        9: () => import('$lib/components/wizard/Step9.svelte')
    };
</script>

{#key stepId}
    {#await steps[stepId as keyof typeof steps]()}
        <Loading show={true} />
    {:then { default: StepComponent }}
        <StepComponent {data} />
    {/await}
{/key}
