import { i18n } from '$lib/i18n.js';

const i18nReroute = i18n.reroute();

export const reroute = (event: { url: URL }) => {
    return i18nReroute(event);
};
