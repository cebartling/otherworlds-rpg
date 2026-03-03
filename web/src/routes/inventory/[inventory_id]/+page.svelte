<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data, form }: {
    data: PageData;
    form: { action?: string; error?: string; success?: boolean } | null;
  } = $props();

  let confirmArchive = $state(false);

  function toggleConfirmArchive() {
    confirmArchive = !confirmArchive;
  }
</script>

<svelte:head>
  <title>Inventory {formatUuidDisplay(data.inventory.inventory_id)}</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <a
        href="/inventory"
        class="text-sm transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        &larr; All Inventories
      </a>
      <h1
        class="text-3xl font-bold tracking-tight mt-1"
        style="color: var(--color-accent);"
      >
        Inventory {formatUuidDisplay(data.inventory.inventory_id)}
      </h1>
    </div>
    <span class="text-xs" style="color: var(--color-text-muted);">
      Version {data.inventory.version}
    </span>
  </section>

  <!-- Add Item -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
      Add Item
    </h3>

    {#if form?.action === 'addItem' && form?.error}
      <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
    {/if}
    {#if form?.action === 'addItem' && form?.success}
      <p class="mb-3 text-sm" style="color: #81c784;">Item added.</p>
    {/if}

    <form method="POST" action="?/addItem" use:enhance class="flex items-end gap-4">
      <div class="flex-1">
        <label
          for="add-item-id"
          class="block text-xs font-medium mb-1"
          style="color: var(--color-text-muted);"
        >
          Item ID
        </label>
        <input
          id="add-item-id"
          type="text"
          name="item_id"
          required
          placeholder="UUID of the item"
          class="w-full px-3 py-2 rounded-md text-sm"
          style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
        />
      </div>
      <button
        type="submit"
        class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
        style="background-color: var(--color-accent); color: var(--color-surface);"
      >
        Add Item
      </button>
    </form>
  </section>

  <!-- Items List -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Items
    </h2>

    {#if form?.action === 'removeItem' && form?.error}
      <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
    {/if}
    {#if form?.action === 'removeItem' && form?.success}
      <p class="mb-3 text-sm" style="color: #81c784;">Item removed.</p>
    {/if}
    {#if form?.action === 'equipItem' && form?.error}
      <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
    {/if}
    {#if form?.action === 'equipItem' && form?.success}
      <p class="mb-3 text-sm" style="color: #81c784;">Item equipped.</p>
    {/if}

    {#if data.inventory.items.length === 0}
      <p class="text-sm" style="color: var(--color-text-muted);">
        No items in this inventory.
      </p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                #
              </th>
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                Item ID
              </th>
              <th class="text-right py-2 font-medium" style="color: var(--color-text-muted);">
                Actions
              </th>
            </tr>
          </thead>
          <tbody>
            {#each data.inventory.items as itemId, i (itemId)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text-muted);">
                  {i + 1}
                </td>
                <td class="py-2 pr-4 font-mono text-xs" style="color: var(--color-text);">
                  {itemId}
                </td>
                <td class="py-2 text-right">
                  <div class="flex items-center justify-end gap-2">
                    <form method="POST" action="?/equipItem" use:enhance>
                      <input type="hidden" name="item_id" value={itemId} />
                      <button
                        type="submit"
                        class="px-3 py-1 rounded text-xs font-medium transition-colors duration-150"
                        style="background-color: var(--color-accent); color: var(--color-surface);"
                      >
                        Equip
                      </button>
                    </form>
                    <form method="POST" action="?/removeItem" use:enhance>
                      <input type="hidden" name="item_id" value={itemId} />
                      <button
                        type="submit"
                        class="px-3 py-1 rounded text-xs font-medium transition-colors duration-150"
                        style="color: #e57373; border: 1px solid #e57373;"
                      >
                        Remove
                      </button>
                    </form>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <!-- Archive -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-base font-semibold" style="color: var(--color-text);">
          Archive Inventory
        </h3>
        <p class="text-sm mt-1" style="color: var(--color-text-muted);">
          Archiving removes this inventory from active use. This action cannot be easily undone.
        </p>
      </div>

      {#if confirmArchive}
        <div class="flex items-center gap-2">
          <form method="POST" action="?/archive" use:enhance>
            <button
              type="submit"
              class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
              style="background-color: #c62828; color: var(--color-text);"
            >
              Confirm Archive
            </button>
          </form>
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
            style="color: var(--color-text-muted); border: 1px solid var(--color-border);"
            onclick={toggleConfirmArchive}
          >
            Cancel
          </button>
        </div>
      {:else}
        <button
          type="button"
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="color: #e57373; border: 1px solid #e57373;"
          onclick={toggleConfirmArchive}
        >
          Archive
        </button>
      {/if}
    </div>
  </section>
</div>
