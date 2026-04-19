<script lang="ts">
  import {
    formatError,
    ldapAdd,
    ldapFetchSchema,
    type Attribute,
    type ObjectClassDef,
    type SchemaInfo
  } from '$lib/bridge';

  interface Props {
    parentDn: string;
    onclose: () => void;
    oncreated?: (dn: string) => void;
  }

  let { parentDn, onclose, oncreated }: Props = $props();

  let schema = $state<SchemaInfo | null>(null);
  let loadingSchema = $state(true);
  let schemaError = $state<string | null>(null);

  let selectedClassNames = $state<string[]>(['inetOrgPerson']);
  let rdn = $state('cn=Nouvel utilisateur');
  let attrValues = $state<Record<string, string>>({});
  let saving = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    loadSchema();
  });

  async function loadSchema() {
    loadingSchema = true;
    schemaError = null;
    try {
      schema = await ldapFetchSchema();
    } catch (err) {
      schemaError = formatError(err);
    } finally {
      loadingSchema = false;
    }
  }

  const selectedClasses = $derived<ObjectClassDef[]>(
    schema
      ? selectedClassNames
          .map((n) => schema!.object_classes.find((c) => c.name.toLowerCase() === n.toLowerCase()))
          .filter((c): c is ObjectClassDef => c !== undefined)
      : []
  );

  const must = $derived(uniqueAttrs(selectedClasses.flatMap((c) => c.must)));
  const may = $derived(uniqueAttrs(selectedClasses.flatMap((c) => c.may)));

  function uniqueAttrs(arr: string[]): string[] {
    const out: string[] = [];
    const seen = new Set<string>();
    for (const a of arr) {
      const lower = a.toLowerCase();
      if (!seen.has(lower)) {
        seen.add(lower);
        out.push(a);
      }
    }
    return out;
  }

  function toggleClass(name: string) {
    if (selectedClassNames.includes(name)) {
      selectedClassNames = selectedClassNames.filter((n) => n !== name);
    } else {
      selectedClassNames = [...selectedClassNames, name];
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    const dn = `${rdn},${parentDn}`;

    const attributes: Attribute[] = [];
    attributes.push({
      name: 'objectClass',
      values: selectedClassNames.map((n) => ({ kind: 'text', data: n }))
    });

    // Extract the RDN's attr=value so we also set it as an attribute
    // (LDAP requires the RDN attribute to be present in the entry).
    const rdnMatch = /^([^=]+)=(.+)$/.exec(rdn.trim());
    if (rdnMatch) {
      const [, rdnAttr, rdnValue] = rdnMatch;
      if (!attrValues[rdnAttr] || attrValues[rdnAttr].trim() === '') {
        attrValues[rdnAttr] = rdnValue;
      }
    }

    for (const [name, raw] of Object.entries(attrValues)) {
      if (raw.trim() === '') continue;
      attributes.push({
        name,
        values: raw
          .split('\n')
          .map((s) => s.trim())
          .filter((s) => s.length > 0)
          .map((s) => ({ kind: 'text', data: s }))
      });
    }

    saving = true;
    try {
      await ldapAdd(dn, attributes);
      oncreated?.(dn);
      onclose();
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label="Créer une entrée">
  <div class="dialog">
    <h2>Nouvelle entrée sous <code>{parentDn}</code></h2>

    {#if loadingSchema}
      <p class="status">Récupération du schéma…</p>
    {:else if schemaError}
      <p class="status error">Schéma : {schemaError}</p>
    {/if}

    <form onsubmit={submit}>
      <label>
        <span>RDN</span>
        <input type="text" bind:value={rdn} required spellcheck="false" />
      </label>

      {#if schema}
        <label>
          <span>Object classes (MUST/MAY ci-dessous)</span>
          <div class="class-picker">
            {#each schema.object_classes.filter((c) => c.kind !== 'abstract') as oc (oc.name)}
              <label class="chip">
                <input
                  type="checkbox"
                  checked={selectedClassNames.includes(oc.name)}
                  onchange={() => toggleClass(oc.name)}
                />
                <span>{oc.name}</span>
                <small>({oc.kind})</small>
              </label>
            {/each}
          </div>
        </label>
      {/if}

      {#if must.length > 0}
        <fieldset>
          <legend>MUST</legend>
          {#each must as name (name)}
            {@const current = attrValues[name] ?? ''}
            <label class="kv">
              <span>{name}*</span>
              <textarea
                rows="1"
                oninput={(e) =>
                  (attrValues = {
                    ...attrValues,
                    [name]: (e.currentTarget as HTMLTextAreaElement).value
                  })}
                value={current}
              ></textarea>
            </label>
          {/each}
        </fieldset>
      {/if}

      {#if may.length > 0}
        <fieldset>
          <legend>MAY</legend>
          {#each may.slice(0, 15) as name (name)}
            {@const current = attrValues[name] ?? ''}
            <label class="kv">
              <span>{name}</span>
              <textarea
                rows="1"
                oninput={(e) =>
                  (attrValues = {
                    ...attrValues,
                    [name]: (e.currentTarget as HTMLTextAreaElement).value
                  })}
                value={current}
              ></textarea>
            </label>
          {/each}
          {#if may.length > 15}
            <p class="muted">… ({may.length - 15} attributs MAY masqués, à ajouter ensuite en édition)</p>
          {/if}
        </fieldset>
      {/if}

      {#if error}
        <p class="status error">{error}</p>
      {/if}

      <div class="actions">
        <button type="submit" disabled={saving}>{saving ? 'Création…' : 'Créer'}</button>
        <button type="button" class="secondary" onclick={onclose} disabled={saving}>
          Annuler
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 100;
  }

  .dialog {
    background: light-dark(#fff, #161616);
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    width: min(90vw, 40rem);
    max-height: 90vh;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.9rem;
  }

  label > span {
    color: light-dark(#555, #aaa);
  }

  input,
  select,
  textarea {
    font: inherit;
    padding: 0.35rem 0.55rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
    resize: vertical;
  }

  .class-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    max-height: 10rem;
    overflow: auto;
    padding: 0.4rem;
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 4px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.1rem 0.45rem;
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 12px;
    font-size: 0.8rem;
  }

  .chip input {
    margin: 0;
  }

  .chip small {
    color: light-dark(#888, #777);
  }

  fieldset {
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 5px;
    padding: 0.5rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  legend {
    font-weight: 600;
    padding: 0 0.25rem;
    font-size: 0.85rem;
  }

  .kv {
    display: grid;
    grid-template-columns: 10rem 1fr;
    gap: 0.5rem;
    align-items: center;
  }

  .kv > span {
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .secondary {
    background: transparent;
  }

  .status {
    margin: 0;
    padding: 0.25rem 0;
    color: light-dark(#666, #888);
  }

  .status.error {
    color: #c0392b;
  }

  .muted {
    color: light-dark(#777, #888);
    font-style: italic;
    margin: 0;
    font-size: 0.8rem;
  }
</style>
