<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    ldapDelete,
    ldapModify,
    type Modification
  } from '$lib/bridge';
  import { selection } from '$lib/selection.svelte';
  import Icon from './Icon.svelte';

  type Op = 'add' | 'replace' | 'delete';

  interface Props {
    onchanged?: () => void;
  }

  let { onchanged }: Props = $props();

  let dialog = $state<'modify' | 'delete' | null>(null);
  let attribute = $state('description');
  let op = $state<Op>('replace');
  let value = $state('');
  let progress = $state(0);
  let total = $state(0);
  let log = $state<string[]>([]);
  let working = $state(false);

  function openModify() {
    dialog = 'modify';
    log = [];
    progress = 0;
  }

  function openDelete() {
    dialog = 'delete';
    log = [];
    progress = 0;
  }

  function close() {
    if (working) return;
    dialog = null;
  }

  async function runModify(event: SubmitEvent) {
    event.preventDefault();
    const dns = selection.list();
    if (dns.length === 0) return;
    working = true;
    progress = 0;
    total = dns.length;
    log = [];
    const values = value
      .split('\n')
      .map((v) => v.trim())
      .filter((v) => v.length > 0);
    const mod: Modification =
      op === 'delete'
        ? { op: 'delete', attribute, values: values.length > 0 ? values : null }
        : { op, attribute, values };
    for (const dn of dns) {
      try {
        await ldapModify(dn, [mod]);
        log = [...log, `✓ ${dn}`];
      } catch (err) {
        log = [...log, `✗ ${dn} — ${formatError(err)}`];
      } finally {
        progress += 1;
      }
    }
    working = false;
    onchanged?.();
  }

  async function runDelete() {
    const dns = selection.list();
    if (dns.length === 0) return;
    const yes = window.confirm(
      get(_)('entry.confirm_delete', {
        values: { dn: `${dns.length} entries:\n${dns.slice(0, 10).join('\n')}${dns.length > 10 ? '\n…' : ''}` }
      })
    );
    if (!yes) return;
    working = true;
    progress = 0;
    total = dns.length;
    log = [];
    for (const dn of dns) {
      try {
        await ldapDelete(dn);
        log = [...log, `✓ ${dn}`];
        selection.remove(dn);
      } catch (err) {
        log = [...log, `✗ ${dn} — ${formatError(err)}`];
      } finally {
        progress += 1;
      }
    }
    working = false;
    onchanged?.();
  }
</script>

{#if selection.count > 0}
  <div class="bar">
    <span class="count">
      <Icon name="check" size={11} />
      <strong>{selection.count}</strong>
      <span>sélectionnée{selection.count > 1 ? 's' : ''}</span>
    </span>
    <div class="actions">
      <button type="button" class="ghost sm" onclick={() => selection.clear()} title="Tout désélectionner">
        <Icon name="x" size={11} />
      </button>
      <button type="button" class="ghost sm" onclick={openModify}>
        <Icon name="pencil" size={11} />
        <span>Modifier</span>
      </button>
      <button type="button" class="ghost sm danger" onclick={openDelete}>
        <Icon name="trash" size={11} />
        <span>Supprimer</span>
      </button>
    </div>
  </div>
{/if}

{#if dialog === 'modify'}
  <div class="backdrop" role="dialog" aria-modal="true" aria-label="Modification en masse" onclick={close} tabindex="-1">
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <header>
        <h3>Modifier {selection.count} entrée{selection.count > 1 ? 's' : ''}</h3>
        <button type="button" class="ghost icon-only" onclick={close} disabled={working}>
          <Icon name="x" size={14} />
        </button>
      </header>
      <form onsubmit={runModify}>
        <div class="row">
          <label class="flex">
            <span>Attribut</span>
            <input type="text" bind:value={attribute} required spellcheck="false" />
          </label>
          <label class="w-op">
            <span>Opération</span>
            <select bind:value={op}>
              <option value="add">Ajouter</option>
              <option value="replace">Remplacer</option>
              <option value="delete">Supprimer</option>
            </select>
          </label>
        </div>
        <label>
          <span>{op === 'delete' ? 'Valeurs (vide = supprimer toutes)' : 'Valeur(s) — une par ligne'}</span>
          <textarea rows="3" bind:value={value} spellcheck="false"></textarea>
        </label>

        {#if working || progress > 0}
          <div class="progress">
            <div class="bar-bg">
              <div class="bar-fg" style:width={`${total ? (progress * 100) / total : 0}%`}></div>
            </div>
            <span>{progress}/{total}</span>
          </div>
          <pre class="log">{log.join('\n')}</pre>
        {/if}

        <footer>
          <button type="button" class="ghost" onclick={close} disabled={working}>Fermer</button>
          <button type="submit" class="primary" disabled={working || total === progress + 1 || (working && progress < total)}>
            {working ? 'En cours…' : 'Lancer'}
          </button>
        </footer>
      </form>
    </div>
  </div>
{/if}

{#if dialog === 'delete'}
  <div class="backdrop" role="dialog" aria-modal="true" aria-label="Suppression en masse" onclick={close} tabindex="-1">
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <header>
        <h3>Supprimer {selection.count} entrée{selection.count > 1 ? 's' : ''}</h3>
        <button type="button" class="ghost icon-only" onclick={close} disabled={working}>
          <Icon name="x" size={14} />
        </button>
      </header>
      <p class="warn">
        <Icon name="alert-triangle" size={14} />
        <span>Action irréversible. Les entrées avec des enfants échoueront.</span>
      </p>
      {#if working || progress > 0}
        <div class="progress">
          <div class="bar-bg">
            <div class="bar-fg" style:width={`${total ? (progress * 100) / total : 0}%`}></div>
          </div>
          <span>{progress}/{total}</span>
        </div>
        <pre class="log">{log.join('\n')}</pre>
      {/if}
      <footer>
        <button type="button" class="ghost" onclick={close} disabled={working}>Annuler</button>
        <button type="button" class="primary danger" onclick={runDelete} disabled={working}>
          {working ? 'Suppression…' : 'Confirmer'}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-primary-soft);
    font-size: var(--text-xs);
  }

  .count {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    color: var(--color-primary);
    flex: 1;
  }

  .actions {
    display: flex;
    gap: 0.3rem;
  }

  :global(button.ghost.sm.danger) {
    color: var(--color-danger);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 55%, transparent);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    z-index: 220;
    padding: 1rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    width: min(92vw, 32rem);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }

  .dialog header {
    display: flex;
    align-items: center;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .dialog header h3 {
    flex: 1;
    font-size: 1rem;
  }

  form {
    padding: 0.85rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .row {
    display: flex;
    gap: 0.5rem;
  }

  .flex {
    flex: 1;
  }

  .w-op {
    width: 9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: var(--text-sm);
  }

  label > span {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .bar-bg {
    flex: 1;
    height: 0.4rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-pill);
    overflow: hidden;
  }

  .bar-fg {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.2s ease;
  }

  .log {
    max-height: 8rem;
    overflow: auto;
    margin: 0;
    padding: 0.45rem 0.55rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-md);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .warn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0 1rem 0.7rem;
    padding: 0.5rem 0.75rem;
    background: color-mix(in oklab, var(--color-warning) 15%, transparent);
    color: var(--color-warning);
    border: 1px solid color-mix(in oklab, var(--color-warning) 35%, transparent);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0.7rem 1rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-2);
  }

  .danger {
    background: var(--color-danger);
    border-color: transparent;
    color: white;
  }

  .danger:hover:not(:disabled) {
    background: color-mix(in oklab, var(--color-danger) 80%, black);
  }
</style>
