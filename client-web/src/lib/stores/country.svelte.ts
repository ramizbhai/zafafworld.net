import { browser } from '$app/environment';

export interface CountryInfo {
  code: string;
  emoji: string;
  nameAr: string;
  nameEn: string;
  eyebrowAr: string;
  eyebrowEn: string;

}

export const COUNTRIES: Record<string, CountryInfo> = {
  SA: {
    code: 'SA',
    emoji: '🇸🇦',
    nameAr: 'المملكة العربية السعودية',
    nameEn: 'Saudi Arabia',
    eyebrowAr: 'منصة حجز القاعات #1 في المملكة',
    eyebrowEn: "Saudi Arabia's No.1 Venue Booking Platform",

  },
  EG: {
    code: 'EG',
    emoji: '🇪🇬',
    nameAr: 'جمهورية مصر العربية',
    nameEn: 'Egypt',
    eyebrowAr: 'منصة حجز القاعات #1 في مصر',
    eyebrowEn: "Egypt's No.1 Venue Booking Platform",

  },
  AE: {
    code: 'AE',
    emoji: '🇦🇪',
    nameAr: 'الإمارات العربية المتحدة',
    nameEn: 'United Arab Emirates',
    eyebrowAr: 'منصة حجز القاعات #1 في الإمارات',
    eyebrowEn: "UAE's No.1 Venue Booking Platform",

  },
  TR: {
    code: 'TR',
    emoji: '🇹🇷',
    nameAr: 'الجمهورية التركية',
    nameEn: 'Turkey',
    eyebrowAr: 'منصة حجز القاعات #1 في تركيا',
    eyebrowEn: "Turkey's No.1 Venue Booking Platform",

  }
};

const STORAGE_KEY = 'zafaf_selected_country';
const DEFAULT_COUNTRY = 'SA';

function getCookie(name: string): string | null {
  if (!browser) return null;
  const nameEQ = name + "=";
  const ca = document.cookie.split(';');
  for (let i = 0; i < ca.length; i++) {
    let c = ca[i];
    while (c.charAt(0) === ' ') c = c.substring(1, c.length);
    if (c.indexOf(nameEQ) === 0) return c.substring(nameEQ.length, c.length);
  }
  return null;
}

function createCountryStore() {
  let initialCountry: string | null = null;
  if (browser) {
    const urlParams = new URLSearchParams(window.location.search);
    const urlCountry = urlParams.get('country');
    if (urlCountry && COUNTRIES[urlCountry.toUpperCase()]) {
      initialCountry = urlCountry.toUpperCase();
      // Store preferred country from URL parameters without reloading
      localStorage.setItem(STORAGE_KEY, initialCountry);
      document.cookie = `${STORAGE_KEY}=${initialCountry}; path=/; max-age=31536000; SameSite=Lax`;
    } else {
      initialCountry = localStorage.getItem(STORAGE_KEY) || getCookie(STORAGE_KEY);
    }
  }

  let activeCode = $state<string>(initialCountry ?? DEFAULT_COUNTRY);

  const active = $derived(COUNTRIES[activeCode] ?? COUNTRIES[DEFAULT_COUNTRY]);

  function setCountry(code: string) {
    if (COUNTRIES[code]) {
      activeCode = code;
      if (browser) {
        localStorage.setItem(STORAGE_KEY, code);
        document.cookie = `${STORAGE_KEY}=${code}; path=/; max-age=31536000; SameSite=Lax`;
        // Reload to apply new country metadata and reload page data
        window.location.reload();
      }
    }
  }

  function setCountryFromUrl(code: string) {
    const upperCode = code.toUpperCase();
    if (COUNTRIES[upperCode] && activeCode !== upperCode) {
      activeCode = upperCode;
      if (browser) {
        localStorage.setItem(STORAGE_KEY, upperCode);
        document.cookie = `${STORAGE_KEY}=${upperCode}; path=/; max-age=31536000; SameSite=Lax`;
      }
    }
  }

  // IP-based Location Auto-Detection
  async function detectLocation() {
    if (!browser) return;
    
    // Only auto-detect if the user hasn't explicitly set a preference
    if (localStorage.getItem(STORAGE_KEY)) return;

    try {
      const response = await fetch('https://ipapi.co/json/');
      if (response.ok) {
        const data = await response.json();
        const detectedCountry = data.country_code; // e.g. "EG", "AE", etc.
        if (detectedCountry && COUNTRIES[detectedCountry]) {
          setCountry(detectedCountry);
        }
      }
    } catch (e) {
      console.warn('Failed to auto-detect country from IP, falling back to default:', e);
    }
  }

  // Trigger detection on store creation if in browser
  if (browser) {
    detectLocation();
  }

  return {
    get active() { return active; },
    get activeCode() { return activeCode; },
    setCountry,
    setCountryFromUrl,
    detectLocation
  };
}

export const countryStore = createCountryStore();
