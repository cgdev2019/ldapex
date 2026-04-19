import { browser } from '$app/environment';
import { addMessages, getLocaleFromNavigator, init, locale } from 'svelte-i18n';

import en from './messages/en.json';
import fr from './messages/fr.json';

/**
 * Local-storage key used to persist the user's manual locale choice.
 * Defaults to `auto` (follow the navigator locale) when absent.
 */
export const LOCALE_STORAGE_KEY = 'ldapex.locale';

/** Locales currently shipped with the app. */
export const SUPPORTED_LOCALES = ['en', 'fr'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

addMessages('en', en);
addMessages('fr', fr);

function detectLocale(): SupportedLocale {
  if (browser) {
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored && (SUPPORTED_LOCALES as readonly string[]).includes(stored)) {
      return stored as SupportedLocale;
    }
    const nav = getLocaleFromNavigator() ?? window.navigator.language ?? 'en';
    return nav.toLowerCase().startsWith('fr') ? 'fr' : 'en';
  }
  return 'en';
}

/**
 * Initialise svelte-i18n. Safe to call multiple times — the underlying
 * store only reacts to the latest `locale.set()`.
 */
export function setupI18n(): void {
  init({
    fallbackLocale: 'en',
    initialLocale: detectLocale()
  });
}

/**
 * Persist `value` to local storage and switch the active locale.
 * Passing `null` clears the stored override and re-runs detection.
 */
export function setLocale(value: SupportedLocale | null): void {
  if (browser) {
    if (value === null) {
      window.localStorage.removeItem(LOCALE_STORAGE_KEY);
    } else {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, value);
    }
  }
  locale.set(value ?? detectLocale());
}
