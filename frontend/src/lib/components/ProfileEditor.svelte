<script lang="ts">
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import {
    formatError,
    profileSave,
    type ConnectionProfile,
    type ProfileSummary,
    type TlsMode
  } from '$lib/bridge';
  import Icon from './Icon.svelte';

  interface Props {
    profile?: ProfileSummary | null;
    onclose: () => void;
    onsaved: (profile: ProfileSummary) => void;
  }

  let { profile = null, onclose, onsaved }: Props = $props();

  const isNew = profile === null;
  let name = $state(profile?.name ?? get(_)('profile.editor.default_name'));
  let url = $state(profile?.url ?? 'ldap://127.0.0.1:389');
  let bindDn = $state(profile?.bind_dn ?? '');
  let baseDn = $state(profile?.base_dn ?? '');
  let tls = $state<TlsMode>(profile?.tls ?? 'none');
  let password = $state(profile?.password ?? '');
  let error = $state<string | null>(null);
  let saving = $state(false);
  let showPassword = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const id = profile?.id ?? crypto.randomUUID();
      const trimmedPassword = password.trim();
      const next: ConnectionProfile = {
        id,
        name: name.trim(),
        url: url.trim(),
        bind_dn: bindDn.trim(),
        base_dn: baseDn.trim(),
        tls,
        timeout_secs: profile?.timeout_secs ?? 30,
        password: trimmedPassword.length > 0 ? trimmedPassword : null
      };
      const saved = await profileSave(next);
      onsaved(saved);
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget && !saving) onclose();
  }
</script>

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-label={isNew
    ? $_('profile.editor.new_title')
    : $_('profile.editor.edit_title', { values: { name: profile?.name ?? '' } })}
  onclick={handleBackdrop}
  onkeydown={(e) => e.key === 'Escape' && !saving && onclose()}
  tabindex="-1"
>
  <div class="dialog">
    <header>
      <h2>
        <Icon name={isNew ? 'plus' : 'pencil'} size={16} />
        <span>
          {isNew
            ? $_('profile.editor.new_title')
            : $_('profile.editor.edit_title', { values: { name: profile?.name ?? '' } })}
        </span>
      </h2>
      <button
        type="button"
        class="ghost icon-only"
        onclick={onclose}
        disabled={saving}
        aria-label={$_('common.close')}
      >
        <Icon name="x" size={15} />
      </button>
    </header>

    <form onsubmit={submit}>
      <div class="body">
        <label>
          <span>{$_('profile.editor.name')}</span>
          <input type="text" bind:value={name} required />
        </label>

        <div class="row">
          <label class="flex">
            <span>{$_('profile.editor.url')}</span>
            <input
              type="text"
              bind:value={url}
              required
              placeholder={$_('profile.editor.url_placeholder')}
              spellcheck="false"
            />
          </label>
          <label class="w-tls">
            <span>TLS</span>
            <select bind:value={tls}>
              <option value="none">{$_('profile.editor.tls_none')}</option>
              <option value="start_tls">{$_('profile.editor.tls_starttls')}</option>
              <option value="ldaps">{$_('profile.editor.tls_ldaps')}</option>
            </select>
          </label>
        </div>

        <label>
          <span>{$_('profile.editor.bind_dn')}</span>
          <input type="text" bind:value={bindDn} spellcheck="false" />
        </label>

        <label>
          <span>{$_('profile.editor.base_dn')}</span>
          <input type="text" bind:value={baseDn} required spellcheck="false" />
        </label>

        <label>
          <span>{$_('profile.editor.password_label')}</span>
          <div class="pw-wrap">
            <input
              type={showPassword ? 'text' : 'password'}
              bind:value={password}
              autocomplete="new-password"
              placeholder={profile?.has_saved_password
                ? $_('profile.editor.password_placeholder_stored')
                : ''}
            />
            <button
              type="button"
              class="ghost icon-only"
              onclick={() => (showPassword = !showPassword)}
              aria-label={showPassword ? 'Hide' : 'Show'}
              tabindex="-1"
            >
              <Icon name={showPassword ? 'eye-off' : 'eye'} size={14} />
            </button>
          </div>
          <p class="hint">
            <Icon name="alert-triangle" size={11} />
            <span>{$_('profile.editor.password_hint')}</span>
          </p>
        </label>

        {#if error}
          <p class="status error">{error}</p>
        {/if}
      </div>

      <footer class="actions">
        <button type="button" class="ghost" onclick={onclose} disabled={saving}>
          {$_('common.cancel')}
        </button>
        <button type="submit" class="primary" disabled={saving}>
          <Icon name={saving ? 'refresh' : 'save'} size={14} />
          <span>{saving ? $_('profile.editor.saving') : $_('profile.editor.save')}</span>
        </button>
      </footer>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 55%, transparent);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    z-index: 200;
    padding: 1.25rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
    width: min(92vw, 34rem);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.2rem;
    border-bottom: 1px solid var(--color-border);
  }

  header h2 {
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
  }

  form {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .body {
    padding: 1rem 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    overflow: auto;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: var(--text-sm);
  }

  label > span {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .row {
    display: flex;
    gap: 0.75rem;
  }

  .flex {
    flex: 1;
  }

  .w-tls {
    width: 12rem;
    flex-shrink: 0;
  }

  .pw-wrap {
    position: relative;
  }

  .pw-wrap input {
    padding-right: 2.4rem;
  }

  .pw-wrap :global(button.icon-only) {
    position: absolute;
    right: 0.2rem;
    top: 50%;
    transform: translateY(-50%);
    width: 1.9rem;
    height: 1.9rem;
    color: var(--color-text-subtle);
  }

  .hint {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: var(--text-xs);
    color: var(--color-text-subtle);
  }

  .status.error {
    color: var(--color-danger);
    background: var(--color-danger-soft);
    border: 1px solid color-mix(in oklab, var(--color-danger) 25%, transparent);
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  footer.actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding: 0.8rem 1.2rem;
    background: var(--color-surface-2);
    border-top: 1px solid var(--color-border);
  }
</style>
