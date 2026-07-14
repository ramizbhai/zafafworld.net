export const uiStore = $state({
    globalLoading: false,
    setLoading(value: boolean) {
        this.globalLoading = value;
    }
});
