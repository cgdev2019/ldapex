<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { bookmarks, recents } from '$lib/bookmarks.svelte';
  import Icon from './Icon.svelte';

  interface Props {
    selectedDn: string | null;
    onselect: (dn: string) => void;
  }

  let { selectedDn, onselect }: Props = $props();
</script>

<div class="wrap">
  {#if bookmarks.items.length === 0}
    <div class="empty">
      <Icon name="star" size={22} />
      <p>{$_('bookmark.empty')}</p>
    </div>
  {:else}
    <ul>
      {#each bookmarks.items as b (b.dn)}
        <li>
          <button
            type="button"
            class="row"
            class:selected={selectedDn === b.dn}
            onclick={() => onselect(b.dn)}
            title={b.dn}
          >
            <Icon name="star-filled" size={12} class="bm-star" />
            <span class="texts">
              <span class="label">{b.label}</span>
              <span class="dn">{b.dn}</span>
            </span>
          </button>
          <button
            type="button"
            class="ghost icon-only tiny"
            onclick={() => bookmarks.remove(b.dn)}
            aria-label={$_('bookmark.remove')}
            title={$_('bookmark.remove')}
          >
            <Icon name="x" size={12} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if recents.items.length > 0}
    <header class="sub-head">
      <Icon name="clock" size={11} />
      <span>{$_('bookmark.recent_head')}</span>
    </header>
    <ul>
      {#each recents.items.slice(0, 10) as r (r.dn)}
        <li>
          <button
            type="button"
            class="row recent"
            class:selected={selectedDn === r.dn}
            onclick={() => onselect(r.dn)}
            title={r.dn}
          >
            <Icon name="clock" size={11} class="dim" />
            <span class="texts">
              <span class="label">{r.label}</span>
              <span class="dn">{r.dn}</span>
            </span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .wrap {
    overflow: auto;
    padding: 0.5rem 0.4rem;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.15rem;
    border-radius: var(--radius-sm);
  }

  li:hover {
    background: var(--color-surface-hover);
  }

  .row {
    flex: 1;
    background: transparent;
    border: none;
    padding: 0.3rem 0.45rem;
    color: inherit;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 0.45rem;
    min-width: 0;
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .row:hover {
    background: transparent;
  }

  .row.selected {
    background: var(--color-primary-soft);
  }

  .row.selected .label {
    color: var(--color-primary);
    font-weight: 500;
  }

  :global(svg.bm-star) {
    color: var(--color-warning);
    flex-shrink: 0;
  }

  :global(svg.dim) {
    color: var(--color-text-subtle);
    flex-shrink: 0;
  }

  .texts {
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 0.02rem;
  }

  .label {
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dn {
    font-size: 0.7rem;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tiny {
    width: 1.5rem;
    height: 1.5rem;
    padding: 0.2rem;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  li:hover .tiny {
    opacity: 1;
  }

  .sub-head {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.6rem 0.45rem 0.3rem;
    border-top: 1px solid var(--color-border-subtle);
    margin-top: 0.25rem;
    color: var(--color-text-subtle);
    font-size: 0.65rem;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    font-weight: 600;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 2rem 1rem;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-align: center;
  }

  .empty :global(svg) {
    color: var(--color-warning);
  }
</style>
