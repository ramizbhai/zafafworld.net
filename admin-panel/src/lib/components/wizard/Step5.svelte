<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { CATEGORY_GROUPS } from "$lib/constants/wizard";
    import FeaturesSelection from "$lib/components/FeaturesSelection.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { env } from "$env/dynamic/public";
    import { ChevronDown } from "lucide-svelte";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();
    const apiBase =
        env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost")
            ? ""
            : env.PUBLIC_API_URL || "";

    function getAttrLabel(name: string, defaultLabel: string): string {
        return i18n.t(`attributeLabels.${name}`) || defaultLabel;
    }
    function getAttrPlaceholder(
        name: string,
        defaultPlaceholder: string,
    ): string {
        if (name.includes("capacity")) {
            return (
                i18n.t("attributeLabels.placeholder_capacity") ||
                defaultPlaceholder
            );
        }
        if (name.includes("weeks")) {
            return (
                i18n.t("attributeLabels.placeholder_weeks") ||
                defaultPlaceholder
            );
        }
        if (name.includes("surcharge")) {
            return (
                i18n.t("attributeLabels.placeholder_surcharge") ||
                defaultPlaceholder
            );
        }
        if (name.includes("staff") || name.includes("size")) {
            return (
                i18n.t("attributeLabels.placeholder_staff") ||
                defaultPlaceholder
            );
        }
        return defaultPlaceholder;
    }

    let categoryAttributes = $state($listingStore.formData.categoryAttributes);
    let featuresSelection = $state($listingStore.formData.featuresSelection);
    let selectedCategory = $derived($listingStore.formData.selectedCategory);

    const selectedCategoryMeta = $derived.by(() => {
        for (const group of CATEGORY_GROUPS) {
            const found = group.items.find((i) => i.value === selectedCategory);
            if (found) return found;
        }
        return null;
    });

    $effect(() => {
        listingStore.setCanContinue(true);
    });

    $effect(() => {
        listingStore.updateFormData({ categoryAttributes, featuresSelection });

        listingStore.setSubmitHandler(async () => {
            listingStore.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = `${apiBase}/api/v1/vendor/products/${$listingStore.productId}`;
                const payload = {
                    version: $listingStore.version,
                    attributes: categoryAttributes,
                    featuresSelection: featuresSelection,
                };

                const res = await fetch(url, {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                    },
                    body: JSON.stringify(payload),
                });

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.setHighestStep(5);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-6`);
            } catch (err: any) {
                listingStore.setError(err.message || "Failed to save details.");
            } finally {
                listingStore.setSubmitting(false);
            }
        });
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">
            {selectedCategoryMeta?.emoji || "📋"}
        </div>
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? selectedCategoryMeta?.ar
                    : selectedCategoryMeta?.en}
                {i18n.locale === "ar" ? " تفاصيل" : " Details"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? `يرجى ملء التفاصيل المحددة لـ ${selectedCategoryMeta?.ar} التي يبحث عنها العملاء عند التصفح.`
                    : `Fill in the specific details for your ${selectedCategoryMeta?.en?.toLowerCase() || "listing"} that clients look for when browsing.`}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card">
        <!-- Capacity fields for venues -->
        {#if ["wedding-palace", "hotel-venue", "villa-resort", "outdoor-garden", "chalet", "restaurant-event"].includes(selectedCategory)}
            <div class="form-row">
                <div class="form-group">
                    <label for="men_capacity">
                        {getAttrLabel("men_capacity", "Men's Section Capacity")}
                        <span class="field-hint-inline"
                            >({i18n.locale === "ar" ? "شخص" : "guests"})</span
                        >
                    </label>
                    <input
                        type="number"
                        id="men_capacity"
                        min="0"
                        bind:value={categoryAttributes["men_capacity"]}
                        placeholder={getAttrPlaceholder(
                            "men_capacity",
                            "e.g. 400",
                        )}
                    />
                </div>
                <div class="form-group">
                    <label for="women_capacity">
                        {getAttrLabel(
                            "women_capacity",
                            "Women's Section Capacity",
                        )}
                        <span class="field-hint-inline"
                            >({i18n.locale === "ar" ? "شخص" : "guests"})</span
                        >
                    </label>
                    <input
                        type="number"
                        id="women_capacity"
                        min="0"
                        bind:value={categoryAttributes["women_capacity"]}
                        placeholder={getAttrPlaceholder(
                            "women_capacity",
                            "e.g. 600",
                        )}
                    />
                </div>
            </div>
            <div class="form-group">
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={
                            categoryAttributes["has_separate_entrances"]
                        }
                    />
                    {getAttrLabel(
                        "has_separate_entrances",
                        "Separate Entrances for Men and Women",
                    )}
                </label>
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={categoryAttributes["has_audio_link"]}
                    />
                    {getAttrLabel("has_audio_link", "Audio Link Between Halls")}
                </label>
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={categoryAttributes["max_events_per_day"]}
                    />
                    {getAttrLabel(
                        "max_events_per_day",
                        "Exclusive (only 1 event per day)",
                    )}
                </label>
            </div>

            <!-- Villa & Chalet: private pool + weekend surcharge -->
            {#if ["villa-resort", "chalet"].includes(selectedCategory)}
                <hr class="form-divider" />
                <div class="form-group">
                    <label for="weekend_surcharge">
                        {getAttrLabel(
                            "weekend_surcharge_sar",
                            "Weekend Surcharge (SAR)",
                        )}
                        <span class="field-hint-inline"
                            >({i18n.locale === "ar"
                                ? "اختياري"
                                : "optional"})</span
                        >
                    </label>
                    <input
                        id="weekend_surcharge"
                        type="number"
                        min="0"
                        bind:value={categoryAttributes["weekend_surcharge_sar"]}
                        placeholder={getAttrPlaceholder(
                            "weekend_surcharge_sar",
                            "e.g. 500",
                        )}
                    />
                </div>
                <div class="form-group">
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={categoryAttributes["private_pool"]}
                        />
                        {getAttrLabel("private_pool", "Private Pool Available")}
                    </label>
                </div>
            {/if}

            <!-- Hotel Venue: in-house catering -->
            {#if selectedCategory === "hotel-venue"}
                <hr class="form-divider" />
                <div class="form-group">
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={
                                categoryAttributes["in_house_catering"]
                            }
                        />
                        {getAttrLabel(
                            "in_house_catering",
                            "In-house Catering Provided",
                        )}
                    </label>
                </div>
            {/if}

            <!-- Restaurant: private hall + family section -->
            {#if selectedCategory === "restaurant-event"}
                <hr class="form-divider" />
                <div class="form-group">
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={
                                categoryAttributes["private_hall_available"]
                            }
                        />
                        {getAttrLabel(
                            "private_hall_available",
                            "Private Hall Available",
                        )}
                    </label>
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={categoryAttributes["family_section"]}
                        />
                        {getAttrLabel(
                            "family_section",
                            "Family Section Available",
                        )}
                    </label>
                </div>
            {/if}
        {/if}

        <!-- Photography/Video specific fields -->
        {#if ["photography-video", "photo-studio"].includes(selectedCategory)}
            <div class="form-group">
                <label for="team_size"
                    >{getAttrLabel("team_size", "Team Size")}</label
                >
                <input
                    id="team_size"
                    type="number"
                    min="1"
                    max="20"
                    bind:value={categoryAttributes["team_size"]}
                    placeholder={getAttrPlaceholder("team_size", "e.g. 3")}
                />
            </div>
            <div class="form-group">
                <label for="delivery_weeks"
                    >{getAttrLabel(
                        "delivery_weeks",
                        "Delivery Time (Weeks)",
                    )}</label
                >
                <input
                    id="delivery_weeks"
                    type="number"
                    min="1"
                    max="52"
                    bind:value={categoryAttributes["delivery_weeks"]}
                    placeholder={getAttrPlaceholder("delivery_weeks", "e.g. 4")}
                />
            </div>
            <div class="form-group">
                <span
                    style="display: block; font-size: 0.875rem; font-weight: 600; color: var(--text); margin-bottom: 8px;"
                    >{i18n.locale === "ar" ? "خيارات إضافية" : "Options"}</span
                >
                <div class="amenity-checks">
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={
                                categoryAttributes["female_team_available"]
                            }
                        />
                        {getAttrLabel(
                            "female_team_available",
                            "Female Team Available",
                        )}
                    </label>
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={
                                categoryAttributes["women_section_coverage"]
                            }
                        />
                        {getAttrLabel(
                            "women_section_coverage",
                            "Women Section Coverage",
                        )}
                    </label>
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={categoryAttributes["drone_available"]}
                        />
                        {getAttrLabel("drone_available", "Drone Photography")}
                    </label>
                    <label class="amenity-check-label">
                        <input
                            type="checkbox"
                            bind:checked={categoryAttributes["highlight_reel"]}
                        />
                        {getAttrLabel("highlight_reel", "Highlight Reel")}
                    </label>
                </div>
            </div>
        {/if}

        <!-- Catering specific -->
        {#if selectedCategory === "catering"}
            <div class="form-row">
                <div class="form-group">
                    <label for="min_guests"
                        >{getAttrLabel(
                            "min_guests",
                            "Minimum Guests Required",
                        )}</label
                    >
                    <input
                        id="min_guests"
                        type="number"
                        min="10"
                        bind:value={categoryAttributes["min_guests"]}
                        placeholder={getAttrPlaceholder(
                            "min_guests",
                            "e.g. 50",
                        )}
                    />
                </div>
                <div class="form-group">
                    <label for="buffet_or_plated"
                        >{getAttrLabel(
                            "buffet_or_plated",
                            "Service Type",
                        )}</label
                    >
                    <div class="select-wrapper">
                        <select
                            id="buffet_or_plated"
                            bind:value={categoryAttributes["buffet_or_plated"]}
                        >
                            <option value=""
                                >— {i18n.locale === "ar" ? "اختر" : "Select"} —</option
                            >
                            <option value="buffet"
                                >{i18n.locale === "ar"
                                    ? "بوفيه مفتوح"
                                    : "Buffet"}</option
                            >
                            <option value="plated"
                                >{i18n.locale === "ar"
                                    ? "خدمة أطباق"
                                    : "Plated Service"}</option
                            >
                            <option value="both"
                                >{i18n.locale === "ar"
                                    ? "كلا الخيارين"
                                    : "Both Options"}</option
                            >
                        </select>
                        <ChevronDown size={16} class="select-icon" />
                    </div>
                </div>
            </div>
            <div class="amenity-checks">
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={categoryAttributes["halal_certified"]}
                    />
                    {getAttrLabel("halal_certified", "Halal Certified")}
                </label>
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={
                            categoryAttributes["serving_staff_included"]
                        }
                    />
                    {getAttrLabel(
                        "serving_staff_included",
                        "Serving Staff Included",
                    )}
                </label>
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={categoryAttributes["taste_testing"]}
                    />
                    {getAttrLabel("taste_testing", "Taste Testing Available")}
                </label>
                <label class="amenity-check-label">
                    <input
                        type="checkbox"
                        bind:checked={categoryAttributes["setup_cleanup"]}
                    />
                    {getAttrLabel("setup_cleanup", "Setup & Cleanup Included")}
                </label>
            </div>
        {/if}

        <!-- Fallback for other categories -->
        {#if !["wedding-palace", "hotel-venue", "villa-resort", "outdoor-garden", "chalet", "restaurant-event", "photography-video", "photo-studio", "catering"].includes(selectedCategory)}
            <p class="step-placeholder">
                ✅ {i18n.t("listingsWizard.noSpecificDetails")}
            </p>
        {/if}

        <hr class="form-divider" />
        <div class="form-group" style="margin-top: 2rem;">
            <h3 style="margin-bottom: 1rem; font-size: 1.1rem; color: #374151;">
                {i18n.locale === "ar"
                    ? "الميزات الإضافية (اختياري)"
                    : "Additional Features (Optional)"}
            </h3>
            <FeaturesSelection bind:featuresSelection />
        </div>
    </div>
</div>
