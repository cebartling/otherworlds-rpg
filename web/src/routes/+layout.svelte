<script lang="ts">
  import { PUBLIC_APP_NAME } from '$env/static/public';
  import favicon from '$lib/assets/favicon.svg';
  import NavLink from '$lib/components/NavLink.svelte';
  import '../app.css';

  let { children } = $props();

  let mobileMenuOpen = $state(false);

  function toggleMobileMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMobileMenu() {
    mobileMenuOpen = false;
  }
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen flex flex-col" style="background-color: var(--color-surface); color: var(--color-text);">
  <header style="background-color: var(--color-surface-alt); border-bottom: 1px solid var(--color-border);">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <div class="flex items-center gap-8">
          <a href="/" class="text-lg font-bold tracking-wide" style="color: var(--color-accent);">
            {PUBLIC_APP_NAME}
          </a>

          <nav class="hidden md:flex items-center gap-1" aria-label="Main navigation">
            <NavLink href="/campaigns">Campaigns</NavLink>
            <NavLink href="/characters">Characters</NavLink>
            <NavLink href="/sessions">Sessions</NavLink>
            <NavLink href="/play">Play</NavLink>
          </nav>
        </div>

        <button
          type="button"
          class="md:hidden inline-flex items-center justify-center p-2 rounded-md transition-colors duration-150"
          style="color: var(--color-text-muted);"
          aria-expanded={mobileMenuOpen}
          aria-controls="mobile-menu"
          aria-label="Toggle navigation menu"
          onclick={toggleMobileMenu}
        >
          {#if mobileMenuOpen}
            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {:else}
            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
            </svg>
          {/if}
        </button>
      </div>
    </div>

    {#if mobileMenuOpen}
      <nav
        id="mobile-menu"
        class="md:hidden px-4 pb-4 flex flex-col gap-1"
        aria-label="Mobile navigation"
      >
        <div onclick={closeMobileMenu} onkeydown={closeMobileMenu} role="presentation">
          <NavLink href="/campaigns">Campaigns</NavLink>
        </div>
        <div onclick={closeMobileMenu} onkeydown={closeMobileMenu} role="presentation">
          <NavLink href="/characters">Characters</NavLink>
        </div>
        <div onclick={closeMobileMenu} onkeydown={closeMobileMenu} role="presentation">
          <NavLink href="/sessions">Sessions</NavLink>
        </div>
        <div onclick={closeMobileMenu} onkeydown={closeMobileMenu} role="presentation">
          <NavLink href="/play">Play</NavLink>
        </div>
      </nav>
    {/if}
  </header>

  <main class="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
    {@render children()}
  </main>

  <footer
    class="py-4 text-center text-xs"
    style="color: var(--color-text-muted); border-top: 1px solid var(--color-border);"
  >
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      &copy; {new Date().getFullYear()} {PUBLIC_APP_NAME}. All rights reserved.
    </div>
  </footer>
</div>
