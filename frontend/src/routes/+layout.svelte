<script lang="ts">
  import '../app.css';
  import { setupI18n } from '$lib/i18n';
  import { theme } from '$lib/theme.svelte';

  // Initialise the locale store as early as possible so every component
  // that uses `$_` already sees a loaded catalogue on first render.
  setupI18n();
  // Apply the persisted theme + density overrides before the first
  // paint so users who picked dark/light/compact don't get a brief
  // OS-default flash.
  theme.init();

  let { children } = $props();
</script>

{@render children()}
