<script lang="ts">
  import { theme, type Density } from '$lib/theme.svelte';
  import Icon from './Icon.svelte';

  const ORDER: Density[] = ['compact', 'normal', 'comfortable'];

  function next() {
    const i = ORDER.indexOf(theme.density);
    theme.setDensity(ORDER[(i + 1) % ORDER.length]);
  }

  function label(d: Density): string {
    if (d === 'compact') return 'Compact';
    if (d === 'comfortable') return 'Aéré';
    return 'Normal';
  }
</script>

<button
  type="button"
  class="ghost density-toggle"
  onclick={next}
  title="Densité — clic pour cycler"
  aria-label={`Densité: ${label(theme.density)}`}
>
  <Icon name="list" size={14} />
  <span class="lbl">{label(theme.density)}</span>
</button>

<style>
  .density-toggle {
    color: var(--color-text-muted);
    padding-left: 0.55rem;
    padding-right: 0.55rem;
  }

  .lbl {
    font-size: 0.75rem;
  }
</style>
