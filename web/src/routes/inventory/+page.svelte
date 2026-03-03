<script lang="ts">
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>Inventories</title>
</svelte:head>

<div class="space-y-8">
  <section>
    <h1
      class="text-3xl font-bold tracking-tight"
      style="color: var(--color-accent);"
    >
      Inventories
    </h1>
    <p class="mt-1 text-sm" style="color: var(--color-text-muted);">
      Inspect inventory contents and manage items.
    </p>
  </section>

  {#if data.inventories.length === 0}
    <section class="text-center py-16">
      <p class="text-lg" style="color: var(--color-text-muted);">
        No inventories yet.
      </p>
    </section>
  {:else}
    <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.inventories as inventory (inventory.inventory_id)}
        <a
          href="/inventory/{inventory.inventory_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <h2
            class="text-xl font-semibold mb-3 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
            style="color: var(--color-text);"
          >
            Inventory {formatUuidDisplay(inventory.inventory_id)}
          </h2>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Items
              </span>
              <p class="text-lg font-bold" style="color: var(--color-accent);">
                {inventory.item_count}
              </p>
            </div>
            <div class="text-right">
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Version
              </span>
              <p class="text-sm" style="color: var(--color-text-muted);">
                v{inventory.version}
              </p>
            </div>
          </div>
        </a>
      {/each}
    </section>
  {/if}
</div>
