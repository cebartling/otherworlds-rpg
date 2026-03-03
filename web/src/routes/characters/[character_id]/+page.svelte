<script lang="ts">
  import { enhance } from '$app/forms';
  import type { PageData } from './$types';

  let { data, form }: {
    data: PageData;
    form: { action?: string; error?: string; success?: boolean } | null;
  } = $props();

  let confirmArchive = $state(false);

  let attributes = $derived(Object.entries(data.character.attributes));

  function toggleConfirmArchive() {
    confirmArchive = !confirmArchive;
  }
</script>

<svelte:head>
  <title>{data.character.name ?? 'Character'}</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <a
        href="/characters"
        class="text-sm transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        &larr; All Characters
      </a>
      <h1
        class="text-3xl font-bold tracking-tight mt-1"
        style="color: var(--color-accent);"
      >
        {data.character.name ?? 'Unnamed Character'}
      </h1>
    </div>
    <span class="text-xs" style="color: var(--color-text-muted);">
      Version {data.character.version}
    </span>
  </section>

  <!-- Experience Points -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <div class="flex items-center justify-between">
      <div>
        <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
          Experience Points
        </span>
        <p class="text-2xl font-bold" style="color: var(--color-accent);">
          {data.character.experience} XP
        </p>
      </div>
    </div>
  </section>

  <!-- Attributes -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Attributes
    </h2>

    {#if attributes.length === 0}
      <p class="text-sm" style="color: var(--color-text-muted);">
        No attributes defined yet. Use the form below to add one.
      </p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                Attribute
              </th>
              <th class="text-right py-2 font-medium" style="color: var(--color-text-muted);">
                Value
              </th>
            </tr>
          </thead>
          <tbody>
            {#each attributes as [name, value] (name)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text);">
                  {name}
                </td>
                <td class="py-2 text-right font-bold" style="color: var(--color-accent);">
                  {value}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>

  <!-- Actions -->
  <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Modify Attribute -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Modify Attribute
      </h3>

      {#if form?.action === 'modifyAttribute' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'modifyAttribute' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Attribute updated.</p>
      {/if}

      <form method="POST" action="?/modifyAttribute" use:enhance class="space-y-3">
        <div>
          <label
            for="attr-name"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Attribute Name
          </label>
          <input
            id="attr-name"
            type="text"
            name="attribute"
            required
            placeholder="e.g. strength"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div>
          <label
            for="attr-value"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            New Value
          </label>
          <input
            id="attr-value"
            type="number"
            name="new_value"
            required
            placeholder="0"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Update Attribute
        </button>
      </form>
    </div>

    <!-- Award Experience -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Award Experience
      </h3>

      {#if form?.action === 'awardExperience' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'awardExperience' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Experience awarded.</p>
      {/if}

      <form method="POST" action="?/awardExperience" use:enhance class="space-y-3">
        <div>
          <label
            for="xp-amount"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Amount
          </label>
          <input
            id="xp-amount"
            type="number"
            name="amount"
            required
            min="1"
            placeholder="100"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Award XP
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
          Archive Character
        </h3>
        <p class="text-sm mt-1" style="color: var(--color-text-muted);">
          Archiving removes this character from active play. This action cannot be easily undone.
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
