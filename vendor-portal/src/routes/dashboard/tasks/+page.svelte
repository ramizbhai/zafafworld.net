<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import type { PageData } from './$types';

  // Get SSR-safe translations context
  const i18n = getI18n();

  let { data }: { data: PageData } = $props();

  let categories = $state<any[]>([
    { id: 'general', name_ar: 'عام', name_en: 'General', color: 'badge-unreach' },
    { id: 'decoration', name_ar: 'تزيين', name_en: 'Decoration', color: 'badge-new' },
    { id: 'hospitality', name_ar: 'ضيافة', name_en: 'Hospitality', color: 'badge-done' },
    { id: 'audio', name_ar: 'صوتيات', name_en: 'Audio', color: 'badge-negot' },
    { id: 'finance', name_ar: 'حسابات', name_en: 'Finance', color: 'badge-expired' },
    { id: 'communication', name_ar: 'تواصل', name_en: 'Communication', color: 'badge-unreach' }
  ]);

  $effect(() => {
    if (data.streamed?.metadata) {
      data.streamed.metadata.then((res: any) => {
        if (res?.data?.taskCategories) {
          categories = res.data.taskCategories;
        }
      });
    }
  });

  // Inline form state
  let newTaskTitle = $state('');
  let newTaskCategory = $state('general');

  // Filter state
  let currentFilter = $state<'all' | 'pending' | 'completed'>('all');

  // Helper to parse category from bracketed title
  function parseTask(t: any) {
    const rawTitle = i18n.locale === 'ar' ? t.title_ar : t.title_en;
    if (rawTitle.startsWith('[')) {
      const closingBracketIndex = rawTitle.indexOf(']');
      if (closingBracketIndex !== -1) {
        const category = rawTitle.slice(1, closingBracketIndex);
        const title = rawTitle.slice(closingBracketIndex + 1).trim();
        return { ...t, category, title };
      }
    }
    return { ...t, category: i18n.locale === 'ar' ? 'عام' : 'General', title: rawTitle };
  }

  // Derived tasks from backend data
  let parsedTasks = $derived((data.tasks || []).map(parseTask));

  // Computed progress metrics
  let totalTasks = $derived(parsedTasks.length);
  let completedTasks = $derived(parsedTasks.filter((t: any) => t.is_completed).length);
  let remainingTasks = $derived(totalTasks - completedTasks);
  let progressPercent = $derived(totalTasks > 0 ? Math.round((completedTasks / totalTasks) * 100) : 0);

  // Computed filtered task list
  let filteredTasks = $derived.by(() => {
    let list = parsedTasks;
    if (currentFilter === 'pending') {
      list = list.filter((t: any) => !t.is_completed);
    } else if (currentFilter === 'completed') {
      list = list.filter((t: any) => t.is_completed);
    }
    return list;
  });

  function getCategoryColor(cat: string): string {
    const found = categories.find((c: any) => c.name_ar === cat || c.name_en === cat || c.id === cat);
    return found ? found.color : 'badge-unreach';
  }
</script>

<svelte:head>
  <title>{i18n.t.nav?.tasks || 'Tasks'} - {i18n.t.common?.appName || 'Zafaf Portal'}</title>
</svelte:head>

<div class="tasks-container">
  <!-- Progress Card -->
  <div class="card progress-card">
    <div class="progress-details">
      <div>
        <h3 class="progress-title">{i18n.t.tasks?.progress || 'Progress'}</h3>
        <p class="progress-stats">
          <span>{completedTasks} {i18n.t.tasks?.completed || 'Completed'}</span>
          <span style="margin-inline-start: 16px;">{remainingTasks} {i18n.t.tasks?.remaining || 'Remaining'}</span>
        </p>
      </div>
      <div class="progress-percentage">{progressPercent}%</div>
    </div>
    
    <div class="progress-bar-bg" aria-hidden="true">
      <div class="progress-bar-fill" style="width: {progressPercent}%;"></div>
    </div>
  </div>

  <!-- Main Tasks Card -->
  <div class="card tasks-card">
    <!-- Inline Add Form -->
    <form class="add-task-form" method="POST" action="?/create" use:enhance={() => {
      return async ({ result, update }) => {
        if (result.type === 'success') {
          newTaskTitle = '';
          await invalidateAll();
        }
        await update();
      };
    }}>
      <input
        type="text"
        name="title"
        class="form-input"
        placeholder={i18n.t.tasks?.taskTitlePl || 'What needs to be done?'}
        bind:value={newTaskTitle}
        aria-label={i18n.t.tasks?.taskTitle || 'Task Title'}
        required
      />
      <div style="position: relative; width: 140px; flex-shrink: 0;">
        <select
          name="category"
          class="form-select"
          bind:value={newTaskCategory}
          aria-label={i18n.t.tasks?.category || 'Category'}
        >
          {#each categories as cat}
            <option value={cat.id}>{i18n.locale === 'ar' ? cat.name_ar : cat.name_en}</option>
          {/each}
        </select>
        <span style="position: absolute; left: 12px; top: 50%; transform: translateY(-50%) var(--arrow-rotation); pointer-events: none; color: var(--text-sec); font-size: 9px;">
          ▼
        </span>
      </div>
      <button type="submit" class="btn btn-primary">
        {i18n.t.tasks?.addBtn || 'Add'}
      </button>
    </form>

    <!-- Filters Tab Row -->
    <div class="task-filters-row">
      <button class="filter-tab {currentFilter === 'all' ? 'active' : ''}" onclick={() => currentFilter = 'all'}>
        {i18n.t.common?.filter || 'Filter'} ({totalTasks})
      </button>
      <button class="filter-tab {currentFilter === 'pending' ? 'active' : ''}" onclick={() => currentFilter = 'pending'}>
        {i18n.t.tasks?.remaining || 'Remaining'} ({remainingTasks})
      </button>
      <button class="filter-tab {currentFilter === 'completed' ? 'active' : ''}" onclick={() => currentFilter = 'completed'}>
        {i18n.t.tasks?.completed || 'Completed'} ({completedTasks})
      </button>
    </div>

    <!-- Tasks Checklist Items -->
    {#if filteredTasks.length === 0}
      <div class="empty-state-inner">
        <div class="empty-icon">✅</div>
        <h3>{i18n.t.nav?.tasks || 'Tasks'}</h3>
        <p>{i18n.t.tasks?.empty || 'No tasks found.'}</p>
      </div>
    {:else}
      <div class="tasks-checklist-list">
        {#each filteredTasks as t (t.id)}
          <div class="task-checklist-item {t.is_completed ? 'done-task' : ''}">
            <form method="POST" action="?/toggle" use:enhance={() => {
              return async ({ result, update }) => {
                if (result.type === 'success') await invalidateAll();
                await update();
              };
            }} class="task-check-label-form">
              <input type="hidden" name="id" value={t.id} />
              <input type="hidden" name="title_ar" value={t.title_ar} />
              <input type="hidden" name="title_en" value={t.title_en} />
              <input type="hidden" name="is_completed" value={!t.is_completed} />
              
              <label class="task-check-label">
                <input
                  type="checkbox"
                  checked={t.is_completed}
                  onchange={(e) => e.currentTarget.form?.requestSubmit()}
                  aria-label={t.title}
                />
                <span class="task-checkbox-custom"></span>
                <span class="task-text">{t.title}</span>
              </label>
            </form>

            <div class="task-item-end">
              <span class="badge {getCategoryColor(t.category)}">{t.category}</span>
              
              <form method="POST" action="?/delete" use:enhance={() => {
                return async ({ result, update }) => {
                  if (result.type === 'success') await invalidateAll();
                  await update();
                };
              }}>
                <input type="hidden" name="id" value={t.id} />
                <button
                  type="submit"
                  class="action-btn delete-btn"
                  title={i18n.t.common?.delete || 'Delete'}
                  aria-label={i18n.t.common?.delete || 'Delete'}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  </svg>
                </button>
              </form>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .tasks-container {
    max-width: 800px;
    margin: 0 auto 24px auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .progress-card {
    padding: 24px;
    background: var(--white);
    border-radius: var(--radius);
  }

  .progress-details {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .progress-title {
    font-size: 16px;
    font-weight: 700;
  }

  .progress-stats {
    font-size: 12.5px;
    color: var(--text-sec);
    margin-top: 4px;
  }

  .progress-percentage {
    font-size: 28px;
    font-weight: 800;
    color: var(--teal);
  }

  .progress-bar-bg {
    height: 8px;
    background: var(--border-light);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--teal);
    border-radius: 999px;
    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .tasks-card {
    padding: 24px;
  }

  .add-task-form {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
  }

  .add-task-form input {
    flex: 1;
  }

  .task-filters-row {
    display: flex;
    gap: 16px;
    border-bottom: 1.5px solid var(--border-light);
    margin-bottom: 20px;
  }

  .filter-tab {
    background: none;
    border: none;
    padding: 10px 0;
    font-family: var(--font);
    font-size: 13.5px;
    color: var(--text-sec);
    font-weight: 600;
    cursor: pointer;
    position: relative;
    outline: none;
  }

  .filter-tab:hover {
    color: var(--teal);
  }

  .filter-tab.active {
    color: var(--teal);
  }

  .filter-tab.active::after {
    content: '';
    position: absolute;
    bottom: -1.5px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--teal);
  }

  .tasks-checklist-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .task-checklist-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 18px;
    border-radius: var(--radius-sm);
    background: var(--bg);
    border: 1px solid var(--border-light);
    transition: all 0.15s;
  }

  .task-checklist-item:hover {
    border-color: var(--teal-mid);
    background: var(--white);
  }

  .task-check-label-form {
    flex: 1;
    display: flex;
  }

  .task-check-label {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    flex: 1;
    user-select: none;
  }

  /* Custom styled checkboxes */
  .task-check-label input {
    position: absolute;
    opacity: 0;
    cursor: pointer;
    height: 0;
    width: 0;
  }

  .task-checkbox-custom {
    width: 20px;
    height: 20px;
    border: 2px solid var(--text-light);
    border-radius: 6px;
    display: inline-block;
    position: relative;
    transition: all 0.15s;
    background: var(--white);
  }

  .task-check-label input:checked ~ .task-checkbox-custom {
    background: var(--teal);
    border-color: var(--teal);
  }

  .task-checkbox-custom::after {
    content: '';
    position: absolute;
    display: none;
    left: 6px;
    top: 2px;
    width: 5px;
    height: 10px;
    border: solid white;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .task-check-label input:checked ~ .task-checkbox-custom::after {
    display: block;
  }

  .task-text {
    font-size: 14px;
    color: var(--text);
    transition: color 0.15s, text-decoration 0.15s;
  }

  .done-task .task-text {
    text-decoration: line-through;
    color: var(--text-light);
  }

  .task-item-end {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .empty-state-inner {
    padding: 40px 20px;
    text-align: center;
  }

  @media (max-width: 600px) {
    .add-task-form {
      flex-direction: column;
      gap: 10px;
    }
    .add-task-form div {
      width: 100% !important;
    }
  }
</style>
