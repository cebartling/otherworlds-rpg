<script lang="ts">
  import { PUBLIC_APP_NAME } from '$env/static/public';

  let { data } = $props();
</script>

<svelte:head>
  <title>Play | {PUBLIC_APP_NAME}</title>
</svelte:head>

<div class="space-y-8">
  <section class="text-center py-6">
    <h1
      class="text-3xl font-bold tracking-tight mb-2"
      style="color: var(--color-accent);"
    >
      Narrative Sessions
    </h1>
    <p
      class="text-base max-w-xl mx-auto"
      style="color: var(--color-text-muted);"
    >
      Choose a session to continue your journey through the Otherworlds.
    </p>
  </section>

  {#if data.sessions.length === 0}
    <div
      class="text-center py-16 rounded-lg"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <p class="text-lg mb-2" style="color: var(--color-text-muted);">
        No active sessions found.
      </p>
      <p class="text-sm" style="color: var(--color-text-muted);">
        Sessions will appear here once they are created through the API.
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.sessions as session (session.session_id)}
        <a
          href="/play/{session.session_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <div class="flex items-start justify-between mb-3">
            <h2
              class="text-lg font-semibold transition-colors duration-150 group-hover:text-[var(--color-accent)]"
              style="color: var(--color-text);"
            >
              Session
            </h2>
            <span
              class="text-xs font-mono px-2 py-0.5 rounded"
              style="background-color: var(--color-surface); color: var(--color-text-muted);"
            >
              v{session.version}
            </span>
          </div>

          <p class="text-xs font-mono mb-3 truncate" style="color: var(--color-text-muted);">
            {session.session_id}
          </p>

          {#if session.current_scene_id}
            <div class="flex items-center gap-2 mb-2">
              <span class="text-xs" style="color: var(--color-text-muted);">Scene:</span>
              <span class="text-sm font-medium" style="color: var(--color-text);">
                {session.current_scene_id}
              </span>
            </div>
          {:else}
            <p class="text-sm italic" style="color: var(--color-text-muted);">
              No scene active
            </p>
          {/if}

          {#if session.current_beat_id}
            <div class="flex items-center gap-2">
              <span class="text-xs" style="color: var(--color-text-muted);">Beat:</span>
              <span class="text-xs font-mono truncate" style="color: var(--color-text-muted);">
                {session.current_beat_id}
              </span>
            </div>
          {/if}
        </a>
      {/each}
    </div>
  {/if}
</div>
