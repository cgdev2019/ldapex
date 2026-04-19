/**
 * Theme store. Three modes: `'auto'` follows the OS, `'light'` and
 * `'dark'` force the corresponding scheme. Choice is persisted to
 * `localStorage` and applied via a `data-theme` attribute on the
 * document root, which our CSS reads to pin `color-scheme` (so the
 * `light-dark()` palette in `app.css` follows along).
 */

import { browser } from '$app/environment';

export type ThemeMode = 'auto' | 'light' | 'dark';

const KEY = 'ldapex.theme';

function readStored(): ThemeMode {
  if (!browser) return 'auto';
  const v = window.localStorage.getItem(KEY);
  return v === 'light' || v === 'dark' ? v : 'auto';
}

function applyToDom(mode: ThemeMode): void {
  if (!browser) return;
  const root = document.documentElement;
  if (mode === 'auto') root.removeAttribute('data-theme');
  else root.setAttribute('data-theme', mode);
}

class ThemeStore {
  mode = $state<ThemeMode>(readStored());

  init(): void {
    applyToDom(this.mode);
  }

  set(next: ThemeMode): void {
    this.mode = next;
    if (browser) {
      if (next === 'auto') window.localStorage.removeItem(KEY);
      else window.localStorage.setItem(KEY, next);
    }
    applyToDom(next);
  }
}

export const theme = new ThemeStore();
