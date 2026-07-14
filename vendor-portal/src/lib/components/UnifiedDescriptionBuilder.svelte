<script lang="ts">
    import { vendorStore } from "$lib/stores/vendorStore";
    import { triggerUpgrade } from "$lib/stores/upgradeStore";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { Sparkles, Globe } from "lucide-svelte";
    import { BLOCK_TYPES } from "$lib/constants/blockTypes";
    import {
        parseBlocks,
        mergeBlocks,
        syncBlocksToJSON,
        generateSeoSuggestions,
        generateSlugPreview,
    } from "$lib/services/descriptionBlocks.service";

    import ReadonlyView from "./DescriptionBuilder/ReadonlyView.svelte";
    import AddBlockMenu from "./DescriptionBuilder/AddBlockMenu.svelte";
    import BlockEditor from "./DescriptionBuilder/BlockEditor.svelte";
    import SeoSettings from "./DescriptionBuilder/SeoSettings.svelte";
    import SeoPreview from "./DescriptionBuilder/SeoPreview.svelte";
    import "./DescriptionBuilder/styles.css";

    const i18n = getI18n();

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

    let blocks = $state<any[]>([]);
    let showAddMenuAtIndex = $state<number | null>(null);

    let lastAr = "";
    let lastEn = "";

    $effect(() => {
        if (descriptionAr !== lastAr || descriptionEn !== lastEn) {
            try {
                const arParsed = parseBlocks(descriptionAr || "");
                const enParsed = parseBlocks(descriptionEn || "");
                blocks = mergeBlocks(arParsed, enParsed);
                lastAr = descriptionAr || "";
                lastEn = descriptionEn || "";
            } catch (e) {
                blocks = [];
            }
        }
    });

    function syncValue() {
        const result = syncBlocksToJSON(blocks);
        lastAr = result.newAr;
        lastEn = result.newEn;
        descriptionAr = result.newAr;
        descriptionEn = result.newEn;
        autoGenerateSEO();
    }

    let isDiamond = $derived($vendorStore.tier_id === "diamond");
    let maxBlocks = $derived($vendorStore.policy_limits?.description_blocks ?? 5);
    let currentBlocks = $derived(blocks.length);
    let atLimit = $derived(maxBlocks !== -1 && currentBlocks >= maxBlocks);

    function checkLimit(action: string) {
        if (atLimit && !isDiamond) {
            const msgEn = `You have reached your plan limit of ${maxBlocks} description blocks. Upgrade required to ${action}.`;
            const msgAr = `لقد وصلت إلى الحد الأقصى لباقتك وهو ${maxBlocks} أقسام وصف. الترقية مطلوبة لـ ${action === 'add more' ? 'إضافة المزيد' : 'نسخ المزيد'}.`;
            triggerUpgrade(
                "description_blocks",
                $vendorStore.tier_id,
                i18n.locale === "ar" ? msgAr : msgEn,
            );
            return false;
        }
        return true;
    }

    function addBlock(type: string, index: number = blocks.length) {
        if (!checkLimit('add more')) return;
        const newBlock = { type, contentAr: "", contentEn: "", url: "" };
        const newBlocks = [...blocks];
        newBlocks.splice(index, 0, newBlock);
        blocks = newBlocks;
        showAddMenuAtIndex = null;
        syncValue();
    }

    function duplicateBlock(index: number) {
        if (!checkLimit('duplicate blocks')) return;
        const blockToCopy = JSON.parse(JSON.stringify(blocks[index]));
        const newBlocks = [...blocks];
        newBlocks.splice(index + 1, 0, blockToCopy);
        blocks = newBlocks;
        syncValue();
    }

    function removeBlock(index: number) {
        blocks = blocks.filter((_, i) => i !== index);
        syncValue();
    }

    function moveUp(index: number) {
        if (index === 0) return;
        const newBlocks = [...blocks];
        const temp = newBlocks[index];
        newBlocks[index] = newBlocks[index - 1];
        newBlocks[index - 1] = temp;
        blocks = newBlocks;
        syncValue();
    }

    function moveDown(index: number) {
        if (index === blocks.length - 1) return;
        const newBlocks = [...blocks];
        const temp = newBlocks[index];
        newBlocks[index] = newBlocks[index + 1];
        newBlocks[index + 1] = temp;
        blocks = newBlocks;
        syncValue();
    }

    function autoGenerateSEO() {
        const res = generateSeoSuggestions(
            blocks,
            metaTitleEn,
            metaTitleAr,
            metaDescriptionEn,
            metaDescriptionAr,
            titleEn,
            titleAr
        );
        metaTitleEn = res.metaTitleEn;
        metaTitleAr = res.metaTitleAr;
        metaDescriptionEn = res.metaDescriptionEn;
        metaDescriptionAr = res.metaDescriptionAr;
    }

    let slugPreview = $derived(generateSlugPreview(titleEn, titleAr));
    let currentLang = $derived(locale || i18n.locale) as "ar" | "en";
</script>

<div class="unified-description-container">
    {#if readonly}
        <ReadonlyView {blocks} {currentLang} />
    {:else}
        <div class="builder-header-banner">
            <div class="header-left">
                <span class="step-icon-badge">✍️</span>
                <div>
                    <h3>
                        {currentLang === "ar"
                            ? "أقسام وصف الإعلان"
                            : "Listing Description Blocks"}
                    </h3>
                    <p class="subtitle-desc">
                        {currentLang === "ar"
                            ? "اكتب بكلتا اللغتين للوصول لجمهور أكبر"
                            : "Write in both languages for maximum audience reach"}
                    </p>
                </div>
            </div>
            <div class="lang-requirement-badge">
                <Globe size={13} />
                <span>
                    {currentLang === "ar" ? "يدعم تعدد اللغات" : "Multi-language supported"}
                </span>
            </div>
        </div>

        <div class="blocks-stack">
            {#if blocks.length > 0}
                <AddBlockMenu
                    index={0}
                    isActive={showAddMenuAtIndex === 0}
                    {currentLang}
                    onToggle={() => (showAddMenuAtIndex = showAddMenuAtIndex === 0 ? null : 0)}
                    onAdd={addBlock}
                />
            {/if}

            {#each blocks as block, i}
                <BlockEditor
                    bind:block={blocks[i]}
                    index={i}
                    isFirst={i === 0}
                    isLast={i === blocks.length - 1}
                    {currentLang}
                    onMoveUp={moveUp}
                    onMoveDown={moveDown}
                    onDuplicate={duplicateBlock}
                    onRemove={removeBlock}
                    onSync={syncValue}
                />
                <AddBlockMenu
                    index={i + 1}
                    isActive={showAddMenuAtIndex === i + 1}
                    {currentLang}
                    onToggle={() => (showAddMenuAtIndex = showAddMenuAtIndex === i + 1 ? null : i + 1)}
                    onAdd={addBlock}
                />
            {/each}

            {#if blocks.length > 0}
                <AddBlockMenu
                    index={blocks.length}
                    isActive={showAddMenuAtIndex === blocks.length}
                    {currentLang}
                    isBottom={true}
                    onToggle={() => (showAddMenuAtIndex = blocks.length)}
                    onAdd={addBlock}
                />
            {/if}

            {#if blocks.length === 0}
                <div class="empty-blocks-state">
                    <p class="empty-txt">
                        {currentLang === "ar"
                            ? "لا يوجد أقسام وصف مضافة بعد."
                            : "No description blocks added yet."}
                    </p>
                    <div class="add-actions-group center-actions mt-4">
                        {#each BLOCK_TYPES as bt}
                            {@const Icon = bt.icon}
                            <button
                                type="button"
                                class="btn-add-block"
                                onclick={() => addBlock(bt.id)}
                            >
                                <Icon size={14} />
                                <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                            </button>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>

        {#if !hideSeo}
            <div class="seo-optimization-section mt-4">
                <div class="seo-section-head">
                    <Sparkles size={16} class="seo-sparkles-icon" />
                    <h4>
                        {currentLang === "ar"
                            ? "تحسين محركات البحث (SEO)"
                            : "Search Engine Optimization (SEO)"}
                    </h4>
                    <span class="auto-suggest-pill">
                        {currentLang === "ar" ? "اقتراح تلقائي نشط" : "Auto-Suggestions Active"}
                    </span>
                </div>

                <div class="seo-layout-grid">
                    <SeoSettings
                        bind:metaTitleEn
                        bind:metaTitleAr
                        bind:metaDescriptionEn
                        bind:metaDescriptionAr
                    />
                    <SeoPreview
                        {currentLang}
                        {slugPreview}
                        {metaTitleEn}
                        {metaTitleAr}
                        {titleEn}
                        {titleAr}
                        {metaDescriptionEn}
                        {metaDescriptionAr}
                    />
                </div>
            </div>
        {/if}
    {/if}
</div>
