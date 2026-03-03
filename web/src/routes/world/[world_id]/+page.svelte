<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data, form }: {
    data: PageData;
    form: { action?: string; error?: string; success?: boolean } | null;
  } = $props();

  let confirmArchive = $state(false);

  let flags = $derived(Object.entries(data.snapshot.flags));

  function toggleConfirmArchive() {
    confirmArchive = !confirmArchive;
  }
</script>

<svelte:head>
  <title>World {formatUuidDisplay(data.snapshot.world_id)}</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <a
        href="/world"
        class="text-sm transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        &larr; All World Snapshots
      </a>
      <h1
        class="text-3xl font-bold tracking-tight mt-1"
        style="color: var(--color-accent);"
      >
        World {formatUuidDisplay(data.snapshot.world_id)}
      </h1>
    </div>
    <span class="text-xs" style="color: var(--color-text-muted);">
      Version {data.snapshot.version}
    </span>
  </section>

  <!-- Facts -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Facts
    </h2>

    {#if data.snapshot.facts.length === 0}
      <p class="text-sm" style="color: var(--color-text-muted);">
        No facts recorded yet.
      </p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                #
              </th>
              <th class="text-left py-2 font-medium" style="color: var(--color-text-muted);">
                Fact Key
              </th>
            </tr>
          </thead>
          <tbody>
            {#each data.snapshot.facts as fact, i (fact)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text-muted);">
                  {i + 1}
                </td>
                <td class="py-2" style="color: var(--color-text);">
                  {fact}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <!-- Flags -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Flags
    </h2>

    {#if flags.length === 0}
      <p class="text-sm" style="color: var(--color-text-muted);">
        No flags set yet.
      </p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                Flag
              </th>
              <th class="text-right py-2 font-medium" style="color: var(--color-text-muted);">
                Value
              </th>
            </tr>
          </thead>
          <tbody>
            {#each flags as [key, value] (key)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text);">
                  {key}
                </td>
                <td class="py-2 text-right">
                  {#if value}
                    <span
                      class="inline-block px-2 py-0.5 rounded text-xs font-medium"
                      style="background-color: #2e7d32; color: #c8e6c9;"
                    >
                      true
                    </span>
                  {:else}
                    <span
                      class="inline-block px-2 py-0.5 rounded text-xs font-medium"
                      style="background-color: #c62828; color: #ffcdd2;"
                    >
                      false
                    </span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <!-- NPC Dispositions -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      NPC Dispositions
    </h2>

    {#if data.snapshot.disposition_entity_ids.length === 0}
      <p class="text-sm" style="color: var(--color-text-muted);">
        No NPC dispositions tracked yet.
      </p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                #
              </th>
              <th class="text-left py-2 font-medium" style="color: var(--color-text-muted);">
                Entity ID
              </th>
            </tr>
          </thead>
          <tbody>
            {#each data.snapshot.disposition_entity_ids as entityId, i (entityId)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text-muted);">
                  {i + 1}
                </td>
                <td class="py-2 font-mono text-xs" style="color: var(--color-text);">
                  {entityId}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <!-- Actions -->
  <section class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Apply Effect -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Apply Effect
      </h3>

      {#if form?.action === 'applyEffect' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'applyEffect' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Effect applied.</p>
      {/if}

      <form method="POST" action="?/applyEffect" use:enhance class="space-y-3">
        <div>
          <label
            for="fact-key"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Fact Key
          </label>
          <input
            id="fact-key"
            type="text"
            name="fact_key"
            required
            placeholder="e.g. dragon_slain"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Apply Effect
        </button>
      </form>
    </div>

    <!-- Set Flag -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Set Flag
      </h3>

      {#if form?.action === 'setFlag' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'setFlag' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Flag updated.</p>
      {/if}

      <form method="POST" action="?/setFlag" use:enhance class="space-y-3">
        <div>
          <label
            for="flag-key"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Flag Key
          </label>
          <input
            id="flag-key"
            type="text"
            name="flag_key"
            required
            placeholder="e.g. quest_active"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div>
          <label
            for="flag-value"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Value
          </label>
          <select
            id="flag-value"
            name="value"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          >
            <option value="true">True</option>
            <option value="false">False</option>
          </select>
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Set Flag
        </button>
      </form>
    </div>

    <!-- Update Disposition -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Update Disposition
      </h3>

      {#if form?.action === 'updateDisposition' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'updateDisposition' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Disposition updated.</p>
      {/if}

      <form method="POST" action="?/updateDisposition" use:enhance class="space-y-3">
        <div>
          <label
            for="entity-id"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Entity ID
          </label>
          <input
            id="entity-id"
            type="text"
            name="entity_id"
            required
            placeholder="UUID of the entity"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Update Disposition
        </button>
      </form>
    </div>
  </section>

  <!-- Archive -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-base font-semibold" style="color: var(--color-text);">
          Archive World Snapshot
        </h3>
        <p class="text-sm mt-1" style="color: var(--color-text-muted);">
          Archiving removes this world snapshot from active use. This action cannot be easily undone.
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
