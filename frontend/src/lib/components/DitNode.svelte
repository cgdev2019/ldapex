<script lang="ts">
  import { formatError, ldapListChildren, type DnLabel } from '$lib/bridge';
  import DitNode from './DitNode.svelte';

  interface Props {
    node: DnLabel;
    selectedDn: string | null;
    onselect: (dn: string) => void;
  }

  let { node, selectedDn, onselect }: Props = $props();

  let expanded = $state(false);
  let loading = $state(false);
  let children = $state<DnLabel[] | null>(null);
  let error = $state<string | null>(null);

  async function toggle() {
    if (!expanded && children === null) {
      loading = true;
      error = null;
      try {
        children = await ldapListChildren(node.dn);
      } catch (err) {
        error = formatError(err);
      } finally {
        loading = false;
      }
    }
    expanded = !expanded;
  }

  function icon(): string {
    const oc = node.object_classes.map((c) => c.toLowerCase());
    if (oc.includes('organizationalunit')) return '📁';
    if (oc.includes('inetorgperson') || oc.includes('person')) return '👤';
    if (oc.includes('groupofnames') || oc.includes('groupofuniquenames')) return '👥';
    if (oc.includes('dcobject') || oc.includes('organization')) return '🏢';
    return '·';
  }

  const isSelected = $derived(selectedDn === node.dn);
</script>

<li>
  <div class="row" class:selected={isSelected}>
    <button
      type="button"
      class="chevron"
      onclick={toggle}
      aria-label={expanded ? 'Replier' : 'Déplier'}
    >
      {expanded ? '▾' : '▸'}
    </button>
    <button type="button" class="label" onclick={() => onselect(node.dn)} title={node.dn}>
      <span class="icon" aria-hidden="true">{icon()}</span>
      <span class="text">{node.label}</span>
    </button>
  </div>

  {#if expanded}
    <ul class="children">
      {#if loading}
        <li class="status">Chargement…</li>
      {:else if error}
        <li class="status error">{error}</li>
      {:else if children && children.length === 0}
        <li class="status muted">(aucun enfant)</li>
      {:else if children}
        {#each children as child (child.dn)}
          <DitNode node={child} {selectedDn} {onselect} />
        {/each}
      {/if}
    </ul>
  {/if}
</li>

<style>
  li {
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.1rem;
    padding: 0.15rem 0.3rem;
    border-radius: 3px;
  }

  .row.selected {
    background: light-dark(#e3f0ff, #1b2d44);
  }

  .chevron,
  .label {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
    text-align: left;
  }

  .chevron {
    width: 1.1rem;
    color: light-dark(#888, #666);
  }

  .label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex: 1;
    min-width: 0;
  }

  .label:hover {
    text-decoration: underline;
  }

  .icon {
    font-size: 0.9em;
  }

  .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .children {
    padding: 0 0 0 1rem;
    margin: 0;
    border-left: 1px dashed light-dark(#ddd, #333);
  }

  .status {
    padding: 0.2rem 0.5rem;
    font-size: 0.85rem;
    color: light-dark(#666, #888);
  }

  .status.error {
    color: #c0392b;
  }

  .status.muted {
    font-style: italic;
  }
</style>
