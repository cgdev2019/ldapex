<script lang="ts">
  import LoginForm from '$lib/components/LoginForm.svelte';
  import DitTree from '$lib/components/DitTree.svelte';
  import EntryPanel from '$lib/components/EntryPanel.svelte';
  import { session } from '$lib/session.svelte';

  let selectedDn = $state<string | null>(null);

  function onselect(dn: string) {
    selectedDn = dn;
  }

  async function onDisconnect() {
    await session.disconnect();
    selectedDn = null;
  }
</script>

{#if !session.connected || !session.baseDn}
  <LoginForm />
{:else}
  <header class="topbar">
    <strong>{session.bindDn ?? '(anonyme)'}</strong>
    <span class="url">@ {session.url}</span>
    <button type="button" onclick={onDisconnect}>Déconnexion</button>
  </header>

  <div class="layout">
    <aside class="tree">
      <DitTree baseDn={session.baseDn} {selectedDn} {onselect} />
    </aside>
    <main class="detail">
      <EntryPanel dn={selectedDn} />
    </main>
  </div>
{/if}

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid light-dark(#ddd, #333);
    font-size: 0.9rem;
  }

  .topbar .url {
    color: light-dark(#666, #888);
    flex: 1;
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(14rem, 22rem) 1fr;
    height: calc(100vh - 3rem);
  }

  .tree {
    border-right: 1px solid light-dark(#ddd, #333);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .detail {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
