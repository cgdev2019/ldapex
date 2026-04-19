<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { formatError, ldapReadEntry, type DnLabel } from '$lib/bridge';
  import DitNode from './DitNode.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    baseDn: string;
    selectedDn: string | null;
    onselect: (dn: string) => void;
  }

  let { baseDn, selectedDn, onselect }: Props = $props();

  let root = $state<DnLabel | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    loadRoot(baseDn);
  });

  async function loadRoot(dn: string) {
    loading = true;
    error = null;
    try {
      const entry = await ldapReadEntry(dn);
      const objectClasses =
        entry.attributes
          .find((a) => a.name.toLowerCase() === 'objectclass')
          ?.values.filter((v) => v.kind === 'text')
          .map((v) => (v as { kind: 'text'; data: string }).data) ?? [];
      root = {
        dn: entry.dn,
        rdn: entry.dn.split(',')[0] ?? entry.dn,
        label: entry.dn,
        object_classes: objectClasses,
        has_children: true
      };
    } catch (err) {
      error = formatError(err);
      root = null;
    } finally {
      loading = false;
    }
  }
</script>

<nav aria-label={$_('tree.label')}>
  {#if loading}
    <p class="status">
      <Icon name="refresh" size={13} />
      <span>{$_('tree.loading_base')}</span>
    </p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if root}
    <ul>
      <DitNode node={root} {selectedDn} {onselect} />
    </ul>
  {/if}
</nav>

<style>
  nav {
    overflow: auto;
    padding: 0.5rem 0.5rem;
    font-size: var(--text-sm);
    flex: 1;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .status.error {
    color: var(--color-danger);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    word-break: break-word;
  }
</style>
