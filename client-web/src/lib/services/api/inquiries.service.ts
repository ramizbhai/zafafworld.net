import { env } from '$env/dynamic/public';
import { toasts } from '$lib/stores/toast.svelte.js';
import * as m from '$lib/paraglide/messages.js';
import { trackBlogFunnelEvent } from '$lib/utils/analytics.js';
import { invalidateAll } from '$app/navigation';

export async function submitVenueInquiry(data: {
  venue: any;
  user: any;
  eventDate: string;
  guestCount: number | null;
  inquiryMessage: string;
  isVenue: boolean;
}): Promise<{ success: boolean; error?: string }> {
  const { venue, user, eventDate, guestCount, inquiryMessage, isVenue } = data;

  if (!eventDate || (isVenue && (!guestCount || guestCount <= 0))) {
    toasts.push("error", m.auto_please_provide_valid());
    return { success: false, error: "Invalid parameters" };
  }

  try {
    const response = await fetch("/api/v1/public/inquiries", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        listingId: venue.id,
        vendorId: venue.vendorId || venue.vendor?.id || venue.id,
        eventDate,
        guestCount: guestCount ? Math.floor(guestCount) : 1,
        message: inquiryMessage,
        name: user?.name || "Guest User",
        phone: user?.phone || "+966500000000",
      }),
    });

    const result = await response.json();

    if (response.ok && result.status === "success") {
      toasts.push("success", m.auto_your_lead_inquiry_ha());
      trackBlogFunnelEvent("inquiry_start");
      await invalidateAll();
      return { success: true };
    } else {
      return { success: false, error: result.message || m.auto_failed_to_submit_inq() };
    }
  } catch (err: any) {
    console.error(err);
    return { success: false, error: m.auto_an_unexpected_error_() };
  }
}
