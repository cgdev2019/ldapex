/**
 * Clipboard wrapper that prefers Tauri's native plugin and falls back
 * to the deprecated `document.execCommand('copy')` trick for any
 * environment where the plugin is unavailable (typically the Vite
 * dev server opened in a regular browser).
 *
 * `navigator.clipboard.writeText` is intentionally NOT used: it's
 * gated by a permission prompt on WebKitGTK and rejects with
 * "The request is not allowed by the user agent or the platform in
 * the current context" inside the Tauri Linux webview.
 */

import { writeText as tauriWriteText } from '@tauri-apps/plugin-clipboard-manager';

export async function copyToClipboard(text: string): Promise<void> {
  try {
    await tauriWriteText(text);
    return;
  } catch {
    /* fall through */
  }

  // Last-resort fallback for plain-browser previews of the SPA.
  const ta = document.createElement('textarea');
  ta.value = text;
  ta.setAttribute('readonly', '');
  ta.style.position = 'fixed';
  ta.style.left = '-1000px';
  ta.style.opacity = '0';
  document.body.appendChild(ta);
  ta.select();
  try {
    document.execCommand('copy');
  } finally {
    document.body.removeChild(ta);
  }
}
