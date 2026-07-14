<script lang="ts">
  import { enhance } from '$app/forms';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { 
    MessageSquare, Search, Filter, RefreshCw, AlertCircle, Clock, CheckCircle2,
    Building2, Calendar, User, Phone, Mail, ChevronLeft, ChevronRight, ShieldAlert, AlertTriangle, ArrowRight
  } from 'lucide-svelte';

  let { data } = $props();

  let inquiries = $derived(data.inquiries || []);
  let pagination = $derived(data.pagination || { page: 1, limit: 20, totalItems: 0, totalPages: 0 });
  let metrics = $derived(data.metrics || { total: 0, unread: 0, waitingVendor: 0, escalated: 0, resolvedToday: 0, highPriority: 0 });

  let searchQuery = $state('');
  let selectedStatus = $state('');
  let selectedPriority = $state('');
  let selectedEscalation = $state('');

  $effect(() => {
    searchQuery = data.filters?.q || '';
    selectedStatus = data.filters?.status || '';
    selectedPriority = data.filters?.priority || '';
    selectedEscalation = data.filters?.escalationStatus || '';
  });

  function applyFilters() {
    const params = new URLSearchParams();
    if (searchQuery.trim()) params.set('q', searchQuery.trim());
    if (selectedStatus) params.set('status', selectedStatus);
    if (selectedPriority) params.set('priority', selectedPriority);
    if (selectedEscalation) params.set('escalation_status', selectedEscalation);
    params.set('page', '1');
    goto(`?${params.toString()}`);
  }

  function changePage(newPage: number) {
    const params = new URLSearchParams(page.url.searchParams);
    params.set('page', newPage.toString());
    goto(`?${params.toString()}`);
  }

  function formatDate(dateStr: string): string {
    if (!dateStr) return 'N/A';
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    } catch {
      return dateStr;
    }
  }
</script>

<svelte:head>
  <title>System Leads & All Inquiries | ZafafWorld Admin</title>
</svelte:head>

<div class="space-y-6 p-6">
  <!-- Header -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold tracking-tight flex items-center gap-2.5">
        <MessageSquare class="w-7 h-7 text-rose-600" />
        System Leads & All Inquiries
      </h1>
      <p class="text-sm text-slate-500 mt-1">
        Platform-wide authoritative repository (`vendor_inquiries`) for vendor lead management.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button onclick={() => applyFilters()} class="inline-flex items-center gap-2 px-3.5 py-2 text-sm font-medium bg-white border border-slate-200 rounded-lg shadow-sm hover:bg-slate-50 transition-colors">
        <RefreshCw class="w-4 h-4 text-slate-500" />
        Refresh
      </button>
    </div>
  </div>

  <!-- Summary Cards -->
  <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4">
    <div class="bg-white p-4 rounded-xl border border-slate-200 shadow-sm">
      <div class="flex items-center justify-between text-slate-500 text-xs font-medium">
        <span>Total Leads</span>
        <MessageSquare class="w-4 h-4 text-slate-400" />
      </div>
      <div class="text-2xl font-bold mt-2 text-slate-900">{metrics.total}</div>
    </div>

    <div class="bg-white p-4 rounded-xl border border-rose-200 bg-rose-50/30 shadow-sm">
      <div class="flex items-center justify-between text-rose-700 text-xs font-semibold">
        <span>Unread</span>
        <AlertCircle class="w-4 h-4 text-rose-600" />
      </div>
      <div class="text-2xl font-bold mt-2 text-rose-900">{metrics.unread}</div>
    </div>

    <div class="bg-white p-4 rounded-xl border border-amber-200 bg-amber-50/30 shadow-sm">
      <div class="flex items-center justify-between text-amber-700 text-xs font-semibold">
        <span>Waiting Response</span>
        <Clock class="w-4 h-4 text-amber-600" />
      </div>
      <div class="text-2xl font-bold mt-2 text-amber-900">{metrics.waitingVendor}</div>
    </div>

    <div class="bg-white p-4 rounded-xl border border-purple-200 bg-purple-50/30 shadow-sm">
      <div class="flex items-center justify-between text-purple-700 text-xs font-semibold">
        <span>Escalated</span>
        <ShieldAlert class="w-4 h-4 text-purple-600" />
      </div>
      <div class="text-2xl font-bold mt-2 text-purple-900">{metrics.escalated}</div>
    </div>

    <div class="bg-white p-4 rounded-xl border border-emerald-200 bg-emerald-50/30 shadow-sm">
      <div class="flex items-center justify-between text-emerald-700 text-xs font-semibold">
        <span>Resolved Today</span>
        <CheckCircle2 class="w-4 h-4 text-emerald-600" />
      </div>
      <div class="text-2xl font-bold mt-2 text-emerald-900">{metrics.resolvedToday}</div>
    </div>

    <div class="bg-white p-4 rounded-xl border border-red-200 bg-red-50/30 shadow-sm">
      <div class="flex items-center justify-between text-red-700 text-xs font-semibold">
        <span>High Priority</span>
        <AlertTriangle class="w-4 h-4 text-red-600" />
      </div>
      <div class="text-2xl font-bold mt-2 text-red-900">{metrics.highPriority}</div>
    </div>
  </div>

  <!-- Filters & Search -->
  <div class="bg-white p-4 rounded-xl border border-slate-200 shadow-sm flex flex-col md:flex-row gap-3 items-center justify-between">
    <div class="relative w-full md:w-96">
      <label for="inquiry-search" class="sr-only">Search inquiries</label>
      <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
      <input 
        id="inquiry-search"
        type="text" 
        bind:value={searchQuery} 
        placeholder="Search client, phone, vendor, or inquiry ID..."
        onkeydown={(e) => e.key === 'Enter' && applyFilters()}
        class="w-full pl-9 pr-4 py-2 text-sm border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-rose-500/20 focus:border-rose-500"
      />
    </div>

    <div class="flex flex-wrap items-center gap-2.5 w-full md:w-auto">
      <label for="filter-status" class="sr-only">Filter by status</label>
      <select id="filter-status" bind:value={selectedStatus} onchange={() => applyFilters()} class="px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-rose-500/20">
        <option value="">All Statuses</option>
        <option value="unread">Unread</option>
        <option value="viewed">Viewed</option>
        <option value="pending">Pending</option>
        <option value="replied">Replied</option>
        <option value="closed">Closed</option>
        <option value="declined">Declined</option>
      </select>

      <label for="filter-priority" class="sr-only">Filter by priority</label>
      <select id="filter-priority" bind:value={selectedPriority} onchange={() => applyFilters()} class="px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-rose-500/20">
        <option value="">All Priorities</option>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>

      <label for="filter-escalation" class="sr-only">Filter by escalation status</label>
      <select id="filter-escalation" bind:value={selectedEscalation} onchange={() => applyFilters()} class="px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-rose-500/20">
        <option value="">All Escalations</option>
        <option value="none">Normal (None)</option>
        <option value="pending">Escalation Pending</option>
        <option value="escalated">Escalated</option>
        <option value="resolved">Escalation Resolved</option>
      </select>

      <button onclick={() => applyFilters()} class="px-4 py-2 text-sm font-medium bg-rose-600 text-white rounded-lg hover:bg-rose-700 transition-colors">
        Filter
      </button>
    </div>
  </div>

  <!-- Data Table -->
  <div class="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left border-collapse text-sm">
        <thead>
          <tr class="bg-slate-50 border-b border-slate-200 text-slate-600 text-xs font-semibold uppercase tracking-wider">
            <th class="py-3.5 px-4">Client</th>
            <th class="py-3.5 px-4">Vendor & Listing</th>
            <th class="py-3.5 px-4">Event Date</th>
            <th class="py-3.5 px-4">Source</th>
            <th class="py-3.5 px-4">Priority & Escalation</th>
            <th class="py-3.5 px-4">Status</th>
            <th class="py-3.5 px-4 text-right">Action</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-200">
          {#if inquiries.length === 0}
            <tr>
              <td colspan="7" class="py-12 text-center text-slate-500">
                <MessageSquare class="w-10 h-10 mx-auto mb-3 text-slate-300" />
                No system inquiries found matching your filter criteria.
              </td>
            </tr>
          {:else}
            {#each inquiries as item (item.id)}
              <tr class="hover:bg-slate-50/80 transition-colors {item.status === 'unread' ? 'bg-rose-50/20 font-medium' : ''}">
                <td class="py-3.5 px-4">
                  <div class="flex items-center gap-2">
                    {#if item.status === 'unread'}
                      <span class="w-2 h-2 rounded-full bg-rose-600 shrink-0" title="Unread Lead"></span>
                    {/if}
                    <div>
                      <div class="text-slate-900 font-semibold">{item.client?.name || 'Guest User'}</div>
                      <div class="text-xs text-slate-500 flex items-center gap-3 mt-0.5">
                        {#if item.client?.phone}<span>{item.client.phone}</span>{/if}
                        {#if item.client?.email}<span class="truncate max-w-[140px]">{item.client.email}</span>{/if}
                      </div>
                    </div>
                  </div>
                </td>

                <td class="py-3.5 px-4">
                  <div class="text-slate-900 font-medium flex items-center gap-1.5">
                    <Building2 class="w-3.5 h-3.5 text-slate-400" />
                    {item.vendor?.nameEn || item.vendor?.nameAr || 'Vendor'}
                  </div>
                  {#if item.listing}
                    <div class="text-xs text-rose-600 font-medium mt-0.5 truncate max-w-[200px]">
                      {item.listing.titleEn || item.listing.titleAr}
                    </div>
                  {:else}
                    <div class="text-xs text-slate-400 mt-0.5">General Vendor Inquiry</div>
                  {/if}
                </td>

                <td class="py-3.5 px-4 whitespace-nowrap">
                  <div class="text-slate-900 font-medium">{formatDate(item.eventDate)}</div>
                  <div class="text-xs text-slate-500 mt-0.5">{item.guestCount} Guests</div>
                </td>

                <td class="py-3.5 px-4 whitespace-nowrap">
                  {#if item.isGuest}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-slate-100 text-slate-700">Guest</span>
                  {:else}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-50 text-blue-700">Verified Client</span>
                  {/if}
                </td>

                <td class="py-3.5 px-4 whitespace-nowrap">
                  <div class="flex items-center gap-1.5">
                    {#if item.management?.priority === 'critical'}
                      <span class="px-2 py-0.5 text-xs font-semibold bg-red-100 text-red-800 rounded">Critical</span>
                    {:else if item.management?.priority === 'high'}
                      <span class="px-2 py-0.5 text-xs font-semibold bg-orange-100 text-orange-800 rounded">High</span>
                    {:else if item.management?.priority === 'low'}
                      <span class="px-2 py-0.5 text-xs font-medium bg-slate-100 text-slate-600 rounded">Low</span>
                    {:else}
                      <span class="px-2 py-0.5 text-xs font-medium bg-blue-50 text-blue-700 rounded">Medium</span>
                    {/if}

                    {#if item.management?.escalationStatus === 'escalated'}
                      <span class="px-2 py-0.5 text-xs font-bold bg-purple-100 text-purple-800 rounded flex items-center gap-1">
                        <ShieldAlert class="w-3 h-3" /> Escalated
                      </span>
                    {/if}
                  </div>
                </td>

                <td class="py-3.5 px-4 whitespace-nowrap">
                  {#if item.status === 'unread'}
                    <span class="px-2.5 py-1 text-xs font-semibold bg-rose-100 text-rose-800 rounded-full">Unread</span>
                  {:else if item.status === 'replied'}
                    <span class="px-2.5 py-1 text-xs font-semibold bg-emerald-100 text-emerald-800 rounded-full">Replied</span>
                  {:else if item.status === 'closed'}
                    <span class="px-2.5 py-1 text-xs font-medium bg-slate-100 text-slate-700 rounded-full">Closed</span>
                  {:else}
                    <span class="px-2.5 py-1 text-xs font-medium bg-amber-100 text-amber-800 rounded-full">{item.status}</span>
                  {/if}
                </td>

                <td class="py-3.5 px-4 text-right whitespace-nowrap">
                  <a href="/dashboard/inquiries/{item.id}" class="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-semibold text-rose-600 bg-rose-50 rounded-lg hover:bg-rose-100 transition-colors">
                    Inspect Detail
                    <ArrowRight class="w-3.5 h-3.5" />
                  </a>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>

    <!-- Pagination Footer -->
    {#if pagination.totalPages > 1}
      <div class="px-4 py-3 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
        <div class="text-xs text-slate-500">
          Showing page <span class="font-semibold text-slate-700">{pagination.page}</span> of <span class="font-semibold text-slate-700">{pagination.totalPages}</span> ({pagination.totalItems} items)
        </div>
        <div class="flex items-center gap-1">
          <button 
            disabled={pagination.page <= 1}
            onclick={() => changePage(pagination.page - 1)}
            class="p-1.5 text-slate-600 rounded-lg hover:bg-slate-200 disabled:opacity-40 disabled:hover:bg-transparent"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <button 
            disabled={pagination.page >= pagination.totalPages}
            onclick={() => changePage(pagination.page + 1)}
            class="p-1.5 text-slate-600 rounded-lg hover:bg-slate-200 disabled:opacity-40 disabled:hover:bg-transparent"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
