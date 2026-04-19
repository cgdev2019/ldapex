<script lang="ts">
  import { _, locale } from 'svelte-i18n';
  import BookmarksPanel from '$lib/components/BookmarksPanel.svelte';
  import CommandPalette, {
    type AppAction
  } from '$lib/components/CommandPalette.svelte';
  import CreateEntryDialog from '$lib/components/CreateEntryDialog.svelte';
  import DitTree from '$lib/components/DitTree.svelte';
  import EntryPanel from '$lib/components/EntryPanel.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import LoginForm from '$lib/components/LoginForm.svelte';
  import ProfilePicker from '$lib/components/ProfilePicker.svelte';
  import SchemaPanel from '$lib/components/SchemaPanel.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import { bookmarks, recents } from '$lib/bookmarks.svelte';
  import { setLocale, type SupportedLocale } from '$lib/i18n';
  import { registerShortcuts } from '$lib/shortcuts.svelte';
  import { session } from '$lib/session.svelte';

  let selectedDn = $state<string | null>(null);
  let sidePanel = $state<'browse' | 'search' | 'bookmarks' | 'schema'>('browse');
  let creatingUnder = $state<string | null>(null);
  let treeRefreshKey = $state(0);
  let paletteOpen = $state(false);

  $effect(() => {
    // Refresh per-profile bookmarks + recent DNs whenever the active
    // session changes (including reconnecting with a different profile).
    if (session.connected) {
      bookmarks.reload();
      recents.reload();
    }
  });

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
      onSave: () => window.dispatchEvent(new Event('ldapex:save')),
      onDelete: () => window.dispatchEvent(new Event('ldapex:delete')),
      onCommandPalette: () => (paletteOpen = true)
    });
  });

  function runAction(action: AppAction) {
    switch (action) {
      case 'new-entry':
        openCreate();
        break;
      case 'switch-tree':
        sidePanel = 'browse';
        break;
      case 'switch-search':
        sidePanel = 'search';
        break;
      case 'switch-bookmarks':
        sidePanel = 'bookmarks';
        break;
      case 'switch-schema':
        sidePanel = 'schema';
        break;
      case 'refresh':
        treeRefreshKey += 1;
        break;
      case 'disconnect':
        void onDisconnect();
        break;
    }
  }

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
    <header class="brand">
      <img class="logo lg" src="/ldapex-logo.png" alt="Ldapex" width="52" height="52" />
      <div class="brand-text">
        <h1>Ldapex</h1>
        <p>LDAP directory browser</p>
      </div>
      <div class="brand-spacer"></div>
      <select
        class="lang"
        aria-label={$_('language.label')}
        value={$locale?.startsWith('fr') ? 'fr' : 'en'}
        onchange={(e) =>
          setLocale((e.currentTarget as HTMLSelectElement).value as SupportedLocale)}
      >
        <option value="en">EN</option>
        <option value="fr">FR</option>
      </select>
    </header>
    <ProfilePicker />
    <LoginForm />
  </div>
{:else}
  <header class="topbar">
    <div class="brand-chip">
      <img class="logo sm" src="/ldapex-logo.png" alt="" width="26" height="26" />
      <span class="brand-name">Ldapex</span>
    </div>

    <div class="session-info" title={session.url ?? ''}>
      <Icon name="user" size={14} />
      <span class="dn">{session.bindDn || $_('common.anonymous')}</span>
      <span class="sep">@</span>
      <span class="url">{session.url}</span>
    </div>

    <div class="topbar-actions">
      <button
        type="button"
        class="ghost palette-btn"
        onclick={() => (paletteOpen = true)}
        title={$_('palette.open_tooltip')}
        aria-label={$_('palette.open_tooltip')}
      >
        <Icon name="command" size={14} />
        <kbd>Ctrl K</kbd>
      </button>
      <button type="button" class="primary" onclick={openCreate} title={$_('nav.new_entry_tooltip')}>
        <Icon name="plus" size={15} />
        <span>{$_('nav.new_entry')}</span>
      </button>
      <select
        class="lang"
        aria-label={$_('language.label')}
        value={$locale?.startsWith('fr') ? 'fr' : 'en'}
        onchange={(e) =>
          setLocale((e.currentTarget as HTMLSelectElement).value as SupportedLocale)}
      >
        <option value="en">EN</option>
        <option value="fr">FR</option>
      </select>
      <button type="button" class="ghost" onclick={onDisconnect} title={$_('nav.disconnect')}>
        <Icon name="log-out" size={15} />
        <span class="hide-sm">{$_('nav.disconnect')}</span>
      </button>
    </div>
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
          <Icon name="folder" size={14} />
          <span>{$_('tabs.tree')}</span>
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={sidePanel === 'search'}
          class:active={sidePanel === 'search'}
          onclick={() => (sidePanel = 'search')}
          title={$_('tabs.search_tooltip')}
        >
          <Icon name="search" size={14} />
          <span>{$_('tabs.search')}</span>
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={sidePanel === 'bookmarks'}
          class:active={sidePanel === 'bookmarks'}
          onclick={() => (sidePanel = 'bookmarks')}
        >
          <Icon name="star" size={14} />
          <span>{$_('tabs.bookmarks')}</span>
          {#if bookmarks.items.length > 0}
            <span class="tab-count">{bookmarks.items.length}</span>
          {/if}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={sidePanel === 'schema'}
          class:active={sidePanel === 'schema'}
          onclick={() => (sidePanel = 'schema')}
        >
          <Icon name="database" size={14} />
          <span>{$_('tabs.schema')}</span>
        </button>
      </nav>

      <div class="side-body">
        {#if sidePanel === 'browse'}
          {#key treeRefreshKey}
            <DitTree baseDn={session.baseDn} {selectedDn} {onselect} />
          {/key}
        {:else if sidePanel === 'search'}
          <SearchPanel baseDn={session.baseDn} {onselect} />
        {:else if sidePanel === 'bookmarks'}
          <BookmarksPanel {selectedDn} {onselect} />
        {:else}
          <SchemaPanel />
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

  {#if paletteOpen}
    <CommandPalette
      onclose={() => (paletteOpen = false)}
      onnavigate={(dn) => (selectedDn = dn)}
      onaction={runAction}
    />
  {/if}
{/if}

<style>
  .login-view {
    max-width: 44rem;
    margin: 0 auto;
    padding: 2.5rem 1.5rem 3rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    padding: 0 0.25rem;
  }

  .brand h1 {
    font-size: 1.6rem;
    letter-spacing: -0.015em;
  }

  .brand p {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    margin-top: 0.15rem;
  }

  .brand-spacer {
    flex: 1;
  }

  .logo {
    display: block;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
    object-fit: contain;
    background: transparent;
  }

  .logo.lg {
    width: 3.2rem;
    height: 3.2rem;
  }

  .logo.sm {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: var(--radius-md);
    box-shadow: none;
  }

  .topbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.55rem 1rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: var(--text-sm);
    min-height: 3rem;
  }

  .brand-chip {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
  }

  .session-info {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    color: var(--color-text-muted);
    min-width: 0;
    overflow: hidden;
  }

  .session-info .dn {
    color: var(--color-text);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .session-info .sep {
    color: var(--color-text-subtle);
  }

  .session-info .url {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .palette-btn {
    color: var(--color-text-muted);
    padding-left: 0.55rem;
    padding-right: 0.55rem;
  }

  .palette-btn kbd {
    font-size: 0.65rem;
  }

  .lang {
    width: auto;
    padding-top: 0.3rem;
    padding-bottom: 0.3rem;
    padding-left: 0.55rem;
    font-size: var(--text-sm);
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(16rem, 22rem) 1fr;
    height: calc(100vh - 3rem);
    min-height: 0;
  }

  .side {
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tabs {
    display: flex;
    padding: 0.4rem 0.4rem 0;
    gap: 0.2rem;
    border-bottom: 1px solid var(--color-border);
  }

  .tabs button {
    flex: 1;
    gap: 0.35rem;
    background: transparent;
    border: none;
    padding: 0.55rem 0.4rem;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    color: var(--color-text-muted);
    cursor: pointer;
    font-weight: 500;
    position: relative;
  }

  .tabs button:hover:not(.active) {
    color: var(--color-text);
    background: var(--color-surface-hover);
  }

  .tabs button.active {
    color: var(--color-primary);
  }

  .tabs button.active::after {
    content: '';
    position: absolute;
    left: 0.4rem;
    right: 0.4rem;
    bottom: -1px;
    height: 2px;
    background: var(--color-primary);
    border-radius: 2px;
  }

  .tab-count {
    font-size: 0.6rem;
    padding: 0.02rem 0.35rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary);
    color: white;
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
    background: var(--color-bg);
  }

  @media (max-width: 680px) {
    .hide-sm {
      display: none;
    }
    .session-info .url {
      display: none;
    }
  }
</style>
