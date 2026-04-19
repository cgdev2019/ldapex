<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    profileDelete,
    profileExport,
    profileImport,
    profileList,
    type ProfileSummary
  } from '$lib/bridge';
  import { session } from '$lib/session.svelte';
  import Icon from './Icon.svelte';
  import ProfileEditor from './ProfileEditor.svelte';

  let profiles = $state<ProfileSummary[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editing = $state<ProfileSummary | null>(null);
  let creating = $state(false);

  let passwordPromptFor = $state<ProfileSummary | null>(null);
  let pendingPassword = $state('');
  let pendingRemember = $state(false);

  $effect(() => {
    reload();
  });

  async function reload() {
    loading = true;
    error = null;
    try {
      profiles = await profileList();
    } catch (err) {
      error = formatError(err);
    } finally {
      loading = false;
    }
  }

  async function connect(profile: ProfileSummary) {
    if (!profile.has_saved_password) {
      passwordPromptFor = profile;
      pendingPassword = '';
      pendingRemember = false;
      return;
    }
    try {
      await session.connectWithProfile({ id: profile.id });
    } catch (err) {
      error = formatError(err);
    }
  }

  async function submitPassword(event: SubmitEvent) {
    event.preventDefault();
    if (!passwordPromptFor) return;
    const target = passwordPromptFor;
    try {
      await session.connectWithProfile({
        id: target.id,
        password: pendingPassword,
        remember: pendingRemember
      });
      passwordPromptFor = null;
      pendingPassword = '';
    } catch (err) {
      error = formatError(err);
    }
  }

  async function confirmDelete(profile: ProfileSummary) {
    const t = get(_);
    const yes = window.confirm(
      t('profile.picker.confirm_delete', { values: { name: profile.name } })
    );
    if (!yes) return;
    try {
      await profileDelete(profile.id);
      await reload();
    } catch (err) {
      error = formatError(err);
    }
  }

  async function doExport() {
    try {
      const json = await profileExport();
      await navigator.clipboard.writeText(json);
      window.alert(get(_)('profile.picker.export_ok'));
    } catch (err) {
      error = formatError(err);
    }
  }

  async function doImport() {
    const raw = window.prompt(get(_)('profile.picker.import_prompt'));
    if (!raw) return;
    try {
      await profileImport(raw);
      await reload();
    } catch (err) {
      error = formatError(err);
    }
  }

  function initials(name: string): string {
    const parts = name.trim().split(/\s+/);
    if (parts.length === 0 || !parts[0]) return '·';
    if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }

  function tlsLabel(tls: ProfileSummary['tls']): string {
    if (tls === 'ldaps') return 'LDAPS';
    if (tls === 'start_tls') return 'STARTTLS';
    return 'PLAIN';
  }
</script>

<section class="card">
  <header>
    <div>
      <h2>{$_('profile.picker.title')}</h2>
      <p class="hint">{profiles.length} {profiles.length === 1 ? 'profile' : 'profiles'}</p>
    </div>
    <div class="tools">
      <button type="button" class="ghost sm" onclick={doImport}>
        <Icon name="import" size={14} />
        <span>{$_('profile.picker.import')}</span>
      </button>
      <button
        type="button"
        class="ghost sm"
        onclick={doExport}
        disabled={profiles.length === 0}
      >
        <Icon name="export" size={14} />
        <span>{$_('profile.picker.export')}</span>
      </button>
      <button type="button" class="primary sm" onclick={() => (creating = true)}>
        <Icon name="plus" size={14} />
        <span>{$_('profile.picker.new')}</span>
      </button>
    </div>
  </header>

  <div class="body">
    {#if loading}
      <p class="status">{$_('common.loading')}</p>
    {:else if error}
      <p class="status error">{error}</p>
    {/if}

    {#if profiles.length > 0}
      <ul>
        {#each profiles as p (p.id)}
          <li>
            <button
              type="button"
              class="row"
              onclick={() => connect(p)}
              disabled={session.connecting}
            >
              <span class="avatar" aria-hidden="true">{initials(p.name)}</span>
              <span class="lines">
                <span class="line1">
                  <strong>{p.name}</strong>
                  <span class="tls" data-mode={p.tls}>{tlsLabel(p.tls)}</span>
                  {#if p.has_saved_password}
                    <span
                      class="lock"
                      title={$_('profile.picker.password_stored')}
                      aria-label={$_('profile.picker.password_stored')}
                    >
                      <Icon name="key" size={12} />
                    </span>
                  {/if}
                </span>
                <span class="line2">
                  <Icon name="user" size={11} />
                  <span>{p.bind_dn || $_('common.anonymous')}</span>
                  <span class="dot">·</span>
                  <span class="mono">{p.url}</span>
                </span>
                <span class="line3">
                  <Icon name="database" size={11} />
                  <span class="mono">{p.base_dn}</span>
                </span>
              </span>
            </button>
            <div class="actions">
              <button
                type="button"
                class="ghost icon-only"
                onclick={() => (editing = p)}
                title={$_('common.edit')}
                aria-label={$_('common.edit')}
              >
                <Icon name="pencil" size={14} />
              </button>
              <button
                type="button"
                class="ghost icon-only danger"
                onclick={() => confirmDelete(p)}
                title={$_('common.delete')}
                aria-label={$_('common.delete')}
              >
                <Icon name="trash" size={14} />
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {:else if !loading}
      <div class="empty">
        <Icon name="list" size={22} />
        <p>{$_('profile.picker.no_profiles')}</p>
      </div>
    {/if}
  </div>
</section>

{#if creating}
  <ProfileEditor
    onclose={() => (creating = false)}
    onsaved={async () => {
      creating = false;
      await reload();
    }}
  />
{/if}

{#if editing}
  <ProfileEditor
    profile={editing}
    onclose={() => (editing = null)}
    onsaved={async () => {
      editing = null;
      await reload();
    }}
  />
{/if}

{#if passwordPromptFor}
  <div class="backdrop" role="dialog" aria-modal="true" aria-label={$_('login.password')}>
    <form class="pw-dialog" onsubmit={submitPassword}>
      <h3>
        {$_('profile.picker.password_dialog_title', {
          values: { name: passwordPromptFor.name }
        })}
      </h3>
      <p class="bind">{passwordPromptFor.bind_dn || $_('common.anonymous')}</p>

      <input
        type="password"
        bind:value={pendingPassword}
        autocomplete="current-password"
        autofocus
        required
      />

      <label class="inline">
        <input type="checkbox" bind:checked={pendingRemember} />
        <span>{$_('profile.picker.password_dialog_remember')}</span>
      </label>

      <div class="actions row-actions">
        <button
          type="button"
          class="ghost"
          onclick={() => (passwordPromptFor = null)}
          disabled={session.connecting}
        >
          {$_('common.cancel')}
        </button>
        <button type="submit" class="primary" disabled={session.connecting}>
          {session.connecting ? $_('common.connecting') : $_('common.connect')}
        </button>
      </div>
    </form>
  </div>
{/if}

<style>
  .card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-md);
    overflow: hidden;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 1rem;
    padding: 1.1rem 1.5rem 0.9rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  header h2 {
    font-size: 1.05rem;
  }

  .hint {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    margin-top: 0.15rem;
  }

  .tools {
    display: flex;
    gap: 0.4rem;
  }

  .body {
    padding: 0.75rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.4rem 0.5rem;
    border-radius: var(--radius-lg);
    transition: background var(--transition-fast);
  }

  li:hover {
    background: var(--color-surface-hover);
  }

  .row {
    flex: 1;
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: inherit;
    text-align: left;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.7rem;
    min-width: 0;
    border-radius: var(--radius-md);
  }

  .row:hover:not(:disabled) {
    background: transparent;
  }

  .row[disabled] {
    cursor: default;
    opacity: 0.6;
  }

  .avatar {
    display: grid;
    place-items: center;
    width: 2.1rem;
    height: 2.1rem;
    flex-shrink: 0;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, #4f46e5 0%, #2563eb 100%);
    color: white;
    font-weight: 700;
    font-size: 0.75rem;
    letter-spacing: 0.02em;
    box-shadow: var(--shadow-sm);
  }

  .lines {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
    flex: 1;
  }

  .line1 {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    font-size: var(--text-base);
    color: var(--color-text);
  }

  .line1 strong {
    font-weight: 600;
  }

  .tls {
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 0.05rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
  }

  .tls[data-mode='ldaps'],
  .tls[data-mode='start_tls'] {
    background: color-mix(in oklab, var(--color-success) 18%, transparent);
    color: var(--color-success);
    border-color: color-mix(in oklab, var(--color-success) 35%, transparent);
  }

  .lock {
    display: inline-flex;
    align-items: center;
    color: var(--color-warning);
  }

  .line2,
  .line3 {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .line2 .dot {
    color: var(--color-text-subtle);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    display: flex;
    gap: 0.2rem;
  }

  :global(button.icon-only.danger:hover:not(:disabled)) {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .status {
    margin: 0;
    padding: 0.5rem 0.75rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .status.error {
    color: var(--color-danger);
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
    border-radius: var(--radius-md);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.6rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-align: center;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 50%, transparent);
    display: grid;
    place-items: center;
    z-index: 200;
    backdrop-filter: blur(4px);
  }

  .pw-dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    padding: 1.3rem 1.4rem;
    width: min(90vw, 24rem);
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
    box-shadow: var(--shadow-lg);
  }

  .pw-dialog h3 {
    font-size: 1rem;
  }

  .bind {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .inline {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: var(--text-sm);
  }

  .inline input {
    width: auto;
  }

  .row-actions {
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.2rem;
  }
</style>
