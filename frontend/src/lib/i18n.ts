import i18next from 'i18next';
import { derived, writable } from 'svelte/store';
import en from './locales/en.json';
import de from './locales/de.json';
import deCool from './locales/de-cool.json';

/** Separate catalogs: German (cool) is deliberately editable independently. */
export const languages = [
  { id: 'en', label: 'English' },
  { id: 'de', label: 'Deutsch' },
  { id: 'de-cool', label: 'Deutsch (cool)' },
] as const;
export type Language = (typeof languages)[number]['id'];
const storageKey = 'arena-language';

function isLanguage(value: string | null): value is Language {
  return languages.some(({ id }) => id === value);
}

/** First supported device language wins; unsupported languages fall back to English. */
function initialLanguage(): Language {
  try {
    const saved = localStorage.getItem(storageKey);
    if (isLanguage(saved)) return saved;
  } catch {
    // Device preferences still work when browser storage is unavailable.
  }
  for (const language of navigator.languages) {
    const base = language.toLowerCase().split('-')[0];
    if (base === 'de' || base === 'en') return base;
  }
  return 'en';
}

export const locale = writable<Language>(initialLanguage());
// Catalogs are bundled; initialization requires no network or async loader.
void i18next.init({
  resources: {
    en: { translation: en },
    de: { translation: de },
    'de-cool': { translation: deCool },
  },
  fallbackLng: 'en',
  initAsync: false,
  interpolation: { escapeValue: false, prefix: '{', suffix: '}' },
});
/** Reactive fixed-language translator; Svelte escapes the resulting text. */
export const t = derived(locale, (language) => {
  const translate = i18next.getFixedT(language);
  return (key: string, options?: { values: Record<string, string> }): string =>
    translate(key, options?.values ?? {});
});
locale.subscribe((language) => {
  document.documentElement.lang = language === 'de-cool' ? 'de' : language;
});

/** Apply immediately and remember an explicit selection on this device. */
export function setLanguage(language: string): void {
  if (!isLanguage(language)) return;
  locale.set(language);
  try {
    localStorage.setItem(storageKey, language);
  } catch {
    // The selection remains usable for the current visit.
  }
}
