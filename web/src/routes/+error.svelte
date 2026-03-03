<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="error-page flex flex-col items-center justify-center py-16 px-4 text-center">
  <p class="error-status text-6xl font-bold mb-4">{$page.status}</p>

  {#if $page.status === 404}
    <h1 class="error-title text-3xl font-semibold mb-4">Lost in the Otherworlds</h1>
    <p class="error-message text-lg mb-8">
      The path you seek does not exist in this realm. Perhaps it was consumed by the void,
      or it never was.
    </p>
  {:else if $page.status === 409}
    <h1 class="error-title text-3xl font-semibold mb-4">Timeline Conflict</h1>
    <p class="error-message text-lg mb-8">
      {$page.error?.message || 'Another force has altered this reality while you acted. Refresh and try again.'}
    </p>
  {:else if $page.status === 400}
    <h1 class="error-title text-3xl font-semibold mb-4">Invalid Incantation</h1>
    <p class="error-message text-lg mb-8">
      {$page.error?.message || 'Your request could not be understood. Check your input and try again.'}
    </p>
  {:else}
    <h1 class="error-title text-3xl font-semibold mb-4">The Void Beckons</h1>
    <p class="error-message text-lg mb-8">
      Something went wrong in the depths of the engine. The keepers have been alerted.
    </p>
  {/if}

  <a href="/" class="return-home inline-block px-6 py-3 rounded-md text-sm font-medium transition-colors duration-150">
    Return Home
  </a>
</div>

<style>
  .error-page {
    min-height: 60vh;
  }

  .error-status {
    color: var(--color-accent);
  }

  .error-title {
    color: var(--color-text);
  }

  .error-message {
    color: var(--color-text-muted);
    max-width: 32rem;
  }

  .return-home {
    background-color: var(--color-accent);
    color: var(--color-surface);
  }

  .return-home:hover {
    background-color: var(--color-accent-hover);
  }
</style>
