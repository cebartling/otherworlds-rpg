<script lang="ts">
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>Rules Resolutions</title>
</svelte:head>

<div class="space-y-8">
  <section>
    <h1
      class="text-3xl font-bold tracking-tight"
      style="color: var(--color-accent);"
    >
      Rules Resolutions
    </h1>
    <p class="mt-1 text-sm" style="color: var(--color-text-muted);">
      View and manage action resolutions through the rules engine.
    </p>
  </section>

  {#if data.resolutions.length === 0}
    <section class="text-center py-16">
      <p class="text-lg" style="color: var(--color-text-muted);">
        No resolutions yet.
      </p>
    </section>
  {:else}
    <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.resolutions as resolution (resolution.resolution_id)}
        <a
          href="/rules/{resolution.resolution_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <h2
            class="text-xl font-semibold mb-3 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
            style="color: var(--color-text);"
          >
            Resolution {formatUuidDisplay(resolution.resolution_id)}
          </h2>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Phase
              </span>
              <p class="text-lg font-bold" style="color: var(--color-accent);">
                {resolution.phase}
              </p>
            </div>
            <div class="text-right">
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Version
              </span>
              <p class="text-sm" style="color: var(--color-text-muted);">
                v{resolution.version}
              </p>
            </div>
          </div>
        </a>
      {/each}
    </section>
  {/if}
</div>
