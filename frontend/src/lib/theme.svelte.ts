/**
 * Theme store. Three modes: `'auto'` follows the OS, `'light'` and
 * `'dark'` force the corresponding scheme. Choice is persisted to
 * `localStorage` and applied via a `data-theme` attribute on the
 * document root, which our CSS reads to pin `color-scheme` (so the
 * `light-dark()` palette in `app.css` follows along).
 */

import { browser } from '$app/environment';

export type ThemeMode = 'auto' | 'light' | 'dark';
export type Density = 'compact' | 'normal' | 'comfortable';

const KEY = 'ldapex.theme';
const DENSITY_KEY = 'ldapex.density';

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

function readDensity(): Density {
  if (!browser) return 'normal';
  const v = window.localStorage.getItem(DENSITY_KEY);
  return v === 'compact' || v === 'comfortable' ? v : 'normal';
}

function applyDensity(d: Density): void {
  if (!browser) return;
  const root = document.documentElement;
  if (d === 'normal') root.removeAttribute('data-density');
  else root.setAttribute('data-density', d);
}

class ThemeStore {
  mode = $state<ThemeMode>(readStored());
  density = $state<Density>(readDensity());

  init(): void {
    applyToDom(this.mode);
    applyDensity(this.density);
  }

  set(next: ThemeMode): void {
    this.mode = next;
    if (browser) {
      if (next === 'auto') window.localStorage.removeItem(KEY);
      else window.localStorage.setItem(KEY, next);
    }
    applyToDom(next);
  }

  setDensity(next: Density): void {
    this.density = next;
    if (browser) {
      if (next === 'normal') window.localStorage.removeItem(DENSITY_KEY);
      else window.localStorage.setItem(DENSITY_KEY, next);
    }
    applyDensity(next);
  }
}

export const theme = new ThemeStore();
