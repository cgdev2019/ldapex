<script lang="ts">
  import '../app.css';
  import { setupI18n } from '$lib/i18n';

  // Initialise the locale store as early as possible so every component
  // that uses `$_` already sees a loaded catalogue on first render.
  setupI18n();

  let { children } = $props();
</script>

{@render children()}
