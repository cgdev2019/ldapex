<script lang="ts">
  import {
    formatError,
    profileClearPassword,
    profileSave,
    profileSetPassword,
    type ConnectionProfile,
    type ProfileSummary,
    type TlsMode
  } from '$lib/bridge';

  interface Props {
    profile?: ProfileSummary | null;
    onclose: () => void;
    onsaved: (profile: ConnectionProfile) => void;
  }

  let { profile = null, onclose, onsaved }: Props = $props();

  const isNew = profile === null;
  let name = $state(profile?.name ?? 'Nouveau profil');
  let url = $state(profile?.url ?? 'ldap://127.0.0.1:389');
  let bindDn = $state(profile?.bind_dn ?? '');
  let baseDn = $state(profile?.base_dn ?? '');
  let tls = $state<TlsMode>(profile?.tls ?? 'none');
  let savePassword = $state(profile?.save_password ?? false);
  let password = $state('');
  let clearExistingPassword = $state(false);
  let error = $state<string | null>(null);
  let saving = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = null;
    try {
      const id = profile?.id ?? crypto.randomUUID();
      const next: ConnectionProfile = {
        id,
        name: name.trim(),
        url: url.trim(),
        bind_dn: bindDn.trim(),
        base_dn: baseDn.trim(),
        tls,
        timeout_secs: profile?.timeout_secs ?? 30,
        save_password: savePassword
      };
      const saved = await profileSave(next);

      if (savePassword && password.length > 0) {
        await profileSetPassword(id, password);
      } else if (clearExistingPassword || !savePassword) {
        if (profile?.has_saved_password) {
          await profileClearPassword(id);
        }
      }

      onsaved(saved);
    } catch (err) {
      error = formatError(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label={isNew ? 'Nouveau profil' : 'Modifier profil'}>
  <div class="dialog">
    <h2>{isNew ? 'Nouveau profil' : `Modifier « ${profile?.name} »`}</h2>

    <form onsubmit={submit}>
      <label>
        <span>Nom</span>
        <input type="text" bind:value={name} required />
      </label>

      <label>
        <span>URL</span>
        <input
          type="text"
          bind:value={url}
          required
          placeholder="ldap://host:389 ou ldaps://host:636"
          spellcheck="false"
        />
      </label>

      <label>
        <span>Bind DN (vide = anonyme)</span>
        <input type="text" bind:value={bindDn} spellcheck="false" />
      </label>

      <label>
        <span>Base DN</span>
        <input type="text" bind:value={baseDn} required spellcheck="false" />
      </label>

      <label>
        <span>TLS</span>
        <select bind:value={tls}>
          <option value="none">Aucun (déconseillé en prod)</option>
          <option value="start_tls">StartTLS</option>
          <option value="ldaps">LDAPS (ldaps://)</option>
        </select>
      </label>

      <fieldset>
        <legend>Mot de passe</legend>
        <label class="inline">
          <input type="checkbox" bind:checked={savePassword} />
          <span>Mémoriser dans le trousseau OS</span>
        </label>

        {#if savePassword}
          <label>
            <span>{profile?.has_saved_password ? 'Remplacer par' : 'Mot de passe'}</span>
            <input type="password" bind:value={password} autocomplete="new-password" />
          </label>
          {#if isNew || !profile?.has_saved_password}
            <p class="hint">Vide ⇒ demandera à chaque connexion.</p>
          {/if}
        {:else if profile?.has_saved_password}
          <label class="inline warn">
            <input type="checkbox" bind:checked={clearExistingPassword} />
            <span>Supprimer le mot de passe déjà stocké</span>
          </label>
        {/if}
      </fieldset>

      {#if error}
        <p class="status error">{error}</p>
      {/if}

      <div class="actions">
        <button type="submit" disabled={saving}>{saving ? 'Enregistrement…' : 'Enregistrer'}</button>
        <button type="button" class="secondary" onclick={onclose} disabled={saving}>Annuler</button>
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

  label.inline {
    flex-direction: row;
    align-items: center;
    gap: 0.4rem;
  }

  label > span {
    color: light-dark(#555, #aaa);
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

  fieldset {
    border: 1px solid light-dark(#ddd, #333);
    border-radius: 5px;
    padding: 0.5rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  legend {
    font-weight: 600;
    padding: 0 0.25rem;
    font-size: 0.85rem;
  }

  .hint {
    font-size: 0.8rem;
    color: light-dark(#777, #888);
    margin: 0;
  }

  .warn {
    color: #c0392b;
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
