<script lang="ts">
  import { enhance } from '$app/forms';
  import { 
    ArrowLeft, MessageSquare, User, Building2, Calendar, Users, MapPin, 
    Clock, ShieldAlert, CheckCircle2, AlertTriangle, Send, Trash2, Tag, ChevronRight
  } from 'lucide-svelte';

  let { data, form } = $props();

  let inquiry = $derived(data.inquiry);
  let client = $derived(data.client);
  let vendor = $derived(data.vendor);
  let listing = $derived(data.listing);
  let city = $derived(data.city);
  let conversation = $derived(data.conversation || []);
  let adminNotes = $derived(data.adminNotes || []);
  let management = $derived(data.management || {});

  let noteInput = $state('');
  let noteTypeInput = $state('internal');

  function formatDate(dateStr: string): string {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      return d.toLocaleString(undefined, { 
        month: 'short', day: 'numeric', year: 'numeric', 
        hour: '2-digit', minute: '2-digit' 
      });
    } catch {
      return dateStr;
    }
  }

  // Unified Chronological Timeline Builder
  interface TimelineEvent {
    id: string;
    timestamp: Date;
    timestampStr: string;
    type: 'submission' | 'message' | 'admin_note' | 'escalated' | 'resolved';
    title: string;
    author?: string;
    content?: string;
    badgeStyle?: string;
  }

  let timelineEvents = $derived(() => {
    const events: TimelineEvent[] = [];

    if (inquiry?.createdAt) {
      events.push({
        id: 'evt-sub',
        timestamp: new Date(inquiry.createdAt),
        timestampStr: formatDate(inquiry.createdAt),
        type: 'submission',
        title: 'Customer Submitted Inquiry',
        author: client?.name || 'Customer',
        content: inquiry.message,
        badgeStyle: 'bg-rose-100 text-rose-800 border-rose-200'
      });
    }

    conversation.forEach((msg: any) => {
      events.push({
        id: `msg-${msg.id}`,
        timestamp: new Date(msg.createdAt),
        timestampStr: formatDate(msg.createdAt),
        type: 'message',
        title: msg.senderRole === 'vendor' ? 'Vendor Replied' : 'Customer Message',
        author: msg.senderName,
        content: msg.body,
        badgeStyle: msg.senderRole === 'vendor' ? 'bg-emerald-100 text-emerald-800 border-emerald-200' : 'bg-blue-100 text-blue-800 border-blue-200'
      });
    });

    adminNotes.forEach((note: any) => {
      events.push({
        id: `note-${note.id}`,
        timestamp: new Date(note.createdAt),
        timestampStr: formatDate(note.createdAt),
        type: 'admin_note',
        title: `Admin Note (${note.noteType})`,
        author: note.adminName,
        content: note.note,
        badgeStyle: 'bg-amber-100 text-amber-900 border-amber-200'
      });
    });

    if (management?.escalatedAt) {
      events.push({
        id: 'evt-esc',
        timestamp: new Date(management.escalatedAt),
        timestampStr: formatDate(management.escalatedAt),
        type: 'escalated',
        title: 'Inquiry Escalated to Management',
        author: management.assignedAdminName || 'Admin System',
        badgeStyle: 'bg-purple-100 text-purple-800 border-purple-200'
      });
    }

    if (management?.resolvedAt) {
      events.push({
        id: 'evt-res',
        timestamp: new Date(management.resolvedAt),
        timestampStr: formatDate(management.resolvedAt),
        type: 'resolved',
        title: 'Inquiry Marked Resolved',
        author: 'Management System',
        badgeStyle: 'bg-emerald-100 text-emerald-800 border-emerald-200'
      });
    }

    return events.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
  });
</script>

<svelte:head>
  <title>Inquiry #{inquiry.id.slice(0, 8)} | ZafafWorld Admin</title>
</svelte:head>

<div class="p-6 max-w-7xl mx-auto space-y-6">
  <!-- Top Nav & Actions -->
  <div class="flex items-center justify-between gap-4">
    <a href="/dashboard/inquiries" class="inline-flex items-center gap-2 text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors">
      <ArrowLeft class="w-4 h-4" />
      Back to System Leads
    </a>

    <div class="flex items-center gap-3">
      <form method="POST" action="?/updateStatus" use:enhance>
        <label for="inquiry-status-select" class="sr-only">Inquiry Status</label>
        <select id="inquiry-status-select" name="status" value={inquiry.status} onchange={(e) => e.currentTarget.form?.requestSubmit()} class="px-3 py-1.5 text-xs font-semibold rounded-lg border border-slate-200 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-rose-500/20">
          <option value="unread">Status: Unread</option>
          <option value="viewed">Status: Viewed</option>
          <option value="pending">Status: Pending</option>
          <option value="replied">Status: Replied</option>
          <option value="closed">Status: Closed</option>
          <option value="declined">Status: Declined</option>
        </select>
      </form>
    </div>
  </div>

  {#if form?.error}
    <div class="p-4 rounded-xl bg-red-50 border border-red-200 text-red-700 text-sm font-medium flex items-center gap-2">
      <AlertTriangle class="w-4 h-4 shrink-0" />
      {form.error}
    </div>
  {/if}

  <!-- Header Card -->
  <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-3 flex-wrap">
        <h1 class="text-xl font-bold text-slate-900">Inquiry ID: #{inquiry.id.slice(0, 8)}</h1>
        {#if inquiry.isGuest}
          <span class="px-2.5 py-0.5 text-xs font-semibold bg-slate-100 text-slate-700 rounded-full">Guest Submission</span>
        {:else}
          <span class="px-2.5 py-0.5 text-xs font-semibold bg-blue-50 text-blue-700 rounded-full">Authenticated Client</span>
        {/if}

        {#if management.priority === 'critical'}
          <span class="px-2.5 py-0.5 text-xs font-bold bg-red-100 text-red-800 rounded-full">Critical Priority</span>
        {:else if management.priority === 'high'}
          <span class="px-2.5 py-0.5 text-xs font-bold bg-orange-100 text-orange-800 rounded-full">High Priority</span>
        {/if}
      </div>
      <p class="text-xs text-slate-500 mt-1.5">Submitted on {formatDate(inquiry.createdAt)}</p>
    </div>
  </div>

  <!-- Main Grid -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Column 1: Metadata & Entities -->
    <div class="space-y-6">
      <!-- Customer Card -->
      <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-4">
        <h3 class="text-sm font-bold text-slate-900 uppercase tracking-wider flex items-center gap-2">
          <User class="w-4 h-4 text-rose-600" /> Customer Information
        </h3>
        <div class="space-y-2.5 text-sm">
          <div>
            <div class="text-xs text-slate-400">Full Name</div>
            <div class="font-semibold text-slate-900">{client.name}</div>
          </div>
          <div>
            <div class="text-xs text-slate-400">Phone Number</div>
            <div class="font-medium text-slate-800">{client.phone || 'Not provided'}</div>
          </div>
          <div>
            <div class="text-xs text-slate-400">Email Address</div>
            <div class="font-medium text-slate-800 truncate">{client.email || 'Not provided'}</div>
          </div>
        </div>
      </div>

      <!-- Vendor Card -->
      <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-4">
        <h3 class="text-sm font-bold text-slate-900 uppercase tracking-wider flex items-center gap-2">
          <Building2 class="w-4 h-4 text-rose-600" /> Vendor & Listing
        </h3>
        <div class="space-y-3 text-sm">
          <div>
            <div class="text-xs text-slate-400">Vendor Name</div>
            <div class="font-semibold text-slate-900">{vendor.nameEn} ({vendor.nameAr})</div>
          </div>
          {#if listing}
            <div>
              <div class="text-xs text-slate-400">Target Listing</div>
              <div class="font-semibold text-rose-600">{listing.titleEn || listing.titleAr}</div>
            </div>
          {:else}
            <div>
              <div class="text-xs text-slate-400">Scope</div>
              <div class="text-slate-500 font-medium">General Vendor Inquiry</div>
            </div>
          {/if}
          {#if city}
            <div>
              <div class="text-xs text-slate-400">Target City</div>
              <div class="font-medium text-slate-800">{city.nameEn} ({city.nameAr})</div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Event Info Card -->
      <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-4">
        <h3 class="text-sm font-bold text-slate-900 uppercase tracking-wider flex items-center gap-2">
          <Calendar class="w-4 h-4 text-rose-600" /> Event Specifications
        </h3>
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <div class="text-xs text-slate-400">Requested Event Date</div>
            <div class="font-bold text-slate-900 mt-0.5">{formatDate(inquiry.eventDate)}</div>
          </div>
          <div>
            <div class="text-xs text-slate-400">Guest Count</div>
            <div class="font-bold text-slate-900 mt-0.5">{inquiry.guestCount} Guests</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Column 2: Unified Chronological Activity Timeline -->
    <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-6">
      <h2 class="text-base font-bold text-slate-900 flex items-center gap-2 border-b border-slate-100 pb-4">
        <Clock class="w-5 h-5 text-rose-600" /> Unified Chronological Timeline
      </h2>

      <div class="relative pl-6 space-y-6 before:absolute before:left-2.5 before:top-2 before:bottom-2 before:w-0.5 before:bg-slate-200">
        {#each timelineEvents() as event (event.id)}
          <div class="relative group">
            <span class="absolute -left-6 top-1.5 w-3.5 h-3.5 rounded-full border-2 border-white bg-slate-400 ring-2 ring-slate-100"></span>
            
            <div class="bg-slate-50 p-4 rounded-xl border border-slate-200/80 space-y-2">
              <div class="flex items-center justify-between gap-2">
                <span class="px-2 py-0.5 text-xs font-semibold rounded border {event.badgeStyle}">
                  {event.title}
                </span>
                <span class="text-xs text-slate-400 font-medium">{event.timestampStr}</span>
              </div>
              
              {#if event.author}
                <div class="text-xs font-semibold text-slate-700">Author: {event.author}</div>
              {/if}
              
              {#if event.content}
                <p class="text-sm text-slate-800 whitespace-pre-wrap leading-relaxed pt-1 border-t border-slate-200/50 mt-2">
                  {event.content}
                </p>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Column 3: Management Controls & Internal Notes -->
    <div class="space-y-6">
      <!-- CRM Management State -->
      <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4">
        <h3 class="text-sm font-bold text-slate-900 uppercase tracking-wider flex items-center gap-2">
          <ShieldAlert class="w-4 h-4 text-purple-600" /> CRM & Support Management
        </h3>

        <form method="POST" action="?/updateManagement" use:enhance class="space-y-3.5">
          <div>
            <label for="priority-level" class="block text-xs font-semibold text-slate-600 mb-1">Priority Level</label>
            <select id="priority-level" name="priority" value={management.priority || 'medium'} class="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:ring-2 focus:ring-rose-500/20">
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
              <option value="critical">Critical</option>
            </select>
          </div>

          <div>
            <label for="escalation-status" class="block text-xs font-semibold text-slate-600 mb-1">Escalation Status</label>
            <select id="escalation-status" name="escalation_status" value={management.escalationStatus || 'none'} class="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:ring-2 focus:ring-rose-500/20">
              <option value="none">None (Standard Workflow)</option>
              <option value="pending">Escalation Pending</option>
              <option value="escalated">Escalated to Support Team</option>
              <option value="resolved">Escalation Resolved</option>
            </select>
          </div>

          <div>
            <label for="resolution-status" class="block text-xs font-semibold text-slate-600 mb-1">Resolution Status</label>
            <select id="resolution-status" name="resolution_status" value={management.resolutionStatus || 'unresolved'} class="w-full px-3 py-2 text-sm border border-slate-200 rounded-lg bg-white focus:ring-2 focus:ring-rose-500/20">
              <option value="unresolved">Unresolved</option>
              <option value="in_progress">In Progress</option>
              <option value="resolved">Resolved</option>
              <option value="closed_no_action">Closed (No Action Needed)</option>
            </select>
          </div>

          <button type="submit" class="w-full py-2 px-4 text-sm font-semibold bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors shadow-sm">
            Save CRM State
          </button>
        </form>
      </div>

      <!-- Internal Admin Notes Manager -->
      <div class="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm space-y-4">
        <h3 class="text-sm font-bold text-slate-900 uppercase tracking-wider flex items-center gap-2">
          <Tag class="w-4 h-4 text-amber-600" /> Admin Internal Notes
        </h3>

        <!-- Add Note Form -->
        <form method="POST" action="?/addNote" use:enhance={() => {
          return async ({ result, update }) => {
            if (result.type === 'success') noteInput = '';
            await update();
          };
        }} class="space-y-3">
          <div>
            <label for="admin-note-text" class="sr-only">Internal Note</label>
            <textarea 
              id="admin-note-text"
              name="note" 
              bind:value={noteInput} 
              rows="3" 
              placeholder="Add internal note or follow-up details..." 
              class="w-full p-3 text-sm border border-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-rose-500/20 focus:border-rose-500"
            ></textarea>
          </div>

          <div class="flex items-center gap-2">
            <label for="admin-note-type" class="sr-only">Note Type</label>
            <select id="admin-note-type" name="note_type" bind:value={noteTypeInput} class="px-3 py-1.5 text-xs border border-slate-200 rounded-lg bg-white">
              <option value="internal">Internal Note</option>
              <option value="customer_followup">Customer Follow-up</option>
              <option value="vendor_followup">Vendor Follow-up</option>
              <option value="escalation">Escalation Note</option>
              <option value="resolution">Resolution Summary</option>
            </select>

            <button type="submit" disabled={!noteInput.trim()} class="ml-auto px-3.5 py-1.5 text-xs font-semibold bg-amber-600 text-white rounded-lg hover:bg-amber-700 disabled:opacity-50 transition-colors flex items-center gap-1.5">
              <Send class="w-3.5 h-3.5" /> Post Note
            </button>
          </div>
        </form>

        <!-- Notes List -->
        <div class="space-y-3 pt-2">
          {#each adminNotes as note (note.id)}
            <div class="p-3.5 rounded-xl bg-amber-50/50 border border-amber-200/70 space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-amber-900">{note.adminName}</span>
                <div class="flex items-center gap-2">
                  <span class="text-[10px] text-amber-700 font-medium">{formatDate(note.createdAt)}</span>
                  <form method="POST" action="?/deleteNote" use:enhance>
                    <input type="hidden" name="note_id" value={note.id} />
                    <button type="submit" class="text-amber-400 hover:text-red-600 transition-colors p-0.5">
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </form>
                </div>
              </div>
              <p class="text-xs text-slate-800 whitespace-pre-wrap">{note.note}</p>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>
