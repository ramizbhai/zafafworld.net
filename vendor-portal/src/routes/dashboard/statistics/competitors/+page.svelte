<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';
  import type { Competitor } from '$lib/types';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  const i18n = getI18n();

  // Table 1 sorting states
  let citySortField = $state<keyof Competitor>('rank');
  let citySortAsc = $state(true);

  // Table 2 sorting states
  let serviceSortField = $state<keyof Competitor>('rank');
  let serviceSortAsc = $state(true);

  // Computed sorted City Competitors
  let sortedCityComps = $derived.by(() => {
    let list = [...data.competitorsCity];
    list.sort((a, b) => {
      let valA = a[citySortField];
      let valB = b[citySortField];

      if (typeof valA === 'string' && typeof valB === 'string') {
        return citySortAsc ? valA.localeCompare(valB, i18n.locale) : valB.localeCompare(valA, i18n.locale);
      } else {
        return citySortAsc
          ? (valA > valB ? 1 : -1)
          : (valB > valA ? 1 : -1);
      }
    });
    return list;
  });

  // Computed sorted Service Competitors
  let sortedServiceComps = $derived.by(() => {
    let list = [...data.competitorsService];
    list.sort((a, b) => {
      let valA = a[serviceSortField];
      let valB = b[serviceSortField];

      if (typeof valA === 'string' && typeof valB === 'string') {
        return serviceSortAsc ? valA.localeCompare(valB, i18n.locale) : valB.localeCompare(valA, i18n.locale);
      } else {
        return serviceSortAsc
          ? (valA > valB ? 1 : -1)
          : (valB > valA ? 1 : -1);
      }
    });
    return list;
  });

  function toggleCitySort(field: keyof Competitor) {
    if (citySortField === field) {
      citySortAsc = !citySortAsc;
    } else {
      citySortField = field;
      citySortAsc = true;
    }
  }

  function toggleServiceSort(field: keyof Competitor) {
    if (serviceSortField === field) {
      serviceSortAsc = !serviceSortAsc;
    } else {
      serviceSortField = field;
      serviceSortAsc = true;
    }
  }

  // Helper check for own page highlight
  function isOwnPage(pageName: string) {
    return pageName === 'قاعة الأفراح الملكية';
  }
</script>

<svelte:head>
  <title>{i18n.t.nav.statisticsCompetitors} - {i18n.t.common.appName}</title>
</svelte:head>

<div class="competitors-page" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
  <div class="top-bar-row">
    <span class="toolbar-title">{i18n.t.nav.statisticsCompetitors}</span>
    <button class="btn btn-ghost btn-sm" onclick={() => alert(i18n.t.common.loading)} title={i18n.locale === 'ar' ? 'تحديث البيانات' : 'Refresh Data'}>
      🔄 {i18n.locale === 'ar' ? 'تحديث' : 'Refresh'}
    </button>
  </div>

  <!-- Same city Competitors -->
  <div class="table-card" style="margin-bottom:24px">
    <div class="section-title">{i18n.t.statistics.competitorsCityTitle}</div>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th onclick={() => toggleCitySort('rank')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.rank}
              <span class="sort-icon">{citySortField === 'rank' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('page')} style="cursor:pointer;" role="columnheader">
              {i18n.t.couples.pageName}
              <span class="sort-icon">{citySortField === 'page' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('city')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.city}
              <span class="sort-icon">{citySortField === 'city' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('district')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.district}
              <span class="sort-icon">{citySortField === 'district' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('reviews')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.reviews}
              <span class="sort-icon">{citySortField === 'reviews' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('couples')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.coupleInquiries}
              <span class="sort-icon">{citySortField === 'couples' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('conversion')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.conversionRate}
              <span class="sort-icon">{citySortField === 'conversion' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleCitySort('calls')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.phoneCalls}
              <span class="sort-icon">{citySortField === 'calls' ? (citySortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {#each sortedCityComps as row (row.page)}
            <tr class:highlight={isOwnPage(row.page)}>
              <td><span class="rank-badge" class:rank-1={row.rank===1}>{row.rank}</span></td>
              <td class:own={isOwnPage(row.page)}>
                {row.page} {isOwnPage(row.page) ? `(${i18n.locale === 'ar' ? 'أنت' : 'You'})` : ''}
              </td>
              <td>{row.city}</td>
              <td>{row.district}</td>
              <td>{row.reviews}</td>
              <td>{row.couples}</td>
              <td>{row.conversion}%</td>
              <td>{row.calls}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Same service type Competitors -->
  <div class="table-card">
    <div class="section-title">{i18n.t.statistics.competitorsServiceTitle}</div>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th onclick={() => toggleServiceSort('rank')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.rank}
              <span class="sort-icon">{serviceSortField === 'rank' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('page')} style="cursor:pointer;" role="columnheader">
              {i18n.t.couples.pageName}
              <span class="sort-icon">{serviceSortField === 'page' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('visits')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.visits}
              <span class="sort-icon">{serviceSortField === 'visits' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('couples')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.coupleInquiries}
              <span class="sort-icon">{serviceSortField === 'couples' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('convRate')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.convRate}
              <span class="sort-icon">{serviceSortField === 'convRate' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('avgReply')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.avgReply}
              <span class="sort-icon">{serviceSortField === 'avgReply' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('comments')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.comments}
              <span class="sort-icon">{serviceSortField === 'comments' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('offers')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.offers}
              <span class="sort-icon">{serviceSortField === 'offers' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
            <th onclick={() => toggleServiceSort('quality')} style="cursor:pointer;" role="columnheader">
              {i18n.t.statistics.qualityPoints}
              <span class="sort-icon">{serviceSortField === 'quality' ? (serviceSortAsc ? '▲' : '▼') : '⇅'}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {#each sortedServiceComps as row (row.page)}
            <tr class:highlight={isOwnPage(row.page)}>
              <td><span class="rank-badge" class:rank-1={row.rank===1}>{row.rank}</span></td>
              <td class:own={isOwnPage(row.page)}>
                {row.page} {isOwnPage(row.page) ? `(${i18n.locale === 'ar' ? 'أنت' : 'You'})` : ''}
              </td>
              <td>{row.visits}</td>
              <td>{row.couples}</td>
              <td>{row.convRate}</td>
              <td>{row.avgReply}m</td>
              <td>{row.comments}</td>
              <td>{row.offers}</td>
              <td>{row.quality}%</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .competitors-page {
    animation: fadeIn 0.3s ease-out;
  }
  .top-bar-row {
    display: flex; justify-content: space-between; align-items: center;
    margin-bottom: 20px; background: var(--white); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 12px 16px; box-shadow: var(--shadow);
  }
  .toolbar-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }
  .section-title {
    font-size: 15px; font-weight: 700; color: var(--text);
    padding: 14px 20px; border-bottom: 1px solid var(--border-light);
    text-align: var(--text-align);
  }
  .sort-icon { color: var(--text-light); font-size: 11px; margin-inline-start: 4px; }
  
  .rank-badge {
    display: inline-flex; align-items: center; justify-content: center;
    width: 24px; height: 24px; border-radius: 50%;
    font-size: 12px; font-weight: 700; background: var(--bg); color: var(--text-sec);
  }
  .rank-badge.rank-1 { background: var(--teal); color: #fff; }
  tr.highlight td { background: #f0faf6; }
  td.own { font-weight: 700; color: var(--teal-dark); }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
</style>
