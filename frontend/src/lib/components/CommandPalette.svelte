<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { bookmarks, recents } from '$lib/bookmarks.svelte';
  import { session } from '$lib/session.svelte';
  import Icon from './Icon.svelte';
  import type { IconName } from './Icon.svelte';

  interface Props {
    onclose: () => void;
    onnavigate: (dn: string) => void;
    onaction: (action: AppAction) => void;
  }

  export type AppAction =
    | 'new-entry'
    | 'switch-search'
    | 'switch-tree'
    | 'switch-bookmarks'
    | 'disconnect'
    | 'refresh';

  let { onclose, onnavigate, onaction }: Props = $props();

  interface Item {
    key: string;
    section: string;
    label: string;
    sub?: string;
    icon: IconName;
    run: () => void;
  }

  let query = $state('');
  let highlighted = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    inputEl?.focus();
    bookmarks.reload();
    recents.reload();
  });

  const items = $derived.by<Item[]>(() => {
    const out: Item[] = [];

    // 1. Actions
    const actions: Array<{ key: AppAction; label: string; icon: IconName }> = [
      { key: 'new-entry', label: $_('nav.new_entry'), icon: 'plus' },
      { key: 'switch-tree', label: $_('tabs.tree'), icon: 'folder' },
      { key: 'switch-search', label: $_('tabs.search'), icon: 'search' },
      { key: 'switch-bookmarks', label: $_('tabs.bookmarks'), icon: 'star' },
      { key: 'refresh', label: 'Refresh tree', icon: 'refresh' },
      { key: 'disconnect', label: $_('nav.disconnect'), icon: 'log-out' }
    ];
    for (const a of actions) {
      out.push({
        key: `action:${a.key}`,
        section: $_('palette.section_actions'),
        label: a.label,
        icon: a.icon,
        run: () => onaction(a.key)
      });
    }

    // 2. Bookmarks
    for (const b of bookmarks.items) {
      out.push({
        key: `bm:${b.dn}`,
        section: $_('palette.section_bookmarks'),
        label: b.label,
        sub: b.dn,
        icon: 'star-filled',
        run: () => onnavigate(b.dn)
      });
    }

    // 3. Recent DNs (not duplicating bookmarks)
    const bookmarkedDns = new Set(bookmarks.items.map((b) => b.dn));
    for (const r of recents.items.slice(0, 15)) {
      if (bookmarkedDns.has(r.dn)) continue;
      out.push({
        key: `rec:${r.dn}`,
        section: $_('palette.section_recent'),
        label: r.label,
        sub: r.dn,
        icon: 'clock',
        run: () => onnavigate(r.dn)
      });
    }

    // 4. Saved searches — from the same localStorage bucket the
    //    SearchPanel uses. Recognised as "Saved searches" items.
    try {
      const key = `ldapex.search-history.${session.activeProfileId ?? 'adhoc'}`;
      const raw = localStorage.getItem(key);
      const parsed: unknown = raw ? JSON.parse(raw) : [];
      if (Array.isArray(parsed)) {
        for (const f of parsed as string[]) {
          if (typeof f !== 'string') continue;
          out.push({
            key: `search:${f}`,
            section: $_('palette.section_searches'),
            label: f,
            icon: 'search',
            run: () => onaction('switch-search')
          });
        }
      }
    } catch {
      /* ignore */
    }

    return out;
  });

  const filtered = $derived.by<Item[]>(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(
      (it) => it.label.toLowerCase().includes(q) || (it.sub?.toLowerCase().includes(q) ?? false)
    );
  });

  /** Group filtered items by their section, preserving insertion order. */
  const grouped = $derived.by<Array<{ section: string; items: Item[] }>>(() => {
    const groups: Array<{ section: string; items: Item[] }> = [];
    const byName = new Map<string, Item[]>();
    for (const it of filtered) {
      if (!byName.has(it.section)) {
        const arr: Item[] = [];
        byName.set(it.section, arr);
        groups.push({ section: it.section, items: arr });
      }
      byName.get(it.section)!.push(it);
    }
    return groups;
  });

  $effect(() => {
    // Clamp the highlight whenever the filtered list shrinks.
    if (highlighted >= filtered.length) highlighted = Math.max(0, filtered.length - 1);
  });

  function onKey(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose();
    } else if (event.key === 'ArrowDown') {
      event.preventDefault();
      highlighted = Math.min(highlighted + 1, filtered.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const pick = filtered[highlighted];
      if (pick) {
        pick.run();
        onclose();
      }
    }
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget) onclose();
  }
</script>

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Command palette"
  onclick={handleBackdrop}
  tabindex="-1"
>
  <div class="palette" onkeydown={onKey}>
    <header>
      <Icon name="command" size={14} />
      <input
        bind:this={inputEl}
        bind:value={query}
        placeholder={$_('palette.placeholder')}
        spellcheck="false"
        autocomplete="off"
        oninput={() => (highlighted = 0)}
      />
    </header>

    <div class="body">
      {#if filtered.length === 0}
        <p class="empty">{$_('palette.no_results')}</p>
      {:else}
        {#each grouped as group (group.section)}
          <div class="group">
            <h4>{group.section}</h4>
            <ul>
              {#each group.items as it (it.key)}
                {@const globalIdx = filtered.indexOf(it)}
                <li>
                  <button
                    type="button"
                    class="item"
                    class:highlighted={globalIdx === highlighted}
                    onmouseenter={() => (highlighted = globalIdx)}
                    onclick={() => {
                      it.run();
                      onclose();
                    }}
                  >
                    <Icon name={it.icon} size={13} class="it-ico" />
                    <span class="it-label">{it.label}</span>
                    {#if it.sub}
                      <span class="it-sub">{it.sub}</span>
                    {/if}
                  </button>
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      {/if}
    </div>

    <footer>
      <span>{$_('palette.hint')}</span>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: color-mix(in oklab, black 45%, transparent);
    backdrop-filter: blur(6px);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: min(12vh, 5rem) 1rem 1rem;
    z-index: 300;
  }

  .palette {
    width: min(92vw, 38rem);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-subtle);
  }

  header input {
    flex: 1;
    border: none;
    background: transparent;
    padding: 0;
    font-size: 0.95rem;
  }

  header input:focus {
    border: none;
    box-shadow: none;
    outline: none;
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 0.35rem 0.35rem 0.5rem;
  }

  .group {
    padding-top: 0.3rem;
  }

  .group h4 {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-text-subtle);
    margin: 0.35rem 0.75rem 0.2rem;
    font-weight: 600;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }

  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.65rem;
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-text);
    font-size: var(--text-sm);
    text-align: left;
    justify-content: flex-start;
  }

  .item.highlighted {
    background: var(--color-primary-soft);
  }

  .item.highlighted :global(svg.it-ico) {
    color: var(--color-primary);
  }

  :global(svg.it-ico) {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .it-label {
    flex-shrink: 0;
  }

  .it-sub {
    flex: 1;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  footer {
    padding: 0.4rem 0.9rem;
    border-top: 1px solid var(--color-border-subtle);
    color: var(--color-text-subtle);
    font-size: 0.7rem;
  }

  .empty {
    padding: 1.2rem;
    text-align: center;
    color: var(--color-text-subtle);
    font-style: italic;
    font-size: var(--text-sm);
  }
</style>
