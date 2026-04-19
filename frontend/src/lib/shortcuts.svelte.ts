/**
 * Global keyboard shortcuts. The caller registers callbacks once
 * (typically in the top-level route) and this module attaches the
 * listener on `window`.
 *
 * Platform mapping: the "modifier" means Ctrl on Linux/Windows and Cmd
 * on macOS. `event.ctrlKey || event.metaKey` covers both.
 */

export interface ShortcutCallbacks {
  onFocusSearch?: () => void;
  onNewEntry?: () => void;
  onRefresh?: () => void;
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return target.isContentEditable;
}

export function registerShortcuts(callbacks: ShortcutCallbacks): () => void {
  function handler(event: KeyboardEvent) {
    const mod = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();

    // F5 → refresh the tree. Works even when an input has focus, but we
    // let the browser handle its own reload if Shift is held.
    if (event.key === 'F5' && !event.shiftKey && callbacks.onRefresh) {
      event.preventDefault();
      callbacks.onRefresh();
      return;
    }

    // Ctrl/Cmd+F → switch to the search panel and focus the filter
    // field. Only trigger when the user is not already typing in a
    // field unrelated to search.
    if (mod && key === 'f' && callbacks.onFocusSearch && !isEditableTarget(event.target)) {
      event.preventDefault();
      callbacks.onFocusSearch();
      return;
    }

    // Ctrl/Cmd+N → open the create-entry dialog. Skip when editing a
    // text field so we don't hijack the user's typing.
    if (mod && key === 'n' && callbacks.onNewEntry && !isEditableTarget(event.target)) {
      event.preventDefault();
      callbacks.onNewEntry();
    }
  }

  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}
