<script lang="ts">
  import { formatError } from '$lib/bridge';
  import { history } from '$lib/history.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    onclose: () => void;
    onchanged?: () => void;
  }

  let { onclose, onchanged }: Props = $props();

  let busyId = $state<string | null>(null);
  let error = $state<string | null>(null);
  let confirmClear = $state(false);

  $effect(() => {
    history.reload();
  });

  async function undoItem(id: string) {
    busyId = id;
    error = null;
    try {
      await history.undo(id);
      onchanged?.();
    } catch (err) {
      error = formatError(err);
    } finally {
      busyId = null;
    }
  }

  function fmtTime(ts: number): string {
    const d = new Date(ts);
    const ago = Math.round((Date.now() - ts) / 1000);
    if (ago < 60) return `il y a ${ago} s`;
    if (ago < 3600) return `il y a ${Math.round(ago / 60)} min`;
    if (ago < 86400) return `il y a ${Math.round(ago / 3600)} h`;
    return d.toLocaleString();
  }

  function opColor(op: string): string {
    if (op === 'add') return 'add';
    if (op === 'delete') return 'delete';
    if (op === 'rename') return 'rename';
    return 'modify';
  }

  function close() {
    onclose();
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label="Historique" onclick={close} tabindex="-1">
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header>
      <h2>
        <Icon name="clock" size={16} />
        <span>Historique</span>
        <span class="count">{history.items.length}</span>
      </h2>
      <div class="spacer"></div>
      {#if history.items.length > 0}
        <button type="button" class="ghost sm" onclick={() => (confirmClear = true)}>
          Effacer
        </button>
      {/if}
      <button type="button" class="ghost icon-only" onclick={close}>
        <Icon name="x" size={14} />
      </button>
    </header>

    {#if error}
      <p class="status error">{error}</p>
    {/if}

    {#if confirmClear}
      <div class="confirm">
        <span>Effacer tout l'historique de ce profil ?</span>
        <button
          type="button"
          class="ghost sm"
          onclick={() => {
            history.clear();
            confirmClear = false;
          }}>Oui</button>
        <button type="button" class="ghost sm" onclick={() => (confirmClear = false)}>Non</button>
      </div>
    {/if}

    {#if history.items.length === 0}
      <div class="empty">
        <Icon name="info" size={22} />
        <p>Pas encore d'opération enregistrée.</p>
        <p class="muted">
          Les créations, modifications, suppressions et renommages apparaissent ici
          avec un bouton « Annuler ».
        </p>
      </div>
    {:else}
      <ul class="items">
        {#each history.items as e (e.id)}
          <li class:undone={e.undone}>
            <span class="badge" data-op={opColor(e.op)}>{e.op}</span>
            <div class="texts">
              <div class="line1">
                <code class="dn" title={e.dn}>{e.dn}</code>
                <span class="when">{fmtTime(e.ts)}</span>
              </div>
              <div class="line2">{e.label}</div>
            </div>
            <button
              type="button"
              class="ghost sm"
              onclick={() => undoItem(e.id)}
              disabled={busyId !== null || e.undone}
              title={e.undone ? 'Déjà annulée' : 'Rejouer l\u2019inverse'}
            >
              <Icon name="refresh" size={11} />
              <span>{e.undone ? 'Annulée' : 'Annuler'}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 55%, transparent);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    z-index: 210;
    padding: 1rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    width: min(92vw, 40rem);
    max-height: 86vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.85rem 1.1rem;
    border-bottom: 1px solid var(--color-border);
    gap: 0.4rem;
  }

  header h2 {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
  }

  .count {
    font-size: 0.62rem;
    padding: 0.05rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
    color: white;
    font-weight: 600;
  }

  .spacer {
    flex: 1;
  }

  .confirm {
    padding: 0.5rem 1rem;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    background: color-mix(in oklab, var(--color-warning) 35%, transparent);
    color: var(--color-text);
    font-size: var(--text-sm);
  }

  .items {
    list-style: none;
    margin: 0;
    padding: 0.4rem 0.5rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .items li {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
  }

  .items li.undone {
    opacity: 0.55;
  }

  .badge {
    font-size: 0.58rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 0.05rem 0.45rem;
    border-radius: var(--radius-pill);
    font-weight: 600;
    background: var(--color-surface-2);
    flex-shrink: 0;
  }

  .badge[data-op='add'] {
    background: color-mix(in oklab, var(--color-success) 18%, transparent);
    color: var(--color-success);
  }

  .badge[data-op='modify'] {
    background: var(--color-primary-soft);
    color: var(--color-primary);
  }

  .badge[data-op='delete'] {
    background: color-mix(in oklab, var(--color-danger) 15%, transparent);
    color: var(--color-danger);
  }

  .badge[data-op='rename'] {
    background: color-mix(in oklab, var(--color-warning) 18%, transparent);
    color: var(--color-warning);
  }

  .texts {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .line1 {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }

  .dn {
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .when {
    font-size: 0.68rem;
    color: var(--color-text-subtle);
    flex-shrink: 0;
  }

  .line2 {
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
    padding: 2rem;
    color: var(--color-text-muted);
    text-align: center;
  }

  .empty :global(svg) {
    color: var(--color-text-subtle);
  }

  .muted {
    color: var(--color-text-subtle);
    font-size: 0.78rem;
  }

  .status {
    margin: 0.3rem 1rem 0;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md);
  }

  .status.error {
    color: var(--color-danger);
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
  }
</style>
