import { env } from '$env/dynamic/public';
import { toasts } from '$lib/stores/toast.svelte.js';
import * as m from '$lib/paraglide/messages.js';
import { invalidateAll } from '$app/navigation';

export async function submitVenueReview(data: {
  venueId: string;
  rating: number;
  reviewText: string;
  reviewPhotos: string[];
}): Promise<{ success: boolean; error?: string }> {
  const { venueId, rating, reviewText, reviewPhotos } = data;

  if (reviewText.trim().length === 0) {
    toasts.push("error", m.auto_please_enter_a_revie());
    return { success: false, error: "Empty review" };
  }

  try {
    const API_BASE = env.PUBLIC_API_URL || "http://localhost:8080";
    const response = await fetch(`${API_BASE}/api/v1/client/reviews`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        vendorId: venueId,
        rating: rating,
        reviewText: reviewText,
        attachments: reviewPhotos,
      }),
    });

    const resData = await response.json();
    if (response.ok && resData.status === "success") {
      await invalidateAll();
      return { success: true };
    } else {
      return { success: false, error: resData.message || m.auto_failed_to_submit_rev() };
    }
  } catch (err) {
    console.error(err);
    return { success: false, error: m.auto_connection_error_sub() };
  }
}

export async function fetchLiveReviews(venueId: string) {
    const API_BASE = env.PUBLIC_API_URL || "http://localhost:8080";
    try {
        const response = await fetch(`${API_BASE}/api/v1/public/vendors/${venueId}/reviews`);
        const data = await response.json();
        if (data.status === "success") {
            return {
                liveReviews: data.reviews,
                averageRating: data.average_rating,
                totalReviews: data.total_count
            };
        }
    } catch (err) {
        console.error("Failed to load live reviews:", err);
    }
    return null;
}
