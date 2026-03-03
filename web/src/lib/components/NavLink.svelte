<script lang="ts">
  import { page } from '$app/stores';
  import type { Snippet } from 'svelte';

  let { href, children }: { href: string; children: Snippet } = $props();

  let isActive = $derived(
    href === '/'
      ? $page.url.pathname === '/'
      : $page.url.pathname.startsWith(href)
  );
</script>

<a
  {href}
  class="nav-link px-3 py-2 rounded-md text-sm font-medium transition-colors duration-150"
  class:active={isActive}
  aria-current={isActive ? 'page' : undefined}
>
  {@render children()}
</a>

<style>
  .nav-link {
    color: var(--color-text-muted);
  }

  .nav-link:hover {
    color: var(--color-text);
    background-color: var(--color-surface-hover);
  }

  .nav-link.active {
    color: var(--color-accent);
    background-color: var(--color-surface-alt);
  }
</style>
