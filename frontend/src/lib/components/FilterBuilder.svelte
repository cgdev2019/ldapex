<script lang="ts">
  import {
    EMPTY_GROUP,
    EMPTY_LEAF,
    compile,
    type Group,
    type LeafOp,
    type Node,
    type Op
  } from '$lib/filter';
  import type { AttributeTypeDef } from '$lib/bridge';
  import Icon from './Icon.svelte';

  interface Props {
    root: Group;
    attributes?: AttributeTypeDef[];
    onchange?: (next: Group) => void;
    nested?: boolean;
  }

  let { root = $bindable(), attributes = [], onchange, nested = false }: Props = $props();

  function updated() {
    onchange?.(root);
    root = { ...root };
  }

  function setOp(op: Op) {
    root.op = op;
    if (op === 'not' && root.children.length > 1) {
      root.children = root.children.slice(0, 1);
    }
    updated();
  }

  function addLeaf() {
    if (root.op === 'not' && root.children.length > 0) return;
    root.children = [...root.children, EMPTY_LEAF()];
    updated();
  }

  function addGroup() {
    if (root.op === 'not' && root.children.length > 0) return;
    root.children = [...root.children, EMPTY_GROUP()];
    updated();
  }

  function remove(i: number) {
    root.children = root.children.filter((_, idx) => idx !== i);
    updated();
  }

  function replaceChild(i: number, next: Node) {
    root.children = root.children.map((c, idx) => (idx === i ? next : c));
    updated();
  }

  const attributeNames = $derived(attributes.map((a) => a.name));
</script>

<div class="group" class:nested>
  <header>
    <div class="ops" role="radiogroup" aria-label="Boolean operator">
      {#each ['and', 'or', 'not'] as op (op)}
        <button
          type="button"
          class="op"
          class:active={root.op === op}
          onclick={() => setOp(op as Op)}
          role="radio"
          aria-checked={root.op === op}
        >
          {op.toUpperCase()}
        </button>
      {/each}
    </div>
    {#if root.op !== 'not' || root.children.length === 0}
      <div class="actions">
        <button type="button" class="ghost sm" onclick={addLeaf}>
          <Icon name="plus" size={11} />
          <span>Condition</span>
        </button>
        <button type="button" class="ghost sm" onclick={addGroup}>
          <Icon name="plus" size={11} />
          <span>Groupe</span>
        </button>
      </div>
    {/if}
  </header>

  <ul class="children">
    {#each root.children as child, i (i)}
      <li>
        {#if child.kind === 'group'}
          <svelte:self
            root={child}
            {attributes}
            nested={true}
            onchange={(g: Group) => replaceChild(i, g)}
          />
        {:else}
          <div class="leaf">
            <input
              class="attr"
              type="text"
              list="ldapex-attr-names"
              bind:value={child.attribute}
              oninput={updated}
              spellcheck="false"
              placeholder="objectClass"
            />
            <select
              class="leaf-op"
              bind:value={child.op}
              onchange={updated}
            >
              <option value="present">est défini</option>
              <option value="equal">=</option>
              <option value="substring">contient (*)</option>
              <option value="ge">≥</option>
              <option value="le">≤</option>
              <option value="approx">~=</option>
            </select>
            {#if child.op !== 'present'}
              <input
                class="value"
                type="text"
                bind:value={child.value}
                oninput={updated}
                placeholder={child.op === 'substring' ? 'a*' : 'valeur'}
                spellcheck="false"
              />
            {/if}
            <button
              type="button"
              class="ghost icon-only tiny"
              onclick={() => remove(i)}
              aria-label="Retirer"
              title="Retirer"
            >
              <Icon name="x" size={11} />
            </button>
          </div>
        {/if}
      </li>
    {/each}
    {#if root.children.length === 0}
      <li class="empty">Vide — ajoute une condition.</li>
    {/if}
  </ul>

  {#if !nested}
    <datalist id="ldapex-attr-names">
      {#each attributeNames as n (n)}
        <option value={n}></option>
      {/each}
    </datalist>
    <footer>
      <span class="lbl">Filtre RFC 4515</span>
      <code class="preview">{compile(root)}</code>
    </footer>
  {/if}
</div>

<style>
  .group {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.4rem 0.5rem 0.5rem;
    background: var(--color-surface);
  }

  .group.nested {
    background: var(--color-surface-2);
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.4rem;
  }

  .ops {
    display: flex;
    background: var(--color-surface-2);
    border-radius: var(--radius-pill);
    padding: 0.1rem;
  }

  .op {
    background: transparent;
    border: none;
    padding: 0.1rem 0.55rem;
    border-radius: var(--radius-pill);
    font-size: 0.65rem;
    color: var(--color-text-muted);
    font-weight: 600;
    letter-spacing: 0.06em;
    cursor: pointer;
  }

  .op.active {
    background: var(--color-primary);
    color: white;
  }

  .actions {
    display: flex;
    gap: 0.25rem;
    margin-left: auto;
  }

  .children {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .leaf {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.15rem 0;
  }

  .leaf .attr {
    flex: 0 0 9rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }

  .leaf .leaf-op {
    flex: 0 0 9rem;
    font-size: 0.78rem;
  }

  .leaf .value {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 0.78rem;
  }

  .tiny {
    width: 1.5rem;
    height: 1.5rem;
    padding: 0.2rem;
    color: var(--color-text-subtle);
  }

  .tiny:hover:not(:disabled) {
    color: var(--color-danger);
  }

  .empty {
    color: var(--color-text-subtle);
    font-style: italic;
    font-size: var(--text-xs);
    padding: 0.25rem 0.4rem;
  }

  footer {
    margin-top: 0.5rem;
    padding-top: 0.45rem;
    border-top: 1px dashed var(--color-border);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .lbl {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-text-subtle);
    font-weight: 600;
    flex-shrink: 0;
  }

  .preview {
    flex: 1;
    overflow: auto;
    white-space: nowrap;
    font-size: 0.75rem;
    background: var(--color-surface-2);
  }
</style>
