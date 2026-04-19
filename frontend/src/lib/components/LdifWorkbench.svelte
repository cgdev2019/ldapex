<script lang="ts">
  import {
    formatError,
    ldapAdd,
    ldapDelete,
    ldapModify,
    ldapRename
  } from '$lib/bridge';
  import { parseLdif, recordLabel, type ChangeRecord } from '$lib/ldif-parser';
  import Icon from './Icon.svelte';

  interface Props {
    onclose: () => void;
    onchanged?: () => void;
  }

  let { onclose, onchanged }: Props = $props();

  const EXAMPLE = `# Example workbench input
dn: uid=claire,ou=People,dc=ldapex,dc=test
objectClass: inetOrgPerson
cn: Claire Example
sn: Example
mail: claire@ldapex.test

dn: uid=alice,ou=People,dc=ldapex,dc=test
changetype: modify
replace: mail
mail: alice.new@ldapex.test
-

dn: uid=bob,ou=People,dc=ldapex,dc=test
changetype: delete
`;

  let text = $state(EXAMPLE);
  let working = $state(false);
  let progress = $state(0);
  let total = $state(0);
  let log = $state<Array<{ ok: boolean; dn: string; message: string }>>([]);

  const parsed = $derived(parseLdif(text));

  async function apply() {
    working = true;
    progress = 0;
    total = parsed.records.length;
    log = [];
    for (const rec of parsed.records) {
      try {
        await applyOne(rec);
        log = [...log, { ok: true, dn: rec.dn, message: '' }];
      } catch (err) {
        log = [...log, { ok: false, dn: rec.dn, message: formatError(err) }];
      } finally {
        progress += 1;
      }
    }
    working = false;
    onchanged?.();
  }

  async function applyOne(rec: ChangeRecord): Promise<void> {
    switch (rec.kind) {
      case 'add':
        await ldapAdd(rec.dn, rec.attributes);
        return;
      case 'modify':
        await ldapModify(rec.dn, rec.mods);
        return;
      case 'delete':
        await ldapDelete(rec.dn);
        return;
      case 'modrdn':
        await ldapRename({
          dn: rec.dn,
          new_rdn: rec.newRdn,
          new_parent: rec.newSuperior ?? null,
          delete_old_rdn: rec.deleteOldRdn
        });
        return;
    }
  }

  function close() {
    if (!working) onclose();
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label="LDIF workbench" onclick={close} tabindex="-1">
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header>
      <h2>
        <Icon name="file-lock" size={16} />
        <span>LDIF workbench</span>
      </h2>
      <div class="spacer"></div>
      <button type="button" class="ghost icon-only" onclick={close} disabled={working}>
        <Icon name="x" size={14} />
      </button>
    </header>

    <div class="split">
      <section class="editor">
        <h4>1. Colle un LDIF (RFC 2849)</h4>
        <textarea
          bind:value={text}
          spellcheck="false"
          disabled={working}
          placeholder="dn: uid=alice,…"
        ></textarea>
        {#if parsed.errors.length > 0}
          <ul class="errors">
            {#each parsed.errors as e (e)}
              <li><Icon name="alert-triangle" size={11} /> {e}</li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="preview">
        <h4>2. Aperçu du plan</h4>
        {#if parsed.records.length === 0}
          <p class="empty">
            {parsed.errors.length > 0
              ? 'Aucun record valide.'
              : 'Le plan s\u2019affichera ici au fil de la saisie.'}
          </p>
        {:else}
          <ol class="records">
            {#each parsed.records as rec, i (i)}
              {@const meta = recordLabel(rec)}
              <li class="record" data-op={meta.color}>
                <header>
                  <span class="badge" data-op={meta.color}>{rec.kind}</span>
                  <code class="dn">{rec.dn}</code>
                </header>
                <p class="sub">{meta.op}</p>
                {#if rec.kind === 'add'}
                  <ul class="attrs">
                    {#each rec.attributes.slice(0, 6) as a (a.name)}
                      <li>
                        <code>{a.name}</code>:
                        <span class="val">
                          {a.values
                            .map((v) => (v.kind === 'text' ? v.data : '<binary>'))
                            .join(', ')}
                        </span>
                      </li>
                    {/each}
                    {#if rec.attributes.length > 6}
                      <li class="more">… ({rec.attributes.length - 6} de plus)</li>
                    {/if}
                  </ul>
                {:else if rec.kind === 'modify'}
                  <ul class="attrs">
                    {#each rec.mods as m (`${m.op}-${m.attribute}`)}
                      <li>
                        <span class="op-tag" data-op={m.op}>{m.op}</span>
                        <code>{m.attribute}</code>
                        {#if m.op !== 'delete' || (m.values && m.values.length > 0)}
                          <span class="val">
                            {#if m.op === 'delete' && !m.values}
                              (all)
                            {:else}
                              {(m.values ?? []).join(', ')}
                            {/if}
                          </span>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                {:else if rec.kind === 'modrdn'}
                  <p class="sub">
                    <code>newRdn={rec.newRdn}</code>
                    {#if rec.newSuperior}
                      <code>newSuperior={rec.newSuperior}</code>
                    {/if}
                  </p>
                {/if}
              </li>
            {/each}
          </ol>
        {/if}

        {#if working || progress > 0}
          <div class="progress">
            <div class="bar-bg">
              <div class="bar-fg" style:width={`${total ? (progress * 100) / total : 0}%`}></div>
            </div>
            <span>{progress}/{total}</span>
          </div>
          <ul class="log">
            {#each log as line, i (i)}
              <li class:ok={line.ok} class:ko={!line.ok}>
                <Icon name={line.ok ? 'check' : 'x'} size={10} />
                <code>{line.dn}</code>
                {#if line.message}
                  <span class="msg">— {line.message}</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>

    <footer>
      <span class="stat">
        <strong>{parsed.records.length}</strong> record{parsed.records.length > 1 ? 's' : ''}
        {#if parsed.errors.length > 0}
          · <span class="err">{parsed.errors.length} erreur{parsed.errors.length > 1 ? 's' : ''}</span>
        {/if}
      </span>
      <div class="actions">
        <button type="button" class="ghost" onclick={close} disabled={working}>Fermer</button>
        <button
          type="button"
          class="primary"
          disabled={working || parsed.records.length === 0}
          onclick={apply}
        >
          <Icon name="save" size={14} />
          <span>{working ? 'Application…' : 'Appliquer'}</span>
        </button>
      </div>
    </footer>
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
    width: min(94vw, 64rem);
    max-height: 92vh;
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
  }

  header h2 {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
  }

  .spacer {
    flex: 1;
  }

  .split {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(20rem, 30rem) 1fr;
    overflow: hidden;
  }

  .editor {
    border-right: 1px solid var(--color-border);
    background: var(--color-surface-2);
    padding: 0.75rem 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    overflow: hidden;
  }

  .preview {
    padding: 0.75rem 0.9rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  h4 {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-text-subtle);
    font-weight: 600;
  }

  textarea {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    resize: none;
    min-height: 12rem;
  }

  .errors {
    list-style: none;
    margin: 0;
    padding: 0;
    color: var(--color-danger);
    font-size: 0.72rem;
  }

  .errors li {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.15rem 0;
  }

  .records {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .record {
    border: 1px solid var(--color-border);
    border-left-width: 3px;
    border-radius: var(--radius-md);
    padding: 0.4rem 0.55rem;
  }

  .record[data-op='add'] {
    border-left-color: var(--color-success);
  }
  .record[data-op='modify'] {
    border-left-color: var(--color-primary);
  }
  .record[data-op='delete'] {
    border-left-color: var(--color-danger);
  }
  .record[data-op='rename'] {
    border-left-color: var(--color-warning);
  }

  .record > header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    border-bottom: none;
    padding: 0;
  }

  .badge {
    font-size: 0.6rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-weight: 600;
    padding: 0.05rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
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

  .dn {
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .sub {
    margin: 0.2rem 0 0.1rem 0;
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  .attrs {
    list-style: none;
    margin: 0.25rem 0 0;
    padding-left: 0.5rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .attrs .val {
    color: var(--color-text);
    word-break: break-all;
  }

  .op-tag {
    display: inline-block;
    font-size: 0.56rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 0 0.35rem;
    border-radius: var(--radius-pill);
    margin-right: 0.15rem;
    font-weight: 600;
    background: var(--color-surface-2);
    color: var(--color-text-muted);
  }

  .op-tag[data-op='add'] {
    color: var(--color-success);
  }
  .op-tag[data-op='replace'] {
    color: var(--color-primary);
  }
  .op-tag[data-op='delete'] {
    color: var(--color-danger);
  }

  .more {
    color: var(--color-text-subtle);
    font-style: italic;
  }

  .empty {
    color: var(--color-text-subtle);
    padding: 0.8rem;
    font-style: italic;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
    margin-top: 0.5rem;
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
    list-style: none;
    margin: 0.25rem 0 0;
    padding: 0;
    font-size: 0.7rem;
    font-family: var(--font-mono);
    max-height: 9rem;
    overflow: auto;
  }

  .log li {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.1rem 0;
    width: 100%;
  }

  .log li.ok {
    color: var(--color-success);
  }

  .log li.ko {
    color: var(--color-danger);
  }

  .log .msg {
    color: var(--color-text-muted);
  }

  footer {
    display: flex;
    align-items: center;
    padding: 0.7rem 1.1rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-2);
    gap: 0.75rem;
  }

  .stat {
    flex: 1;
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .stat .err {
    color: var(--color-danger);
    font-weight: 500;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  @media (max-width: 820px) {
    .split {
      grid-template-columns: 1fr;
    }

    .editor {
      border-right: none;
      border-bottom: 1px solid var(--color-border);
    }
  }
</style>
