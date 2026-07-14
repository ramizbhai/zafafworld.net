<script lang="ts">
    import { Plus } from "lucide-svelte";
    import { BLOCK_TYPES } from "$lib/constants/blockTypes";

    let {
        index,
        isActive,
        currentLang = "en",
        isBottom = false,
        onToggle,
        onAdd,
    } = $props<{
        index: number;
        isActive: boolean;
        currentLang?: "ar" | "en";
        isBottom?: boolean;
        onToggle: () => void;
        onAdd: (type: string, index: number) => void;
    }>();
</script>

{#if isBottom}
    <div class="add-new-field-wrapper mt-4">
        <button
            type="button"
            class="btn-add-block primary-add-btn"
            onclick={onToggle}
        >
            <Plus size={16} />
            <span>
                {currentLang === "ar" ? "إضافة قسم جديد" : "Add New Field"}
            </span>
        </button>
        {#if isActive}
            <div class="inline-add-menu bottom-menu">
                {#each BLOCK_TYPES as bt}
                    {@const Icon = bt.icon}
                    <button
                        type="button"
                        class="menu-item-btn"
                        onclick={() => onAdd(bt.id, index)}
                    >
                        <Icon size={14} />
                        <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>
{:else}
    <div class="inline-add-zone" class:active={isActive}>
        <button
            type="button"
            class="inline-add-btn"
            onclick={onToggle}
        >
            <Plus size={16} />
        </button>
        {#if isActive}
            <div class="inline-add-menu">
                {#each BLOCK_TYPES as bt}
                    {@const Icon = bt.icon}
                    <button
                        type="button"
                        class="menu-item-btn"
                        onclick={() => onAdd(bt.id, index)}
                    >
                        <Icon size={14} />
                        <span>{currentLang === "ar" ? bt.labelAr : bt.labelEn}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>
{/if}
