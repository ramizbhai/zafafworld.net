<script lang="ts">
  import { env } from "$env/dynamic/public";
  import { t, lang } from "$lib/i18n/index.js";
  import {
    Search,
    RefreshCw,
    Sparkles,
    Phone,
    Calendar,
    User,
    MessageSquare,
    CheckCircle2,
    Clock,
    AlertTriangle,
    ExternalLink,
  } from "lucide-svelte";

  interface Props {
    data: { inquiries: any[] };
  }
  let { data }: Props = $props();

  // svelte-ignore state_referenced_locally
  let inquiries = $state<any[]>(data.inquiries || []);
  let searchQuery = $state("");
  let isLoading = $state(false);
  let statusFilter = $state("all");
  let errorMsg = $state("");
  let updatingIds = $state<string[]>([]);

  $effect(() => {
    inquiries = data.inquiries || [];
  });

  let filteredInquiries = $derived.by(() => {
    let filtered = inquiries;
    if (statusFilter !== "all") {
      filtered = filtered.filter((i) => i.status === statusFilter);
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (i) =>
          i.name?.toLowerCase().includes(q) ||
          i.phone?.includes(q) ||
          i.email?.toLowerCase().includes(q) ||
          i.message?.toLowerCase().includes(q),
      );
    }
    return filtered;
  });

  let totalCount = $derived(inquiries.length);
  let pendingCount = $derived(
    inquiries.filter((i) => i.status === "pending").length,
  );
  let contactedCount = $derived(
    inquiries.filter((i) => i.status === "contacted").length,
  );
  let resolvedCount = $derived(
    inquiries.filter((i) => i.status === "resolved").length,
  );

  async function refreshInquiries() {
    isLoading = true;
    errorMsg = "";
    try {
      const res = await fetch(`/api/v1/admin/afrah/inquiries`);
      if (res.ok) {
        const body = await res.json();
        if (body.status === "success") inquiries = body.inquiries || [];
      } else {
        errorMsg = "Failed to load Afrah inquiries.";
      }
    } catch {
      errorMsg = "Network error.";
    } finally {
      isLoading = false;
    }
  }

  async function updateStatus(id: string, newStatus: string) {
    updatingIds = [...updatingIds, id];
    errorMsg = "";
    try {
      const res = await fetch(
        `/api/v1/admin/afrah/inquiries/${id}/status`,
        {
          method: "PATCH",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ status: newStatus }),
        },
      );
      if (res.ok) {
        inquiries = inquiries.map((i) =>
          i.id === id ? { ...i, status: newStatus } : i,
        );
      } else {
        errorMsg = "Failed to update status.";
      }
    } catch {
      errorMsg = "Network error.";
    } finally {
      updatingIds = updatingIds.filter((uid) => uid !== id);
    }
  }

  function formatDate(dt: string) {
    return new Date(dt).toLocaleDateString(
      $lang === "ar" ? "ar-SA" : "en-US",
      {
        year: "numeric",
        month: "short",
        day: "numeric",
      },
    );
  }

  function formatDateTime(dt: string) {
    return new Date(dt).toLocaleString($lang === "ar" ? "ar-SA" : "en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function getStatusColor(status: string) {
    switch (status) {
      case "pending":
        return "var(--warning)";
      case "contacted":
        return "var(--info, #3b82f6)";
      case "resolved":
        return "var(--success)";
      default:
        return "var(--text-ghost)";
    }
  }

  function getStatusBg(status: string) {
    switch (status) {
      case "pending":
        return "var(--warning-dim, rgba(234,179,8,0.12))";
      case "contacted":
        return "rgba(59,130,246,0.12)";
      case "resolved":
        return "var(--success-dim, rgba(34,197,94,0.12))";
      default:
        return "var(--bg-elevated)";
    }
  }

  function openWhatsApp(phone: string, name: string) {
    const msg = encodeURIComponent(
      `Hello ${name}, we received your wedding planning request on ZafafWorld. We'd love to help you plan your perfect day! 🎊`,
    );
    window.open(`https://wa.me/${phone.replace("+", "")}?text=${msg}`, "_blank");
  }
</script>

<svelte:head>
  <title>Afrah VIP Desk | ZafafWorld Admin</title>
</svelte:head>

<div class="fade-in">
  <!-- HEADER -->
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">
        <Sparkles size={20} class="inline-block mr-2" style="color:var(--gold)"
        />
        {$lang === "ar" ? "مكتب أفراح VIP" : "Afrah VIP Desk"}
      </h1>
      <p class="page-subtitle">
        {$lang === "ar"
          ? "إدارة استفسارات تخطيط الزفاف من خدمة أفراح الشخصية"
          : "Manage wedding planning inquiries from the Afrah concierge service"}
      </p>
    </div>
    <div class="page-header-right">
      <button
        onclick={refreshInquiries}
        disabled={isLoading}
        class="btn btn-primary btn-sm"
      >
        <RefreshCw size={14} class={isLoading ? "animate-spin" : ""} />
        {isLoading
          ? $lang === "ar"
            ? "تحديث..."
            : "Refreshing…"
          : $lang === "ar"
            ? "تحديث"
            : "Refresh"}
      </button>
    </div>
  </div>

  {#if errorMsg}
    <div
      class="m-6 flex items-center gap-3 px-6 py-4 bg-[var(--danger-dim)] text-[var(--danger)] text-[14px] rounded-[var(--radius-md)] border border-[var(--danger-border)] shadow-md"
    >
      <AlertTriangle size={18} />
      {errorMsg}
    </div>
  {/if}

  <!-- STATS CARDS -->
  <div class="mod-stats">
    <button
      class="mini-stat card {statusFilter === 'all' ? 'stat-active' : ''}"
      onclick={() => (statusFilter = "all")}
    >
      <span class="mini-stat-label"
        >{$lang === "ar" ? "إجمالي الطلبات" : "Total Inquiries"}</span
      >
      <span class="mini-stat-value">{totalCount}</span>
    </button>
    <button
      class="mini-stat card {statusFilter === 'pending' ? 'stat-active' : ''}"
      onclick={() => (statusFilter = "pending")}
    >
      <span class="mini-stat-label"
        >{$lang === "ar" ? "قيد الانتظار" : "Pending"}</span
      >
      <span class="mini-stat-value" style="color:var(--warning)"
        >{pendingCount}</span
      >
    </button>
    <button
      class="mini-stat card {statusFilter === 'contacted' ? 'stat-active' : ''}"
      onclick={() => (statusFilter = "contacted")}
    >
      <span class="mini-stat-label"
        >{$lang === "ar" ? "تم التواصل" : "Contacted"}</span
      >
      <span class="mini-stat-value" style="color:var(--info, #3b82f6)"
        >{contactedCount}</span
      >
    </button>
    <button
      class="mini-stat card {statusFilter === 'resolved' ? 'stat-active' : ''}"
      onclick={() => (statusFilter = "resolved")}
    >
      <span class="mini-stat-label"
        >{$lang === "ar" ? "مكتملة" : "Resolved"}</span
      >
      <span class="mini-stat-value" style="color:var(--success)"
        >{resolvedCount}</span
      >
    </button>
  </div>

  <!-- SEARCH -->
  <div class="search-container">
    <div class="search-box">
      <Search size={15} class="text-[var(--text-ghost)]" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={$lang === "ar"
          ? "البحث بالاسم أو الهاتف أو البريد..."
          : "Search by name, phone, or email…"}
      />
    </div>
  </div>

  <!-- TABLE -->
  {#if filteredInquiries.length === 0}
    <div class="empty-state">
      <Sparkles size={42} style="color:var(--gold); opacity:0.4" />
      <h2>
        {$lang === "ar"
          ? "لا توجد استفسارات"
          : "No Afrah Inquiries Found"}
      </h2>
      <p>
        {$lang === "ar"
          ? "لم يتم تسجيل أي استفسارات من خدمة أفراح بعد."
          : "No wedding planning inquiries have been recorded yet."}
      </p>
    </div>
  {:else}
    <div class="table-container">
      <table class="leads-table">
        <thead>
          <tr>
            <th>{$lang === "ar" ? "الاسم" : "Name"}</th>
            <th>{$lang === "ar" ? "الهاتف" : "Phone"}</th>
            <th>{$lang === "ar" ? "تاريخ الزفاف" : "Wedding Date"}</th>
            <th>{$lang === "ar" ? "الرسالة" : "Message"}</th>
            <th>{$lang === "ar" ? "تم الإرسال في" : "Submitted"}</th>
            <th>{$lang === "ar" ? "الحالة" : "Status"}</th>
            <th class="actions-header"
              >{$lang === "ar" ? "الإجراءات" : "Actions"}</th
            >
          </tr>
        </thead>
        <tbody>
          {#each filteredInquiries as inquiry (inquiry.id)}
            {@const isUpdating = updatingIds.includes(inquiry.id)}
            <tr class="lead-row" class:resolved={inquiry.status === "resolved"}>
              <!-- Name -->
              <td class="name-cell">
                <div class="flex items-center gap-2.5">
                  <div
                    class="avatar-sm"
                    style="background:{getStatusBg(inquiry.status)}"
                  >
                    <User
                      size={14}
                      style="color:{getStatusColor(inquiry.status)}"
                    />
                  </div>
                  <div>
                    <span class="lead-name">{inquiry.name}</span>
                    {#if inquiry.email}
                      <span class="lead-email">{inquiry.email}</span>
                    {/if}
                  </div>
                </div>
              </td>

              <!-- Phone -->
              <td class="phone-cell">
                <div class="flex items-center gap-2">
                  <span class="font-mono text-[13px]">{inquiry.phone}</span>
                  {#if inquiry.isWhatsapp}
                    <span class="wa-badge" title="WhatsApp Available">
                      <svg viewBox="0 0 24 24" class="w-3.5 h-3.5" fill="currentColor">
                        <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347z"/>
                        <path d="M12 0C5.373 0 0 5.373 0 12c0 2.625.846 5.059 2.284 7.034L.789 23.492a.75.75 0 00.917.917l4.458-1.495A11.94 11.94 0 0012 24c6.627 0 12-5.373 12-12S18.627 0 12 0zm0 22c-2.297 0-4.426-.736-6.156-1.987l-.434-.322-3.15 1.056 1.056-3.15-.322-.434A9.96 9.96 0 012 12C2 6.486 6.486 2 12 2s10 4.486 10 10-4.486 10-10 10z"/>
                      </svg>
                    </span>
                  {/if}
                </div>
              </td>

              <!-- Wedding Date -->
              <td>
                <div class="flex items-center gap-1.5">
                  <Calendar size={13} class="text-[var(--text-ghost)]" />
                  <span class="text-[13px] font-[600]"
                    >{formatDate(inquiry.eventDate)}</span
                  >
                </div>
              </td>

              <!-- Message -->
              <td>
                <p class="msg-preview">{inquiry.message || "—"}</p>
              </td>

              <!-- Submitted -->
              <td>
                <span class="text-[12px] text-[var(--text-secondary)]"
                  >{formatDateTime(inquiry.createdAt)}</span
                >
              </td>

              <!-- Status -->
              <td>
                <span
                  class="status-badge"
                  style="background:{getStatusBg(inquiry.status)}; color:{getStatusColor(inquiry.status)}"
                >
                  {#if inquiry.status === "pending"}
                    <Clock size={11} />
                  {:else if inquiry.status === "contacted"}
                    <Phone size={11} />
                  {:else}
                    <CheckCircle2 size={11} />
                  {/if}
                  {inquiry.status}
                </span>
              </td>

              <!-- Actions -->
              <td>
                <div class="flex items-center gap-2">
                  {#if inquiry.status === "pending"}
                    <button
                      class="btn btn-sm btn-outline"
                      disabled={isUpdating}
                      onclick={() => updateStatus(inquiry.id, "contacted")}
                    >
                      {isUpdating ? "..." : $lang === "ar" ? "تم التواصل" : "Mark Contacted"}
                    </button>
                  {:else if inquiry.status === "contacted"}
                    <button
                      class="btn btn-sm btn-success"
                      disabled={isUpdating}
                      onclick={() => updateStatus(inquiry.id, "resolved")}
                    >
                      {isUpdating ? "..." : $lang === "ar" ? "إنهاء" : "Resolve"}
                    </button>
                  {:else}
                    <button
                      class="btn btn-sm btn-ghost"
                      disabled={isUpdating}
                      onclick={() => updateStatus(inquiry.id, "pending")}
                    >
                      {isUpdating ? "..." : $lang === "ar" ? "إعادة فتح" : "Reopen"}
                    </button>
                  {/if}

                  {#if inquiry.isWhatsapp}
                    <button
                      class="btn btn-sm btn-wa"
                      title="Open WhatsApp"
                      onclick={() =>
                        openWhatsApp(inquiry.phone, inquiry.name)}
                    >
                      <ExternalLink size={12} /> WA
                    </button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  /* Stats Grid */
  .mod-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 20px;
  }
  .mini-stat {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all 0.2s ease;
  }
  .mini-stat:hover {
    border-color: var(--glass-border);
  }
  .stat-active {
    border-color: var(--gold) !important;
    box-shadow: 0 0 0 1px var(--gold-glow, rgba(201, 169, 110, 0.2));
  }
  .mini-stat-label {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--text-ghost);
  }
  .mini-stat-value {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.4px;
  }

  /* Search */
  .search-container {
    margin-bottom: 16px;
  }

  /* Empty State */
  .empty-state {
    text-align: center;
    padding: 80px 40px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .empty-state h2 {
    font-size: 16px;
    font-weight: 750;
    color: var(--text-primary);
  }
  .empty-state p {
    font-size: 13.5px;
    color: var(--text-secondary);
    max-width: 360px;
  }

  /* Table */
  .table-container {
    background: var(--glass-sm);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
  .leads-table {
    width: 100%;
    border-collapse: collapse;
  }
  .leads-table thead {
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--glass-border);
  }
  .leads-table th {
    padding: 12px 16px;
    font-size: 10.5px;
    font-weight: 750;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-ghost);
    text-align: start;
    white-space: nowrap;
  }
  .leads-table td {
    padding: 14px 16px;
    border-bottom: 1px solid var(--glass-border);
    vertical-align: middle;
  }
  .lead-row {
    transition: background 0.15s ease;
  }
  .lead-row:hover {
    background: var(--bg-elevated);
  }
  .lead-row.resolved {
    opacity: 0.6;
  }

  .lead-name {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
    display: block;
  }
  .lead-email {
    font-size: 11px;
    color: var(--text-ghost);
    display: block;
    margin-top: 1px;
  }

  .avatar-sm {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .msg-preview {
    font-size: 12.5px;
    color: var(--text-secondary);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    text-transform: capitalize;
    white-space: nowrap;
  }

  .wa-badge {
    display: inline-flex;
    align-items: center;
    color: #25d366;
    flex-shrink: 0;
  }

  .btn-wa {
    background: #25d366 !important;
    color: white !important;
    border: none !important;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 700;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: opacity 0.15s ease;
  }
  .btn-wa:hover {
    opacity: 0.85;
  }

  .btn-success {
    background: var(--success) !important;
    color: white !important;
    border: none !important;
  }
  .btn-ghost {
    background: transparent !important;
    color: var(--text-secondary) !important;
    border: 1px solid var(--glass-border) !important;
  }

  .actions-header {
    text-align: center;
  }

  @media (max-width: 900px) {
    .mod-stats {
      grid-template-columns: repeat(2, 1fr);
    }
    .table-container {
      overflow-x: auto;
    }
    .leads-table {
      min-width: 900px;
    }
  }
</style>
