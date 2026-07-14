export function createDashboardState(getData: () => any) {
  let selectedBooking = $state<any>(null);
  let cancelling = $state(false);
  let readingAll = $state(false);

  let days = $state(0);
  let hours = $state(0);
  let minutes = $state(0);
  let seconds = $state(0);
  let hasWeddingDate = $state(false);

  // Derived properties from server data
  const profile = $derived(
    getData().profile || {
      weddingDate: null,
      daysRemaining: null,
      budget: { totalBudget: 0, spentAmount: 0, remainingBudget: 0 },
    },
  );

  const weddingCountdown = $derived({
    weddingDate: profile.weddingDate,
    daysRemaining: profile.daysRemaining,
  });

  const budget = $derived(profile.budget);
  const totalBudget = $derived(budget.totalBudget || 0);
  const spentAmount = $derived(budget.spentAmount || 0);
  const remainingBudget = $derived(budget.remainingBudget || 0);

  const budgetPercent = $derived(
    totalBudget > 0
      ? Math.min(100, Math.round((spentAmount / totalBudget) * 100))
      : 0,
  );

  function startCountdown() {
    if (weddingCountdown.weddingDate) {
      hasWeddingDate = true;
      const targetTime = new Date(weddingCountdown.weddingDate).getTime();
      const serverOffset = getData().serverTimestamp
        ? getData().serverTimestamp - Date.now()
        : 0;

      const updateTimer = () => {
        const now = Date.now() + serverOffset;
        const difference = targetTime - now;

        if (difference > 0) {
          days = Math.floor(difference / (1000 * 60 * 60 * 24));
          hours = Math.floor(
            (difference % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
          );
          minutes = Math.floor((difference % (1000 * 60 * 60)) / (1000 * 60));
          seconds = Math.floor((difference % (1000 * 60)) / 1000);
        } else {
          days = 0;
          hours = 0;
          minutes = 0;
          seconds = 0;
        }
      };

      updateTimer();
      const interval = setInterval(updateTimer, 1000);
      return () => clearInterval(interval);
    } else {
      hasWeddingDate = false;
    }
  }

  return {
    get selectedBooking() { return selectedBooking; },
    set selectedBooking(v) { selectedBooking = v; },
    get cancelling() { return cancelling; },
    set cancelling(v) { cancelling = v; },
    get readingAll() { return readingAll; },
    set readingAll(v) { readingAll = v; },

    get days() { return days; },
    get hours() { return hours; },
    get minutes() { return minutes; },
    get seconds() { return seconds; },
    get hasWeddingDate() { return hasWeddingDate; },

    get profile() { return profile; },
    get weddingCountdown() { return weddingCountdown; },
    get totalBudget() { return totalBudget; },
    get spentAmount() { return spentAmount; },
    get remainingBudget() { return remainingBudget; },
    get budgetPercent() { return budgetPercent; },

    startCountdown,
  };
}

export type DashboardState = ReturnType<typeof createDashboardState>;
