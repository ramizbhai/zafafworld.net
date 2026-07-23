<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ProfileState } from '../../features/vendor/profileState.svelte.js';

    let { state, data }: { state: ProfileState; data: any } = $props();
    const i18n = getI18n();
</script>

<div class="operational-section">
    <div class="section-header">
        <h2>📋 {i18n.t.pagesConfig.operationalDetails}</h2>
        <p class="section-desc">{i18n.t.pagesConfig.operationalDetailsDesc}<a href="/dashboard/products" class="inline-link">{i18n.t.nav.listings}</a>.</p>
    </div>

    <!-- Halls CTA Banner -->
    <a href="/dashboard/products" class="halls-cta">
        <span class="halls-cta-icon">🏛️</span>
        <div class="halls-cta-text">
            <strong>{i18n.t.pagesConfig.hallsCtaTitle}</strong>
            <span>{i18n.t.pagesConfig.hallsCtaDesc}</span>
        </div>
    </a>

    <div class="ops-grid">
        <div class="form-field">
            <label for="address_en">{i18n.t.pagesConfig.addressEn}</label>
            <input type="text" id="address_en" name="address_en" value={data.vendor?.address_en ?? ''} placeholder={i18n.t.pagesConfig.addressEnPl} />
        </div>
        <div class="form-field text-right" dir="rtl">
            <label for="address_ar">{i18n.t.pagesConfig.addressAr}</label>
            <input type="text" id="address_ar" name="address_ar" value={data.vendor?.address_ar ?? ''} placeholder={i18n.t.pagesConfig.addressArPl} />
        </div>
        <div class="form-field">
            <label for="city_id">{i18n.t.pagesConfig.citySelect}</label>
            <select id="city_id" name="city_id" bind:value={state.selected_city_id} class="city-select">
                <option value="">{i18n.t.pagesConfig.citySelectPl}</option>
                {#each (data.cities || []) as c}
                    <option value={c.id}>{i18n.locale === 'ar' ? (c.name_ar || c.ar) : (c.name_en || c.en)}</option>
                {/each}
            </select>
        </div>
        <div class="form-field">
            <label for="phone">{i18n.t.pagesConfig.phone}</label>
            <input type="tel" id="phone" name="phone" value={data.vendor?.phone ?? ''} placeholder={i18n.t.pagesConfig.phonePl} />
        </div>
        <div class="form-field">
            <label for="email">{i18n.t.pagesConfig.businessEmail}</label>
            <input type="email" id="email" name="email" value={data.vendor?.email ?? ''} oninput={(e) => { e.currentTarget.value = e.currentTarget.value.toLowerCase(); }} placeholder={i18n.t.pagesConfig.businessEmailPl} />
        </div>
        <div class="form-field">
            <label for="latitude">{i18n.t.pagesConfig.latitude}</label>
            <input type="number" step="any" id="latitude" name="latitude" value={data.vendor?.latitude ?? ''} placeholder="24.7136" />
        </div>
        <div class="form-field">
            <label for="longitude">{i18n.t.pagesConfig.longitude}</label>
            <input type="number" step="any" id="longitude" name="longitude" value={data.vendor?.longitude ?? ''} placeholder="46.6753" />
        </div>
        <div class="form-field">
            <label for="website">{i18n.t.pagesConfig.website}</label>
            <input type="url" id="website" name="website" bind:value={state.website} placeholder={i18n.t.pagesConfig.websitePl} />
        </div>
        <div class="form-field">
            <label for="video_url_1">{i18n.t.pagesConfig.videoUrl}</label>
            <input type="url" id="video_url_1" name="video_url_1" bind:value={state.video_url_1} placeholder={i18n.t.pagesConfig.videoUrlPl} />
        </div>
        <div class="form-field">
            <label for="maps_url">{i18n.t.pagesConfig.mapsUrl}</label>
            <input type="url" id="maps_url" name="maps_url" bind:value={state.maps_url} placeholder={i18n.t.pagesConfig.mapsUrlPl} />
        </div>
        <div class="form-field">
            <label for="star_rating">{i18n.t.pagesConfig.starRating}</label>
            <input type="number" step="0.1" min="1" max="5" id="star_rating" name="star_rating" bind:value={state.star_rating} placeholder="5.0" />
        </div>
        <div class="form-field">
            <label for="crm_venue_id">{i18n.t.pagesConfig.crmVenueId}</label>
            <input type="text" id="crm_venue_id" name="crm_venue_id" bind:value={state.crm_venue_id} placeholder={i18n.t.pagesConfig.crmVenueIdPl} />
        </div>
    </div>
</div>

<style>
    /* ─── FORM CONTROLS ──────────────────────────────────────────────────── */
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

    input {
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

    input:focus {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.1);
    }

    .text-right {
        text-align: right;
    }

    .text-right label, .text-right input {
        font-family: 'Cairo', sans-serif;
    }

    /* ─── OPERATIONAL DETAILS SECTION ──────────────────────────────────── */
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

    .ops-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.25rem;
    }

    @media (max-width: 768px) {
        .ops-grid {
            grid-template-columns: 1fr;
        }
    }

    /* ─── HALLS CTA BANNER ──────────────────────────────────────────────── */
    .halls-cta {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 14px 18px;
        background: rgba(91, 33, 182, 0.05);
        border: 1px solid rgba(91, 33, 182, 0.15);
        border-radius: 10px;
        text-decoration: none;
        transition: background 0.15s, border-color 0.15s;
        margin-bottom: 24px;
    }

    .halls-cta:hover {
        background: rgba(91, 33, 182, 0.09);
        border-color: rgba(91, 33, 182, 0.3);
    }

    .halls-cta-icon { font-size: 1.6rem; flex-shrink: 0; }

    .halls-cta-text {
        display: flex;
        flex-direction: column;
        gap: 3px;
    }

    .halls-cta-text strong {
        font-size: 0.88rem;
        font-weight: 700;
        color: var(--color-primary);
    }

    .halls-cta-text span {
        font-size: 0.78rem;
        color: var(--text-sec);
        line-height: 1.4;
    }

    .inline-link {
        color: var(--color-primary);
        text-decoration: underline;
        font-weight: 600;
    }

    /* ─── CITY SELECT ─────────────────────────────────────────────────────── */
    .city-select {
        background: var(--white);
        border: 1.5px solid var(--border);
        border-radius: 8px;
        color: var(--text);
        font-size: 0.9rem;
        padding: 0.75rem 1rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
        width: 100%;
        box-sizing: border-box;
        cursor: pointer;
        appearance: auto;
    }

    .city-select:focus {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.1);
    }
</style>
