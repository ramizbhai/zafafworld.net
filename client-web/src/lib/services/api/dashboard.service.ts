import type { SubmitFunction } from "@sveltejs/kit";
import { toasts } from "$lib/stores/toast.svelte.js";
import { invalidateAll } from "$app/navigation";
import * as m from "$lib/paraglide/messages.js";
import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

export function handleCancelBooking(state: DashboardState): SubmitFunction {
  return ({ cancel }) => {
    const confirmed = confirm(m.auto_are_you_sure_you_wan());
    if (!confirmed) {
      cancel();
      return;
    }

    state.cancelling = true;
    return async ({ result }) => {
      state.cancelling = false;
      if (result.type === "success") {
        toasts.push("success", m.auto_booking_cancelled_su());
        state.selectedBooking = null; // Close modal
        await invalidateAll();
      } else if (result.type === "failure") {
        toasts.push(
          "error",
          result.data?.message || "Failed to cancel booking",
        );
      } else {
        toasts.push("error", "An unexpected error occurred.");
      }
    };
  };
}

export function handleReadNotifications(state: DashboardState): SubmitFunction {
  return () => {
    state.readingAll = true;
    return async ({ result }) => {
      state.readingAll = false;
      if (result.type === "success") {
        toasts.push("success", m.auto_all_notifications_ma());
        await invalidateAll();
      } else if (result.type === "failure") {
        toasts.push(
          "error",
          result.data?.message || "Failed to mark notifications read",
        );
      }
    };
  };
}

export function getStatusLabel(status: string): string {
  switch (status.toLowerCase()) {
    case "confirmed":
    case "booking_active":
      return m.auto_confirmed();
    case "pending":
    case "draft_inquiry":
    case "pending_vendor_acceptance":
      return m.auto_pending();
    case "cancelled":
      return m.auto_cancelled();
    default:
      return status;
  }
}

export function getStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case "confirmed":
    case "booking_active":
      return "bg-emerald-50 text-emerald-700 border-emerald-200";
    case "pending":
    case "draft_inquiry":
    case "pending_vendor_acceptance":
      return "bg-purple-50 text-purple-700 border-purple-200";
    case "cancelled":
      return "bg-rose-50 text-rose-700 border-rose-200";
    default:
      return "bg-slate-50 text-slate-700 border-slate-200";
  }
}

export function getInquiryStatusLabel(status: string): string {
  switch (status.toLowerCase()) {
    case "unread":
      return m.auto_unread();
    case "viewed":
      return m.auto_viewed();
    case "pending":
      return m.auto_pending();
    case "replied":
      return m.auto_replied();
    case "closed":
      return m.auto_closed();
    case "declined":
      return m.auto_declined();
    default:
      return status;
  }
}

export function getInquiryStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case "replied":
      return "bg-emerald-50 text-emerald-700 border-emerald-200";
    case "unread":
      return "bg-blue-50 text-blue-700 border-blue-200";
    case "viewed":
      return "bg-indigo-50 text-indigo-700 border-indigo-200";
    case "pending":
      return "bg-amber-50 text-amber-700 border-amber-200";
    case "declined":
    case "closed":
      return "bg-rose-50 text-rose-700 border-rose-200";
    default:
      return "bg-slate-50 text-slate-700 border-slate-200";
  }
}
