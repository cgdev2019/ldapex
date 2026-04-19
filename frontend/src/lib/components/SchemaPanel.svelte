<script lang="ts">
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    ldapFetchSchema,
    schemaResolveClasses,
    type AttributeTypeDef,
    type ObjectClassDef,
    type ResolvedClass,
    type SchemaInfo
  } from '$lib/bridge';
  import Icon from './Icon.svelte';

  type Tab = 'classes' | 'attributes';

  let schema = $state<SchemaInfo | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let tab = $state<Tab>('classes');
  let query = $state('');
  let selectedOcName = $state<string | null>(null);
  let selectedAtName = $state<string | null>(null);
  let resolved = $state<ResolvedClass | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    error = null;
    try {
      schema = await ldapFetchSchema();
      // Pre-select something useful by default.
      if (schema.object_classes.length > 0) {
        selectedOcName = schema.object_classes.find((c) => c.name === 'inetOrgPerson')?.name
          ?? schema.object_classes[0].name;
      }
      if (schema.attribute_types.length > 0) {
        selectedAtName = schema.attribute_types[0].name;
      }
    } catch (err) {
      error = formatError(err);
    } finally {
      loading = false;
    }
  }

  async function resolveSelected() {
    if (!schema || !selectedOcName) {
      resolved = null;
      return;
    }
    const list = await schemaResolveClasses(schema, [selectedOcName]);
    resolved = list[0] ?? null;
  }

  $effect(() => {
    if (selectedOcName) void resolveSelected();
  });

  const filteredClasses = $derived.by<ObjectClassDef[]>(() => {
    if (!schema) return [];
    const q = query.trim().toLowerCase();
    const all = schema.object_classes;
    return q ? all.filter((c) => c.name.toLowerCase().includes(q)) : all;
  });

  const filteredAttributes = $derived.by<AttributeTypeDef[]>(() => {
    if (!schema) return [];
    const q = query.trim().toLowerCase();
    const all = schema.attribute_types;
    return q
      ? all.filter(
          (a) =>
            a.name.toLowerCase().includes(q) ||
            a.aliases.some((al) => al.toLowerCase().includes(q)) ||
            a.oid.includes(q)
        )
      : all;
  });

  const selectedAt = $derived(
    selectedAtName && schema
      ? schema.attribute_types.find((a) => a.name === selectedAtName) ?? null
      : null
  );

  function kindColor(kind: string): string {
    if (kind === 'structural') return 'structural';
    if (kind === 'auxiliary') return 'auxiliary';
    return 'abstract';
  }
</script>

<div class="wrap">
  <header>
    <nav class="sub-tabs" role="tablist">
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'classes'}
        class:active={tab === 'classes'}
        onclick={() => (tab = 'classes')}
      >
        <Icon name="list" size={12} />
        <span>Classes</span>
        {#if schema}
          <span class="count">{schema.object_classes.length}</span>
        {/if}
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={tab === 'attributes'}
        class:active={tab === 'attributes'}
        onclick={() => (tab = 'attributes')}
      >
        <Icon name="circle-dot" size={12} />
        <span>Attributs</span>
        {#if schema}
          <span class="count">{schema.attribute_types.length}</span>
        {/if}
      </button>
    </nav>

    <label class="filter">
      <Icon name="search" size={12} />
      <input
        type="search"
        placeholder={tab === 'classes' ? 'inetOrgPerson…' : 'cn / 2.5.4.3…'}
        bind:value={query}
        spellcheck="false"
      />
    </label>
  </header>

  {#if loading}
    <p class="status"><Icon name="refresh" size={12} /> {$_('common.loading')}</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if schema}
    <div class="split">
      <ul class="list">
        {#if tab === 'classes'}
          {#each filteredClasses as oc (oc.name)}
            <li>
              <button
                type="button"
                class:active={selectedOcName === oc.name}
                onclick={() => (selectedOcName = oc.name)}
                title={oc.name}
              >
                <span class="oc-name">{oc.name}</span>
                <span class="kind-pill" data-kind={kindColor(oc.kind)}>{oc.kind.slice(0, 3)}</span>
              </button>
            </li>
          {/each}
        {:else}
          {#each filteredAttributes as at (at.name)}
            <li>
              <button
                type="button"
                class:active={selectedAtName === at.name}
                onclick={() => (selectedAtName = at.name)}
                title={at.name}
              >
                <span class="oc-name">{at.name}</span>
                {#if at.single_valued}
                  <span class="kind-pill" data-kind="single">1</span>
                {/if}
              </button>
            </li>
          {/each}
        {/if}
      </ul>

      <section class="detail">
        {#if tab === 'classes' && resolved}
          <h3 class="title">
            <code>{resolved.name}</code>
            <span class="kind-pill" data-kind={kindColor(resolved.kind)}>{resolved.kind}</span>
          </h3>

          {#if resolved.sup_chain.length > 0}
            <div class="block">
              <h4>Hérite de</h4>
              <div class="chain">
                {#each resolved.sup_chain as ancestor (ancestor)}
                  <button type="button" class="chip" onclick={() => (selectedOcName = ancestor)}>
                    {ancestor}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <div class="block">
            <h4>MUST <span class="muted">({resolved.must.length})</span></h4>
            <div class="attrs">
              {#each resolved.must as a (a)}
                <button
                  type="button"
                  class="attr-chip must"
                  onclick={() => {
                    tab = 'attributes';
                    selectedAtName = a;
                  }}
                >
                  <code>{a}</code>
                </button>
              {/each}
            </div>
          </div>

          <div class="block">
            <h4>MAY <span class="muted">({resolved.may.length})</span></h4>
            <div class="attrs">
              {#each resolved.may as a (a)}
                <button
                  type="button"
                  class="attr-chip may"
                  onclick={() => {
                    tab = 'attributes';
                    selectedAtName = a;
                  }}
                >
                  <code>{a}</code>
                </button>
              {/each}
            </div>
          </div>
        {:else if tab === 'attributes' && selectedAt}
          <h3 class="title">
            <code>{selectedAt.name}</code>
            {#if selectedAt.single_valued}
              <span class="kind-pill" data-kind="single">single</span>
            {/if}
          </h3>

          <dl class="kv">
            {#if selectedAt.aliases.length > 0}
              <dt>Aliases</dt>
              <dd>
                {#each selectedAt.aliases as a (a)}
                  <code class="inline">{a}</code>
                {/each}
              </dd>
            {/if}
            <dt>OID</dt>
            <dd><code>{selectedAt.oid || '—'}</code></dd>
            {#if selectedAt.sup}
              <dt>SUP</dt>
              <dd>
                <button
                  type="button"
                  class="chip"
                  onclick={() => (selectedAtName = selectedAt.sup ?? null)}
                >
                  {selectedAt.sup}
                </button>
              </dd>
            {/if}
            {#if selectedAt.syntax}
              <dt>Syntax</dt>
              <dd><code>{selectedAt.syntax}</code></dd>
            {/if}
            {#if selectedAt.equality}
              <dt>Equality</dt>
              <dd><code>{selectedAt.equality}</code></dd>
            {/if}
            {#if selectedAt.substring}
              <dt>Substring</dt>
              <dd><code>{selectedAt.substring}</code></dd>
            {/if}
            {#if selectedAt.ordering}
              <dt>Ordering</dt>
              <dd><code>{selectedAt.ordering}</code></dd>
            {/if}
            {#if selectedAt.usage}
              <dt>Usage</dt>
              <dd><code>{selectedAt.usage}</code></dd>
            {/if}
            {#if selectedAt.no_user_modification}
              <dt>Flags</dt>
              <dd><span class="kind-pill" data-kind="locked">no-user-mod</span></dd>
            {/if}
          </dl>
        {:else}
          <p class="empty">Sélectionne un élément à gauche.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.5rem 0.5rem 0.6rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .sub-tabs {
    display: flex;
    gap: 0.25rem;
  }

  .sub-tabs button {
    flex: 1;
    background: transparent;
    border: 1px solid transparent;
    padding: 0.25rem 0.5rem;
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    border-radius: var(--radius-sm);
    gap: 0.3rem;
    justify-content: flex-start;
  }

  .sub-tabs button:hover:not(.active) {
    background: var(--color-surface-hover);
  }

  .sub-tabs button.active {
    background: var(--color-primary-soft);
    color: var(--color-primary);
    border-color: color-mix(in oklab, var(--color-primary) 25%, transparent);
  }

  .count {
    font-size: 0.62rem;
    padding: 0.02rem 0.35rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    margin-left: auto;
  }

  .filter {
    position: relative;
    display: flex;
    align-items: center;
  }

  .filter :global(svg) {
    position: absolute;
    left: 0.55rem;
    color: var(--color-text-subtle);
    pointer-events: none;
  }

  .filter input {
    padding-left: 1.85rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }

  .split {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(11rem, 14rem) 1fr;
    min-height: 0;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0.25rem;
    overflow: auto;
    border-right: 1px solid var(--color-border);
  }

  .list li {
    margin: 0;
  }

  .list button {
    width: 100%;
    background: transparent;
    border: none;
    padding: 0.25rem 0.45rem;
    color: var(--color-text);
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    justify-content: flex-start;
    text-align: left;
  }

  .list button:hover {
    background: var(--color-surface-hover);
  }

  .list button.active {
    background: var(--color-primary-soft);
    color: var(--color-primary);
    font-weight: 500;
  }

  .oc-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
  }

  .kind-pill {
    font-size: 0.6rem;
    padding: 0.02rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
    flex-shrink: 0;
  }

  .kind-pill[data-kind='structural'] {
    background: color-mix(in oklab, var(--color-primary) 18%, transparent);
    color: var(--color-primary);
    border-color: color-mix(in oklab, var(--color-primary) 30%, transparent);
  }

  .kind-pill[data-kind='auxiliary'] {
    background: color-mix(in oklab, var(--color-warning) 18%, transparent);
    color: var(--color-warning);
    border-color: color-mix(in oklab, var(--color-warning) 30%, transparent);
  }

  .kind-pill[data-kind='abstract'] {
    background: color-mix(in oklab, var(--color-text-subtle) 18%, transparent);
  }

  .kind-pill[data-kind='single'],
  .kind-pill[data-kind='locked'] {
    background: color-mix(in oklab, var(--color-success) 18%, transparent);
    color: var(--color-success);
    border-color: color-mix(in oklab, var(--color-success) 30%, transparent);
  }

  .detail {
    overflow: auto;
    padding: 0.7rem 0.85rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .detail .title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1rem;
  }

  .detail .title code {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 1.05em;
    color: var(--color-text);
  }

  .block h4 {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-text-subtle);
    font-weight: 600;
    margin-bottom: 0.35rem;
  }

  .muted {
    color: var(--color-text-subtle);
    font-weight: 400;
  }

  .chain,
  .attrs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .chip {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text);
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.72rem;
    font-family: var(--font-mono);
  }

  .chip:hover {
    background: var(--color-surface-hover);
  }

  .attr-chip {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text);
    padding: 0.15rem 0.4rem;
    border-radius: var(--radius-sm);
    font-size: 0.72rem;
  }

  .attr-chip code {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 1em;
  }

  .attr-chip.must {
    background: color-mix(in oklab, var(--color-danger) 8%, transparent);
    border-color: color-mix(in oklab, var(--color-danger) 25%, transparent);
    color: var(--color-danger);
  }

  .attr-chip.may {
    background: var(--color-surface-2);
  }

  .attr-chip:hover {
    background: var(--color-surface-hover);
  }

  .kv {
    display: grid;
    grid-template-columns: 6rem 1fr;
    gap: 0.35rem 0.7rem;
    font-size: 0.78rem;
    margin: 0;
  }

  .kv dt {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-subtle);
    align-self: center;
  }

  .kv dd {
    margin: 0;
    color: var(--color-text);
    word-break: break-all;
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .inline {
    font-size: 0.72rem;
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.6rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .status.error {
    color: var(--color-danger);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .empty {
    color: var(--color-text-subtle);
    padding: 1rem;
    font-style: italic;
  }
</style>
