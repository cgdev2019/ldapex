<script lang="ts">
  import { theme, type ThemeMode } from '$lib/theme.svelte';
  import Icon from './Icon.svelte';
  import type { IconName } from './Icon.svelte';

  const ORDER: ThemeMode[] = ['auto', 'light', 'dark'];

  function next() {
    const current = theme.mode;
    const idx = ORDER.indexOf(current);
    theme.set(ORDER[(idx + 1) % ORDER.length]);
  }

  function icon(mode: ThemeMode): IconName {
    if (mode === 'light') return 'eye';
    if (mode === 'dark') return 'eye-off';
    return 'globe';
  }

  function label(mode: ThemeMode): string {
    if (mode === 'light') return 'Clair';
    if (mode === 'dark') return 'Sombre';
    return 'Auto';
  }
</script>

<button
  type="button"
  class="ghost theme-toggle"
  onclick={next}
  title="Thème — clic pour cycler (auto / clair / sombre)"
  aria-label={`Thème: ${label(theme.mode)}`}
>
  <Icon name={icon(theme.mode)} size={14} />
  <span class="lbl">{label(theme.mode)}</span>
</button>

<style>
  .theme-toggle {
    color: var(--color-text-muted);
    padding-left: 0.55rem;
    padding-right: 0.55rem;
  }

  .lbl {
    font-size: 0.75rem;
  }
</style>
