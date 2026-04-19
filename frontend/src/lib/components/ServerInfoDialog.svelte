<script lang="ts">
  import { formatError, ldapSearch, type Entry } from '$lib/bridge';
  import Icon from './Icon.svelte';

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let entry = $state<Entry | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    error = null;
    try {
      const results = await ldapSearch({
        base_dn: '',
        scope: 'base',
        filter: '(objectClass=*)',
        attributes: [
          '*',
          '+',
          'vendorName',
          'vendorVersion',
          'supportedLDAPVersion',
          'supportedSASLMechanisms',
          'supportedControl',
          'supportedExtension',
          'supportedFeatures',
          'namingContexts',
          'subschemaSubentry',
          'altServer',
          'defaultNamingContext',
          'configurationNamingContext',
          'schemaNamingContext',
          'dnsHostName',
          'serverName',
          'dsServiceName',
          'currentTime',
          'highestCommittedUSN'
        ],
        size_limit: 1
      });
      entry = results[0] ?? null;
      if (!entry) error = 'Aucune rootDSE renvoyée par ce serveur.';
    } catch (err) {
      error = formatError(err);
    } finally {
      loading = false;
    }
  }

  function get(name: string): string[] {
    if (!entry) return [];
    const attr = entry.attributes.find((a) => a.name.toLowerCase() === name.toLowerCase());
    if (!attr) return [];
    return attr.values.filter((v) => v.kind === 'text').map((v) => (v as { kind: 'text'; data: string }).data);
  }

  const vendor = $derived(get('vendorName')[0] ?? '');
  const vendorVer = $derived(get('vendorVersion')[0] ?? '');
  const versions = $derived(get('supportedLDAPVersion'));
  const sasl = $derived(get('supportedSASLMechanisms'));
  const controls = $derived(get('supportedControl'));
  const extensions = $derived(get('supportedExtension'));
  const features = $derived(get('supportedFeatures'));
  const naming = $derived([...get('namingContexts'), ...get('defaultNamingContext')]);
  const subschema = $derived(get('subschemaSubentry')[0] ?? '');
  const altServer = $derived(get('altServer'));
  const hostName = $derived(get('dnsHostName')[0] ?? get('serverName')[0] ?? '');
  const currentTime = $derived(get('currentTime')[0] ?? '');

  // Named OID -> friendly label lookup for the controls grid.
  const KNOWN: Record<string, string> = {
    '1.2.840.113556.1.4.319': 'Paged Results (RFC 2696)',
    '1.2.840.113556.1.4.473': 'Server-Side Sort (RFC 2891)',
    '1.2.840.113556.1.4.474': 'Sort Response (RFC 2891)',
    '1.2.840.113556.1.4.528': 'Notification',
    '1.2.840.113556.1.4.805': 'Tree Delete',
    '1.2.840.113556.1.4.841': 'DirSync (AD)',
    '1.2.840.113556.1.4.1338': 'Verify Name (AD)',
    '1.2.840.113556.1.4.1339': 'Domain Scope (AD)',
    '1.2.840.113556.1.4.1340': 'Search Options (AD)',
    '1.2.840.113556.1.4.1413': 'Permissive Modify (AD)',
    '1.2.840.113556.1.4.1504': 'Attribute Scope Query (AD)',
    '1.2.840.113556.1.4.1852': 'Quota',
    '1.2.840.113556.1.4.1907': 'Synchronization State (AD)',
    '1.2.840.113556.1.4.2064': 'ShutdownNotification (AD)',
    '1.3.6.1.1.12': 'Assertion (RFC 4528)',
    '1.3.6.1.1.13.1': 'PreRead (RFC 4527)',
    '1.3.6.1.1.13.2': 'PostRead (RFC 4527)',
    '1.3.6.1.1.22': 'Don\u2019t Use Copy (RFC 6171)',
    '1.3.6.1.4.1.4203.1.9.1.1': 'Sync Request (RFC 4533)',
    '1.3.6.1.4.1.4203.1.9.1.2': 'Sync State (RFC 4533)',
    '1.3.6.1.4.1.4203.1.9.1.3': 'Sync Done (RFC 4533)',
    '1.3.6.1.4.1.4203.1.10.1': 'Subentries (RFC 3672)',
    '1.3.6.1.4.1.4203.1.10.2': 'ManageDsaIT (RFC 3296)',
    '1.3.6.1.4.1.42.2.27.8.5.1': 'PasswordPolicy (draft-behera)',
    '2.16.840.1.113730.3.4.2': 'ManageDsaIT',
    '2.16.840.1.113730.3.4.18': 'Proxied Authorization v2 (RFC 4370)',
    '2.16.840.1.113730.3.4.3': 'Persistent Search',
    '2.16.840.1.113730.3.4.4': 'Password Expired',
    '2.16.840.1.113730.3.4.5': 'Password Expiring',
    '2.16.840.1.113730.3.4.9': 'Virtual List View (VLV)',
    '2.16.840.1.113730.3.4.10': 'VLV Response'
  };

  function friendly(oid: string): string {
    return KNOWN[oid] ?? oid;
  }

  function close() {
    onclose();
  }
</script>

<div class="backdrop" role="dialog" aria-modal="true" aria-label="Info serveur" onclick={close} tabindex="-1">
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header>
      <h2>
        <Icon name="database" size={16} />
        <span>Info serveur</span>
      </h2>
      <div class="spacer"></div>
      <button type="button" class="ghost icon-only" onclick={close}>
        <Icon name="x" size={14} />
      </button>
    </header>

    {#if loading}
      <p class="status"><Icon name="refresh" size={13} /> Chargement de la rootDSE…</p>
    {:else if error}
      <p class="status error">{error}</p>
    {:else if entry}
      <div class="body">
        <section class="meta">
          {#if vendor}
            <dl>
              <dt>Vendor</dt>
              <dd>
                <strong>{vendor}</strong>
                {#if vendorVer}<code class="inline">{vendorVer}</code>{/if}
              </dd>
            </dl>
          {/if}
          {#if hostName}
            <dl>
              <dt>Host</dt>
              <dd><code>{hostName}</code></dd>
            </dl>
          {/if}
          {#if versions.length > 0}
            <dl>
              <dt>Version LDAP</dt>
              <dd>
                {#each versions as v (v)}
                  <span class="mini-pill">v{v}</span>
                {/each}
              </dd>
            </dl>
          {/if}
          {#if currentTime}
            <dl>
              <dt>Current time</dt>
              <dd><code>{currentTime}</code></dd>
            </dl>
          {/if}
        </section>

        {#if naming.length > 0}
          <section>
            <h4>Naming contexts ({naming.length})</h4>
            <ul class="pills">
              {#each naming as n (n)}
                <li class="pill mono">{n}</li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if sasl.length > 0}
          <section>
            <h4>SASL mechanisms ({sasl.length})</h4>
            <ul class="pills">
              {#each sasl as m (m)}
                <li class="pill accent">{m}</li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if controls.length > 0}
          <section>
            <h4>Contrôles supportés ({controls.length})</h4>
            <ul class="pills">
              {#each controls as oid (oid)}
                <li class="pill" title={oid}>{friendly(oid)}</li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if extensions.length > 0}
          <section>
            <h4>Extensions ({extensions.length})</h4>
            <ul class="pills">
              {#each extensions as oid (oid)}
                <li class="pill" title={oid}>{friendly(oid)}</li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if features.length > 0}
          <section>
            <h4>Features</h4>
            <ul class="pills">
              {#each features as oid (oid)}
                <li class="pill" title={oid}>{friendly(oid)}</li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if subschema}
          <dl>
            <dt>Subschema</dt>
            <dd><code>{subschema}</code></dd>
          </dl>
        {/if}
        {#if altServer.length > 0}
          <dl>
            <dt>altServer</dt>
            <dd>
              {#each altServer as s (s)}
                <code class="inline">{s}</code>
              {/each}
            </dd>
          </dl>
        {/if}
      </div>
    {/if}
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
    z-index: 210;
    padding: 1rem;
  }

  .dialog {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    width: min(92vw, 48rem);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: center;
    padding: 0.85rem 1.1rem;
    border-bottom: 1px solid var(--color-border);
  }

  header h2 {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
  }

  .spacer {
    flex: 1;
  }

  .body {
    overflow: auto;
    padding: 1rem 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: 0.4rem 1rem;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  dl {
    margin: 0;
    display: grid;
    grid-template-columns: 8rem 1fr;
    gap: 0.35rem 0.75rem;
    font-size: 0.8rem;
  }

  dt {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-subtle);
    align-self: center;
  }

  dd {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .inline {
    font-size: 0.72rem;
  }

  h4 {
    font-size: 0.64rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-subtle);
    font-weight: 600;
    margin-bottom: 0.4rem;
  }

  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .pill {
    padding: 0.12rem 0.55rem;
    border-radius: var(--radius-pill);
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.72rem;
    border: 1px solid var(--color-border);
  }

  .pill.mono {
    font-family: var(--font-mono);
    font-size: 0.7rem;
  }

  .pill.accent {
    background: var(--color-primary-soft);
    color: var(--color-primary);
    border-color: color-mix(in oklab, var(--color-primary) 30%, transparent);
  }

  .mini-pill {
    padding: 0.05rem 0.5rem;
    border-radius: var(--radius-pill);
    background: var(--color-primary-soft);
    color: var(--color-primary);
    font-size: 0.72rem;
    font-weight: 600;
  }

  .status {
    padding: 1.5rem;
    text-align: center;
    color: var(--color-text-muted);
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    justify-content: center;
  }

  .status.error {
    color: var(--color-danger);
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }
</style>
