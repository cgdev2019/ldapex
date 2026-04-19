<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { formatError, ldapListChildren, type DnLabel } from '$lib/bridge';
  import type { IconName } from './Icon.svelte';
  import Icon from './Icon.svelte';
  import DitNode from './DitNode.svelte';

  interface Props {
    node: DnLabel;
    selectedDn: string | null;
    onselect: (dn: string) => void;
    depth?: number;
  }

  let { node, selectedDn, onselect, depth = 0 }: Props = $props();

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

  function nodeIcon(): IconName {
    const oc = node.object_classes.map((c) => c.toLowerCase());
    if (oc.includes('organizationalunit')) return 'folder';
    if (oc.includes('inetorgperson') || oc.includes('person') || oc.includes('account')) return 'user';
    if (
      oc.includes('groupofnames') ||
      oc.includes('groupofuniquenames') ||
      oc.includes('posixgroup')
    )
      return 'users';
    if (oc.includes('dcobject') || oc.includes('organization') || oc.includes('country'))
      return 'building';
    return 'circle-dot';
  }

  const isSelected = $derived(selectedDn === node.dn);
</script>

<li>
  <div class="row" class:selected={isSelected}>
    <button
      type="button"
      class="chevron"
      onclick={toggle}
      aria-label={expanded ? $_('tree.collapse') : $_('tree.expand')}
      aria-expanded={expanded}
    >
      <Icon name={expanded ? 'chevron-down' : 'chevron-right'} size={12} />
    </button>
    <button
      type="button"
      class="label"
      onclick={() => onselect(node.dn)}
      title={node.dn}
    >
      <span class="icon" aria-hidden="true">
        <Icon name={nodeIcon()} size={14} />
      </span>
      <span class="text">{node.label}</span>
    </button>
  </div>

  {#if expanded}
    <ul class="children" style:--depth={depth + 1}>
      {#if loading}
        <li class="status">
          <Icon name="refresh" size={11} />
          <span>{$_('tree.loading')}</span>
        </li>
      {:else if error}
        <li class="status error">{error}</li>
      {:else if children && children.length === 0}
        <li class="status muted">{$_('tree.empty_children')}</li>
      {:else if children}
        {#each children as child (child.dn)}
          <DitNode node={child} {selectedDn} {onselect} depth={depth + 1} />
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
    padding: 0.15rem 0.25rem;
    border-radius: var(--radius-sm);
    gap: 0.05rem;
    transition: background var(--transition-fast);
  }

  .row:hover {
    background: var(--color-surface-hover);
  }

  .row.selected {
    background: var(--color-primary-soft);
  }

  .row.selected .label {
    color: var(--color-primary);
    font-weight: 500;
  }

  .chevron {
    width: 1.25rem;
    height: 1.25rem;
    padding: 0;
    background: transparent;
    border: none;
    color: var(--color-text-subtle);
    border-radius: var(--radius-sm);
  }

  .chevron:hover:not(:disabled) {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .label {
    flex: 1;
    padding: 0.15rem 0.35rem;
    background: transparent;
    border: none;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
    border-radius: var(--radius-sm);
  }

  .label:hover {
    background: transparent;
  }

  .icon {
    color: var(--color-text-subtle);
    display: inline-flex;
  }

  .row.selected .icon {
    color: var(--color-primary);
  }

  .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-sm);
  }

  .children {
    padding: 0;
    margin: 0 0 0 0.7rem;
    border-left: 1px dashed var(--color-border);
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.6rem;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .status.error {
    color: var(--color-danger);
  }

  .status.muted {
    font-style: italic;
    color: var(--color-text-subtle);
  }
</style>
