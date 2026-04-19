<script lang="ts">
  import '../app.css';
  import { setupI18n } from '$lib/i18n';
  import { theme } from '$lib/theme.svelte';

  // Initialise the locale store as early as possible so every component
  // that uses `$_` already sees a loaded catalogue on first render.
  setupI18n();
  // Apply the persisted theme override (if any) before the first paint
  // so users who picked dark/light don't get a brief OS-default flash.
  theme.init();

  let { children } = $props();
</script>

{@render children()}
