<script lang="ts">
  import { page } from '$app/stores';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let sessionId = $derived($page.params.session_id);
  let shortId = $derived(sessionId ? sessionId.slice(0, 8) : '');
</script>

<nav class="mb-6" aria-label="Breadcrumb">
  <ol class="flex items-center gap-2 text-sm">
    <li>
      <a
        href="/play"
        class="breadcrumb-link transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        Play
      </a>
    </li>
    <li style="color: var(--color-text-muted);" aria-hidden="true">
      /
    </li>
    <li>
      <span
        class="font-mono text-xs"
        style="color: var(--color-accent);"
        aria-current="page"
      >
        Session {shortId}...
      </span>
    </li>
  </ol>
</nav>

{@render children()}

<style>
  .breadcrumb-link:hover {
    color: var(--color-text);
  }
</style>
