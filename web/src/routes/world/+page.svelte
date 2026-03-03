<script lang="ts">
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>World State</title>
</svelte:head>

<div class="space-y-8">
  <section>
    <h1
      class="text-3xl font-bold tracking-tight"
      style="color: var(--color-accent);"
    >
      World State
    </h1>
    <p class="mt-1 text-sm" style="color: var(--color-text-muted);">
      Inspect world snapshots and their facts, flags, and dispositions.
    </p>
  </section>

  {#if data.snapshots.length === 0}
    <section class="text-center py-16">
      <p class="text-lg" style="color: var(--color-text-muted);">
        No world snapshots yet.
      </p>
    </section>
  {:else}
    <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.snapshots as snapshot (snapshot.world_id)}
        <a
          href="/world/{snapshot.world_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <h2
            class="text-xl font-semibold mb-3 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
            style="color: var(--color-text);"
          >
            World {formatUuidDisplay(snapshot.world_id)}
          </h2>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Facts
              </span>
              <p class="text-lg font-bold" style="color: var(--color-accent);">
                {snapshot.fact_count}
              </p>
            </div>
            <div class="text-center">
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Flags
              </span>
              <p class="text-lg font-bold" style="color: var(--color-accent);">
                {snapshot.flag_count}
              </p>
            </div>
            <div class="text-right">
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Version
              </span>
              <p class="text-sm" style="color: var(--color-text-muted);">
                v{snapshot.version}
              </p>
            </div>
          </div>
        </a>
      {/each}
    </section>
  {/if}
</div>
