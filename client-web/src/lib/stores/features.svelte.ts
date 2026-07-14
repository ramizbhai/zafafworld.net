import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';

export interface FeatureItem {
  id: string;
  nameAr: string;
  nameEn: string;
  category: string;
  inputType: string;
}

const API_BASE = `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1`;

function createFeaturesStore() {
  let featuresMap = $state<Record<string, FeatureItem>>({});
  let loaded = $state(false);

  async function loadFeatures() {
    if (!browser) return;
    try {
      const res = await fetch(`${API_BASE}/features`);
      if (res.ok) {
        const json = await res.json();
        if (json.status === 'success' && Array.isArray(json.data)) {
          const map: Record<string, FeatureItem> = {};
          for (const item of json.data) {
            map[item.id] = item;
          }
          featuresMap = map;
          loaded = true;
        }
      }
    } catch (e) {
      console.warn('Failed to load master features dictionary:', e);
    }
  }

  if (browser) {
    loadFeatures();
  }

  return {
    get map() { return featuresMap; },
    get loaded() { return loaded; },
    loadFeatures
  };
}

export const featuresStore = createFeaturesStore();
