<script lang="ts">
  import { PUBLIC_APP_NAME } from '$env/static/public';
  import { enhance } from '$app/forms';

  let { data } = $props();

  let submitting = $state(false);
  let metadataOpen = $state(false);

  let session = $derived(data.session);
  let hasScene = $derived(session.current_scene_id !== null);
  let hasChoices = $derived(session.active_choice_options.length > 0);
</script>

<svelte:head>
  <title>
    {hasScene ? `Scene: ${session.current_scene_id}` : 'Session'} | {PUBLIC_APP_NAME}
  </title>
</svelte:head>

<div class="max-w-3xl mx-auto space-y-8">
  <!-- Narrative text: the hero element -->
  {#if hasScene}
    <section class="narrative-hero py-8">
      <div
        class="narrative-text text-lg leading-relaxed"
        style="color: var(--color-text);"
      >
        <p class="scene-label text-xs uppercase tracking-widest mb-4" style="color: var(--color-text-muted);">
          Scene: {session.current_scene_id}
        </p>
        <!-- Narrative text would come from the scene content; scene_id is the reference -->
        <div class="narrative-content text-xl leading-9" style="color: var(--color-text);">
          <p class="italic" style="color: var(--color-text-muted);">
            The scene unfolds before you...
          </p>
        </div>
      </div>
    </section>
  {:else}
    <section class="text-center py-12">
      <p
        class="text-xl italic"
        style="color: var(--color-text-muted);"
      >
        No scene is active. Advance the beat or enter a scene to begin.
      </p>
    </section>
  {/if}

  <!-- NPC references -->
  {#if hasScene && session.scene_history.length > 0}
    <section>
      <h3
        class="text-xs uppercase tracking-widest mb-3"
        style="color: var(--color-text-muted);"
      >
        Scene History
      </h3>
      <div class="flex flex-wrap gap-2">
        {#each session.scene_history as sceneRef}
          <span
            class="npc-badge text-xs px-3 py-1 rounded-full"
            style="background-color: var(--color-surface-alt); color: var(--color-text-muted); border: 1px solid var(--color-border);"
          >
            {sceneRef}
          </span>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Choices -->
  {#if hasChoices}
    <section class="space-y-3">
      <h3
        class="text-xs uppercase tracking-widest mb-3"
        style="color: var(--color-text-muted);"
      >
        What do you do?
      </h3>
      <div class="space-y-3">
        {#each session.active_choice_options as choice, index}
          <form
            method="POST"
            action="?/selectChoice"
            use:enhance={() => {
              submitting = true;
              return async ({ update }) => {
                submitting = false;
                await update();
              };
            }}
          >
            <input type="hidden" name="choice_index" value={index} />
            <input type="hidden" name="scene_id" value={choice.target_scene_id} />
            <input type="hidden" name="narrative_text" value="" />
            <input type="hidden" name="choices" value="[]" />
            <button
              type="submit"
              disabled={submitting}
              class="choice-button w-full text-left px-6 py-4 rounded-lg transition-all duration-150"
              style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border); color: var(--color-text);"
            >
              <span class="choice-index text-sm font-mono mr-3" style="color: var(--color-accent);">
                {index + 1}.
              </span>
              <span class="choice-label text-base">
                {choice.label}
              </span>
              {#if choice.target_scene_id}
                <span
                  class="block text-xs mt-1 ml-7"
                  style="color: var(--color-text-muted);"
                >
                  leads to: {choice.target_scene_id}
                </span>
              {/if}
            </button>
          </form>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Actions: advance beat / enter scene -->
  <section class="flex flex-wrap gap-3 pt-4" style="border-top: 1px solid var(--color-border);">
    <form
      method="POST"
      action="?/advanceBeat"
      use:enhance={() => {
        submitting = true;
        return async ({ update }) => {
          submitting = false;
          await update();
        };
      }}
    >
      <button
        type="submit"
        disabled={submitting}
        class="action-button px-5 py-2.5 rounded-md text-sm font-medium transition-colors duration-150"
        style="background-color: var(--color-accent); color: var(--color-surface);"
      >
        {#if submitting}
          Advancing...
        {:else}
          Advance Beat
        {/if}
      </button>
    </form>
  </section>

  <!-- Session metadata (collapsible) -->
  <section class="pt-4" style="border-top: 1px solid var(--color-border);">
    <button
      type="button"
      class="flex items-center gap-2 text-xs uppercase tracking-widest cursor-pointer"
      style="color: var(--color-text-muted);"
      onclick={() => metadataOpen = !metadataOpen}
    >
      <svg
        class="w-3 h-3 transition-transform duration-150"
        class:rotate-90={metadataOpen}
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="2"
        stroke="currentColor"
      >
        <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
      </svg>
      Session Details
    </button>

    {#if metadataOpen}
      <div class="mt-3 space-y-2 text-xs font-mono" style="color: var(--color-text-muted);">
        <div class="flex gap-2">
          <span class="opacity-60">Session ID:</span>
          <span>{session.session_id}</span>
        </div>
        {#if session.current_beat_id}
          <div class="flex gap-2">
            <span class="opacity-60">Current Beat:</span>
            <span>{session.current_beat_id}</span>
          </div>
        {/if}
        {#if session.current_scene_id}
          <div class="flex gap-2">
            <span class="opacity-60">Current Scene:</span>
            <span>{session.current_scene_id}</span>
          </div>
        {/if}
        <div class="flex gap-2">
          <span class="opacity-60">Version:</span>
          <span>{session.version}</span>
        </div>
        <div class="flex gap-2">
          <span class="opacity-60">Choices recorded:</span>
          <span>{session.choice_ids.length}</span>
        </div>
        <div class="flex gap-2">
          <span class="opacity-60">Scenes visited:</span>
          <span>{session.scene_history.length}</span>
        </div>
      </div>
    {/if}
  </section>
</div>

<style>
  .narrative-hero {
    font-family: Georgia, 'Times New Roman', serif;
  }

  .narrative-content {
    font-family: Georgia, 'Times New Roman', serif;
    line-height: 1.8;
  }

  .choice-button:hover:not(:disabled) {
    background-color: var(--color-surface-hover) !important;
    border-color: var(--color-accent) !important;
  }

  .choice-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .action-button:hover:not(:disabled) {
    background-color: var(--color-accent-hover) !important;
  }

  .action-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
