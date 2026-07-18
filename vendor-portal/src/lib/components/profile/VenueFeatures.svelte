<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ProfileState } from '../../features/vendor/profileState.svelte.js';

    let { state }: { state: ProfileState } = $props();
    const i18n = getI18n();
</script>

<div class="operational-section">
    <div class="section-header">
        <h2>🏛️ {i18n.t.pagesConfig.venueFeatures}</h2>
        <p class="section-desc">{i18n.t.pagesConfig.venueFeaturesDesc}</p>
    </div>

    <div class="capabilities-grid">
        <div class="form-field">
            <label for="capacity_min">{i18n.t.pagesConfig.minCapacity}</label>
            <input 
                type="number" 
                id="capacity_min" 
                name="capacity_min" 
                bind:value={state.capacity_min} 
                placeholder={i18n.t.pagesConfig.minCapacityPl} 
                min="0"
            />
        </div>
        <div class="form-field">
            <label for="capacity_max">{i18n.t.pagesConfig.maxCapacity}</label>
            <input 
                type="number" 
                id="capacity_max" 
                name="capacity_max" 
                bind:value={state.capacity_max} 
                placeholder={i18n.t.pagesConfig.maxCapacityPl} 
                min="0"
            />
        </div>
    </div>

    <div class="features-toggles mt-6">
        <!-- Ladies/Gents Partition Segregation -->
        <div class="toggle-field">
            <label class="custom-toggle">
                <input 
                    type="checkbox" 
                    name="has_partition" 
                    value="true" 
                    bind:checked={state.has_partition} 
                />
                <span class="toggle-slider"></span>
                <span class="toggle-label-text">
                    <strong>{i18n.t.pagesConfig.strictSegregation}</strong>
                    <span>{i18n.t.pagesConfig.strictSegregationDesc}</span>
                </span>
            </label>
        </div>
    </div>

    <!-- Saudi Market Amenities Chips -->
    <div class="amenities-selection mt-8">
        <span class="block-label">{i18n.t.pagesConfig.specializedAmenities}</span>
        <p class="field-help mb-4">{i18n.t.pagesConfig.specializedAmenitiesDesc}</p>
        <div class="amenity-chips-grid">
            <label class="chip-checkbox-label {state.selectedAmenities.includes('in_house_catering') ? 'checked' : ''}">
                <input 
                    type="checkbox" 
                    name="amenities" 
                    value="in_house_catering" 
                    bind:group={state.selectedAmenities} 
                />
                <span class="chip-icon">🍽️</span>
                <span class="chip-text">{i18n.t.pagesConfig.inHouseCatering}</span>
            </label>

            <label class="chip-checkbox-label {state.selectedAmenities.includes('valet_parking') ? 'checked' : ''}">
                <input 
                    type="checkbox" 
                    name="amenities" 
                    value="valet_parking" 
                    bind:group={state.selectedAmenities} 
                />
                <span class="chip-icon">🚗</span>
                <span class="chip-text">{i18n.t.pagesConfig.valetParking}</span>
            </label>

            <label class="chip-checkbox-label {state.selectedAmenities.includes('bridal_suite') ? 'checked' : ''}">
                <input 
                    type="checkbox" 
                    name="amenities" 
                    value="bridal_suite" 
                    bind:group={state.selectedAmenities} 
                />
                <span class="chip-icon">👰</span>
                <span class="chip-text">{i18n.t.pagesConfig.bridalSuite}</span>
            </label>

            <label class="chip-checkbox-label {state.selectedAmenities.includes('zaffa_setup') ? 'checked' : ''}">
                <input 
                    type="checkbox" 
                    name="amenities" 
                    value="zaffa_setup" 
                    bind:group={state.selectedAmenities} 
                />
                <span class="chip-icon">✨</span>
                <span class="chip-text">{i18n.t.pagesConfig.zaffaSetup}</span>
            </label>
        </div>
    </div>
</div>

<style>
    /* ─── VENUE FEATURES & CAPABILITIES STYLES ───────────────────────────────── */
    .operational-section {
        border-top: 1px solid var(--border-light);
        padding-top: 2.5rem;
        margin-top: 2.5rem;
    }

    .section-header h2 {
        margin: 0 0 0.25rem 0;
        font-size: 1.3rem;
        font-weight: 700;
        color: var(--text);
    }

    .section-desc {
        margin: 0 0 1.75rem 0;
        font-size: 0.8rem;
        color: var(--text-sec);
        line-height: 1.4;
    }
    
    .capabilities-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.25rem;
    }

    @media (max-width: 768px) {
        .capabilities-grid {
            grid-template-columns: 1fr;
        }
    }

    .form-field {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    label {
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--text);
    }

    input[type="number"] {
        background: var(--white);
        border: 1.5px solid var(--border);
        border-radius: 8px;
        color: var(--text);
        font-size: 0.9rem;
        padding: 0.75rem 1rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
        width: 100%;
        box-sizing: border-box;
    }

    input[type="number"]:focus {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.1);
    }

    .mt-6 { margin-top: 1.5rem; }
    .mt-8 { margin-top: 2rem; }
    .mb-4 { margin-bottom: 1rem; }

    /* Custom premium switch toggle style */
    .custom-toggle {
        display: flex;
        align-items: flex-start;
        gap: 12px;
        cursor: pointer;
        padding: 1.25rem;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 12px;
        transition: background 0.2s, border-color 0.2s;
    }

    .custom-toggle:hover {
        background: var(--bg);
        border-color: var(--color-primary);
    }

    .custom-toggle input[type="checkbox"] {
        display: none;
    }

    .toggle-slider {
        width: 44px;
        height: 22px;
        background: var(--border);
        border-radius: 100px;
        position: relative;
        flex-shrink: 0;
        transition: background 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        border: 1px solid var(--border-light);
    }

    .toggle-slider::before {
        content: "";
        width: 16px;
        height: 16px;
        background: var(--white);
        border-radius: 50%;
        position: absolute;
        top: 2px;
        left: 2px;
        transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), background-color 0.25s;
    }


    .custom-toggle input:checked + .toggle-slider {
        background: linear-gradient(135deg, var(--color-secondary) 0%, var(--color-secondary) 100%);
    }

    .custom-toggle input:checked + .toggle-slider::before {
        transform: translateX(22px);
        background: var(--white);
    }


    .toggle-label-text {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .toggle-label-text strong {
        font-size: 0.88rem;
        color: var(--text);
    }

    .toggle-label-text span {
        font-size: 0.78rem;
        color: var(--text-sec);
    }

    /* Specialized Saudi Amenities Chips Grid */
    .block-label {
        font-size: 0.9rem;
        font-weight: 700;
        color: var(--text);
        margin-bottom: 0.25rem;
        display: block;
    }

    .field-help {
        font-size: 0.78rem;
        color: var(--text-sec);
    }

    .amenity-chips-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 1rem;
    }

    .chip-checkbox-label {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 12px 16px;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 10px;
        cursor: pointer;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
        user-select: none;
    }

    .chip-checkbox-label input {
        display: none;
    }

    .chip-checkbox-label:hover {
        background: var(--bg);
        border-color: var(--border-focus);
    }

    .chip-checkbox-label.checked {
        background: rgba(91, 33, 182, 0.05);
        border-color: var(--color-primary);
        box-shadow: 0 0 12px rgba(91, 33, 182, 0.1);
    }

    .chip-checkbox-label.checked .chip-text {
        color: var(--color-primary);
        font-weight: 600;
    }

    .chip-icon {
        font-size: 1.25rem;
    }

    .chip-text {
        font-size: 0.82rem;
        color: var(--text-sec);
        transition: color 0.2s;
    }
</style>
