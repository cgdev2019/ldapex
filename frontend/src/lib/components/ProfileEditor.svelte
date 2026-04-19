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
</script>

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-label={isNew ? $_('profile.editor.new_title') : $_('profile.editor.edit_title', { values: { name: profile?.name ?? '' } })}
>
  <div class="dialog">
    <h2>
      {isNew
        ? $_('profile.editor.new_title')
        : $_('profile.editor.edit_title', { values: { name: profile?.name ?? '' } })}
    </h2>

    <form onsubmit={submit}>
      <label>
        <span>{$_('profile.editor.name')}</span>
        <input type="text" bind:value={name} required />
      </label>

      <label>
        <span>{$_('profile.editor.url')}</span>
        <input
          type="text"
          bind:value={url}
          required
          placeholder={$_('profile.editor.url_placeholder')}
          spellcheck="false"
        />
      </label>

      <label>
        <span>{$_('profile.editor.bind_dn')}</span>
        <input type="text" bind:value={bindDn} spellcheck="false" />
      </label>

      <label>
        <span>{$_('profile.editor.base_dn')}</span>
        <input type="text" bind:value={baseDn} required spellcheck="false" />
      </label>

      <label>
        <span>TLS</span>
        <select bind:value={tls}>
          <option value="none">{$_('profile.editor.tls_none')}</option>
          <option value="start_tls">{$_('profile.editor.tls_starttls')}</option>
          <option value="ldaps">{$_('profile.editor.tls_ldaps')}</option>
        </select>
      </label>

      <label>
        <span>{$_('profile.editor.password_label')}</span>
        <input
          type="password"
          bind:value={password}
          autocomplete="new-password"
          placeholder={profile?.has_saved_password ? $_('profile.editor.password_placeholder_stored') : ''}
        />
        <p class="hint">{$_('profile.editor.password_hint')}</p>
      </label>

      {#if error}
        <p class="status error">{error}</p>
      {/if}

      <div class="actions">
        <button type="submit" disabled={saving}>
          {saving ? $_('profile.editor.saving') : $_('profile.editor.save')}
        </button>
        <button type="button" class="secondary" onclick={onclose} disabled={saving}>
          {$_('common.cancel')}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 100;
  }

  .dialog {
    background: light-dark(#fff, #161616);
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    width: min(90vw, 32rem);
    max-height: 90vh;
    overflow: auto;
  }

  h2 {
    margin: 0 0 0.75rem 0;
    font-size: 1.1rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.9rem;
  }

  label > span {
    color: light-dark(#555, #aaa);
  }

  .hint {
    margin: 0;
    font-size: 0.75rem;
    color: light-dark(#777, #888);
  }

  input[type='text'],
  input[type='password'],
  select {
    font: inherit;
    padding: 0.4rem 0.55rem;
    border: 1px solid light-dark(#ccc, #333);
    background: light-dark(#fff, #0e0e0e);
    color: inherit;
    border-radius: 4px;
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', monospace;
    background: light-dark(#eee, #222);
    padding: 0 0.2em;
    border-radius: 3px;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .secondary {
    background: transparent;
  }

  .status {
    margin: 0;
  }

  .status.error {
    color: #c0392b;
  }
</style>
