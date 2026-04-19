<script lang="ts">
  import { formatError, ldapReadEntry, type DnLabel } from '$lib/bridge';
  import DitNode from './DitNode.svelte';

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
      // Fabricate a DnLabel from the base entry so the tree component
      // sees a single, always-expandable root.
      const objectClasses = entry.attributes
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

<nav aria-label="Arborescence LDAP">
  {#if loading}
    <p class="status">Chargement de la base…</p>
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
    padding: 0.5rem;
    font-size: 0.9rem;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .status {
    padding: 0.5rem;
    color: light-dark(#666, #888);
    margin: 0;
  }

  .status.error {
    color: #c0392b;
  }
</style>
