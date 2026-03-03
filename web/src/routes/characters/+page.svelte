<script lang="ts">
  import { enhance } from '$app/forms';
  import type { PageData } from './$types';

  let { data, form }: { data: PageData; form: { error?: string } | null } = $props();

  let showCreateForm = $state(false);

  function toggleCreateForm() {
    showCreateForm = !showCreateForm;
  }
</script>

<svelte:head>
  <title>Characters</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <h1
        class="text-3xl font-bold tracking-tight"
        style="color: var(--color-accent);"
      >
        Characters
      </h1>
      <p class="mt-1 text-sm" style="color: var(--color-text-muted);">
        Manage your character roster.
      </p>
    </div>

    <button
      type="button"
      class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
      style="background-color: var(--color-accent); color: var(--color-surface);"
      onclick={toggleCreateForm}
    >
      {showCreateForm ? 'Cancel' : 'Create Character'}
    </button>
  </section>

  {#if showCreateForm}
    <section
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
        New Character
      </h2>

      {#if form?.error}
        <p class="mb-4 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}

      <form method="POST" action="?/create" use:enhance class="flex items-end gap-4">
        <div class="flex-1">
          <label
            for="character-name"
            class="block text-sm font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Character Name
          </label>
          <input
            id="character-name"
            type="text"
            name="name"
            required
            placeholder="Enter a name..."
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Create
        </button>
      </form>
    </section>
  {/if}

  {#if data.characters.length === 0}
    <section class="text-center py-16">
      <p class="text-lg" style="color: var(--color-text-muted);">
        No characters yet. Create your first character to begin.
      </p>
    </section>
  {:else}
    <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.characters as character (character.character_id)}
        <a
          href="/characters/{character.character_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <h2
            class="text-xl font-semibold mb-3 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
            style="color: var(--color-text);"
          >
            {character.name ?? 'Unnamed Character'}
          </h2>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Experience
              </span>
              <p class="text-lg font-bold" style="color: var(--color-accent);">
                {character.experience} XP
              </p>
            </div>
            <div class="text-right">
              <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
                Version
              </span>
              <p class="text-sm" style="color: var(--color-text-muted);">
                v{character.version}
              </p>
            </div>
          </div>
        </a>
      {/each}
    </section>
  {/if}
</div>
