<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { createDashboardState } from "$lib/stores/dashboardState.svelte.js";

  import DashboardHeader from "$lib/components/dashboard/DashboardHeader.svelte";
  import DashboardCountdown from "$lib/components/dashboard/DashboardCountdown.svelte";
  import DashboardBudget from "$lib/components/dashboard/DashboardBudget.svelte";
  import ActiveBookings from "$lib/components/dashboard/ActiveBookings.svelte";
  import RecentInquiries from "$lib/components/dashboard/RecentInquiries.svelte";
  import ConversationsWidget from "$lib/components/dashboard/ConversationsWidget.svelte";
  import NotificationsWidget from "$lib/components/dashboard/NotificationsWidget.svelte";
  import ActivityFeedWidget from "$lib/components/dashboard/ActivityFeedWidget.svelte";
  import BookingDetailModal from "$lib/components/dashboard/BookingDetailModal.svelte";

  let { data } = $props();

  let state = createDashboardState(() => data);

  $effect(() => {
    return state.startCountdown();
  });
</script>

<svelte:head>
  <title>{m.auto_my_dashboard()} — {m.meta_siteName()}</title>
</svelte:head>

<div class="min-h-screen bg-warm-ivory pb-16">
  <!-- Header Banner -->
  <DashboardHeader user={data.user} conversations={data.conversations} />

  <div class="container-page py-8 flex flex-col gap-8">
    <!-- Top Area Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      <!-- Wedding Countdown Widget -->
      <DashboardCountdown {state} />

      <!-- Budget Overview Widget -->
      <DashboardBudget {state} />
    </div>

    <!-- Main Content Area Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <!-- Left + Center Columns (Main Area) -->
      <div class="lg:col-span-2 flex flex-col gap-8">
        <!-- Active Bookings Widget -->
        <ActiveBookings {state} activeBookings={data.bookings} />

        <!-- Recent Inquiries Widget -->
        <RecentInquiries inquiries={data.inquiries} />

        <!-- Recent Conversations Widget -->
        <ConversationsWidget conversations={data.conversations} />
      </div>

      <!-- Right Column (Side Area) -->
      <div class="flex flex-col gap-8">
        <!-- Notifications Widget -->
        <NotificationsWidget {state} notifications={data.notifications} />

        <!-- Activity Feed Widget -->
        <ActivityFeedWidget activities={data.activities} />
      </div>
    </div>
  </div>
</div>

<BookingDetailModal {state} />

<style>
  .bg-warm-ivory {
    background-color: #faf8f5 !important;
  }
</style>
