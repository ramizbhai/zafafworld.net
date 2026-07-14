<script lang="ts">
  import StarRating from "$lib/components/ui/StarRating.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { formatDate } from "$lib/utils/localize.js";
  import * as m from "$lib/paraglide/messages.js";

  let {
    venue,
    state,
    t
  } = $props<{
    venue: any;
    state: any;
    t: (ar: string, en: string) => string;
  }>();

</script>

<div class="flex flex-col gap-6 text-start">
  <!-- Review Stats Header -->
  <div
    class="bg-white p-6 md:p-8 rounded-2xl border border-[var(--color-border)] shadow-sm flex flex-col md:flex-row items-center justify-between gap-6"
  >
    <div>
      <h3
        class="text-lg font-bold text-[var(--color-secondary)] mb-2"
      >
        {m.auto_couple_reviews__rat()}
      </h3>
      <p
        class="text-xs text-[var(--color-muted)] font-medium max-w-sm"
      >
        {m.auto_verified_feedback_fr()}
      </p>
    </div>

    <div
      class="flex items-center gap-4 bg-[var(--color-surface-alt)] p-4 rounded-xl border border-[var(--color-border)] shrink-0 w-full md:w-auto justify-center"
    >
      <div class="text-center">
        <div
          class="font-display text-4xl font-extrabold text-[#5b21b6]"
        >
          {state.averageRating > 0
            ? state.averageRating.toFixed(1)
            : venue.rating}
        </div>
        <p
          class="text-[9px] uppercase font-bold tracking-wider text-[var(--color-muted)] mt-1"
        >
          {state.totalReviews > 0 ? state.totalReviews : venue.reviewCount}
          {m.auto_reviews_1()}
        </p>
      </div>
      <div class="h-10 w-[1px] bg-[var(--color-border)]"></div>
      <div>
        <StarRating
          rating={state.averageRating > 0 ? state.averageRating : venue.rating}
          size="md"
        />
        <p class="text-[10px] text-[var(--color-muted)] mt-1">
          {m.auto_calculated_average_r()}
        </p>
      </div>
    </div>
  </div>

  <!-- Write a review form (Paywalled) -->
  <div
    class="p-6 rounded-2xl border border-[var(--color-border)] bg-white shadow-sm relative overflow-hidden"
  >
    <div
      class="absolute top-0 start-0 w-full h-[3px] bg-gradient-to-r from-[var(--color-primary-contrast)] to-[var(--color-primary)]"
    ></div>

    <h4
      class="font-bold text-[var(--color-secondary)] text-sm mb-4"
    >
      ✍️ {m.auto_share_your_experienc()}
    </h4>

    {#if state.showReviewSuccessOverlay}
      <div
        class="flex flex-col items-center justify-center py-6 text-center bg-[#FDFAF6] border border-emerald-100 rounded-xl p-6"
      >
        <span class="text-3xl mb-2">🎉</span>
        <h5 class="font-bold text-emerald-800 text-sm mb-1">
          {m.auto_review_submitted_suc()}
        </h5>
        <p class="text-xs text-emerald-600 max-w-sm">
          {m.auto_your_review_has_been()}
        </p>
        <button
          type="button"
          onclick={() => (state.showReviewSuccessOverlay = false)}
          class="mt-4 text-xs font-bold text-[#5b21b6] hover:underline cursor-pointer"
        >
          {m.auto_write_another_review()}
        </button>
      </div>
    {:else}
      <form onsubmit={state.submitReviewForm} class="flex flex-col gap-4">
        <!-- Rating Star Select -->
        <div>
          <span
            class="text-xs font-bold text-[var(--color-secondary)] block mb-1"
          >
            {m.auto_overall_rating()} *
          </span>
          <div class="flex gap-1">
            {#each Array(5) as _, i}
              <button
                type="button"
                onclick={() => (state.newRating = i + 1)}
                aria-label="Rate {i + 1} stars"
                class="cursor-pointer transition hover:scale-110 p-1"
              >
                <svg
                  viewBox="0 0 24 24"
                  class="w-7 h-7 {i < state.newRating ? 'text-amber-400 fill-amber-400'
                    : 'text-gray-200'}"
                  stroke="currentColor"
                  stroke-width="1.5"
                >
                  <polygon
                    points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
                  />
                </svg>
              </button>
            {/each}
          </div>
        </div>

        <!-- Input Box -->
        <div class="flex flex-col gap-1.5">
          <label
            for="review-text"
            class="text-xs font-bold text-[var(--color-secondary)]"
          >
            {m.auto_your_detailed_experi()} *
          </label>
          <textarea
            id="review-text"
            bind:value={state.reviewText}
            rows="4"
            placeholder={m.auto_write_your_honest_ex()}
            class="w-full text-sm p-3 border border-[var(--color-border)] rounded-xl outline-none bg-[var(--color-surface)] focus:bg-white focus:border-[var(--color-primary-contrast)] focus:ring-1 focus:ring-[var(--color-primary)] transition"
          ></textarea>
        </div>

        <!-- Images widget -->
        <div>
          <span
            class="text-xs font-bold text-[var(--color-secondary)] block mb-1"
          >
            {m.auto_event__wedding_phot()}
          </span>
          <div class="flex flex-wrap gap-2 items-center">
            <label
              class="w-16 h-16 rounded-xl border border-dashed border-[var(--color-border)] flex flex-col items-center justify-center cursor-pointer hover:bg-[var(--color-surface-alt)] transition"
            >
              <span
                class="text-2xl text-[var(--color-muted)] font-light"
                >+</span
              >
              <span
                class="text-[8px] text-[var(--color-muted)] font-bold uppercase"
                >Upload</span
              >
              <input
                type="file"
                accept="image/*"
                multiple
                onchange={state.handlePhotoUpload}
                class="hidden"
              />
            </label>

            <!-- Display uploads -->
            {#each state.reviewPhotos as img, index}
              <div
                class="relative w-16 h-16 rounded-xl border border-[var(--color-border)] overflow-hidden group shadow-sm"
              >
                <img
                  src={img}
                  alt="Upload thumbnail"
                  class="w-full h-full object-cover"
                />
                <button
                  type="button"
                  onclick={() => state.removePhoto(index)}
                  class="absolute inset-0 bg-black/70 opacity-0 group-hover:opacity-100 flex items-center justify-center text-white text-xs font-extrabold transition cursor-pointer"
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        </div>

        <!-- Submit -->
        <Button
          type="submit"
          variant="primary"
          size="sm"
          loading={state.submittingReview}
          class="self-start px-6 font-bold cursor-pointer"
        >
          {m.auto_publish_review()}
        </Button>
      </form>
    {/if}
  </div>

  <!-- Reviews Feed Stream -->
  <div class="flex flex-col gap-4">
    {#if state.liveReviews.length === 0}
      <div
        class="p-8 rounded-2xl border border-[var(--color-border)] bg-white text-center"
      >
        <span class="text-4xl block mb-2">💬</span>
        <p class="text-sm text-[var(--color-muted)]">
          {m.auto_no_verified_reviews_()}
        </p>
      </div>
    {:else}
      {#each state.liveReviews as review}
        <article
          class="p-6 rounded-2xl border border-[var(--color-border)] bg-white shadow-sm flex flex-col gap-3"
        >
          <div
            class="flex items-start justify-between flex-wrap gap-2"
          >
            <div class="flex items-center gap-3">
              <div
                class="w-10 h-10 rounded-full bg-[var(--color-primary-light)] text-[var(--color-primary-contrast)] flex items-center justify-center font-extrabold text-sm border border-[var(--color-border)]"
              >
                {review.client_name
                  ? review.client_name.charAt(0).toUpperCase()
                  : "C"}
              </div>
              <div>
                <p
                  class="font-bold text-[var(--color-secondary)] text-sm"
                >
                  {review.client_name || "Client"}
                </p>
                <p
                  class="text-[10px] text-[var(--color-muted)] font-medium"
                >
                  {review.created_at
                    ? formatDate(review.created_at.substring(0, 10))
                    : ""}
                </p>
              </div>
            </div>
            <StarRating rating={review.rating} size="sm" />
          </div>

          <p
            class="text-xs md:text-sm text-[var(--color-text)] leading-relaxed"
          >
            {review.review_text}
          </p>

          <!-- Review attachments expand lightbox -->
          {#if review.attachments && review.attachments.length > 0}
            <div class="flex gap-2 flex-wrap mt-2">
              {#each review.attachments as img}
                <a
                  href={img}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="block w-20 h-20 rounded-lg overflow-hidden border border-[var(--color-border)] hover:opacity-90 hover:scale-[1.02] transition shadow-sm"
                >
                  <img
                    src={img}
                    alt="Attached event reference"
                    class="w-full h-full object-cover"
                  />
                </a>
              {/each}
            </div>
          {/if}
        </article>
      {/each}
    {/if}
  </div>
</div>
