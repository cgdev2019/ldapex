<script lang="ts">
  import { _, locale } from 'svelte-i18n';
  import CreateEntryDialog from '$lib/components/CreateEntryDialog.svelte';
  import DitTree from '$lib/components/DitTree.svelte';
  import EntryPanel from '$lib/components/EntryPanel.svelte';
  import LoginForm from '$lib/components/LoginForm.svelte';
  import ProfilePicker from '$lib/components/ProfilePicker.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import { setLocale, type SupportedLocale } from '$lib/i18n';
  import { registerShortcuts } from '$lib/shortcuts.svelte';
  import { session } from '$lib/session.svelte';

  let selectedDn = $state<string | null>(null);
  let sidePanel = $state<'browse' | 'search'>('browse');
  let creatingUnder = $state<string | null>(null);
  let treeRefreshKey = $state(0);

  $effect(() => {
    if (!session.connected) return;
    return registerShortcuts({
      onFocusSearch: () => {
        sidePanel = 'search';
      },
      onNewEntry: openCreate,
      onRefresh: () => {
        treeRefreshKey += 1;
      },
      // Save and Delete are routed to EntryPanel via window events
      // so the component stays authoritative on "am I editing?" and
      // "which DN is selected?".
      onSave: () => window.dispatchEvent(new Event('ldapex:save')),
      onDelete: () => window.dispatchEvent(new Event('ldapex:delete'))
    });
  });

  function onselect(dn: string) {
    selectedDn = dn;
  }

  async function onDisconnect() {
    await session.disconnect();
    selectedDn = null;
    creatingUnder = null;
  }

  function openCreate() {
    creatingUnder = selectedDn ?? session.baseDn;
  }

  function onCreated(dn: string) {
    selectedDn = dn;
    treeRefreshKey += 1;
  }

  function onEntryChanged(e: { dn: string; kind: 'modified' | 'deleted' }) {
    if (e.kind === 'deleted') {
      selectedDn = null;
    }
    treeRefreshKey += 1;
  }
</script>

{#if !session.connected || !session.baseDn}
  <div class="login-view">
    <ProfilePicker />
    <LoginForm />
  </div>
{:else}
  <header class="topbar">
    <strong>{session.bindDn || $_('common.anonymous')}</strong>
    <span class="url">@ {session.url}</span>
    <button type="button" onclick={openCreate} title={$_('nav.new_entry_tooltip')}>
      {$_('nav.new_entry')}
    </button>
    <select
      class="lang"
      aria-label={$_('language.label')}
      value={$locale?.startsWith('fr') ? 'fr' : 'en'}
      onchange={(e) => setLocale((e.currentTarget as HTMLSelectElement).value as SupportedLocale)}
    >
      <option value="en">EN</option>
      <option value="fr">FR</option>
    </select>
    <button type="button" onclick={onDisconnect}>{$_('nav.disconnect')}</button>
  </header>

  <div class="layout">
    <aside class="side">
      <nav class="tabs" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={sidePanel === 'browse'}
          class:active={sidePanel === 'browse'}
          onclick={() => (sidePanel = 'browse')}
        >
          {$_('tabs.tree')}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={sidePanel === 'search'}
          class:active={sidePanel === 'search'}
          onclick={() => (sidePanel = 'search')}
          title={$_('tabs.search_tooltip')}
        >
          {$_('tabs.search')}
        </button>
      </nav>

      <div class="side-body">
        {#if sidePanel === 'browse'}
          {#key treeRefreshKey}
            <DitTree baseDn={session.baseDn} {selectedDn} {onselect} />
          {/key}
        {:else}
          <SearchPanel baseDn={session.baseDn} {onselect} />
        {/if}
      </div>
    </aside>

    <main class="detail">
      <EntryPanel dn={selectedDn} onentrychanged={onEntryChanged} />
    </main>
  </div>

  {#if creatingUnder}
    <CreateEntryDialog
      parentDn={creatingUnder}
      onclose={() => (creatingUnder = null)}
      oncreated={onCreated}
    />
  {/if}
{/if}

<style>
  .login-view {
    padding: 1rem 0.5rem;
  }

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
    grid-template-columns: minmax(16rem, 24rem) 1fr;
    height: calc(100vh - 3rem);
  }

  .side {
    border-right: 1px solid light-dark(#ddd, #333);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid light-dark(#ddd, #333);
  }

  .tabs button {
    flex: 1;
    background: transparent;
    border: none;
    padding: 0.5rem;
    border-bottom: 2px solid transparent;
    color: inherit;
    cursor: pointer;
    font: inherit;
  }

  .tabs button.active {
    border-bottom-color: light-dark(#0057b7, #7aaeff);
    font-weight: 600;
  }

  .side-body {
    flex: 1;
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
