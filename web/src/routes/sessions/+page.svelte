<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidShort } from '$lib/utils';

  let { data, form } = $props();

  let campaignId = $state('');
</script>

<svelte:head>
  <title>Sessions | Otherworlds</title>
</svelte:head>

<div class="space-y-8">
  <section>
    <h1
      class="text-3xl font-bold tracking-tight mb-2"
      style="color: var(--color-accent);"
    >
      Campaign Sessions
    </h1>
    <p style="color: var(--color-text-muted);">
      Manage your campaign runs. Start new sessions, review progress, and branch timelines.
    </p>
  </section>

  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Start Campaign Run
    </h2>

    {#if form?.error}
      <p class="mb-4 text-sm" style="color: #e57373;">
        {form.error}
      </p>
    {/if}

    <form method="POST" action="?/start" use:enhance class="flex flex-col sm:flex-row gap-3">
      <label class="flex-1">
        <span class="sr-only">Campaign ID</span>
        <input
          type="text"
          name="campaign_id"
          bind:value={campaignId}
          placeholder="Enter campaign ID..."
          required
          class="w-full rounded-md px-4 py-2 text-sm transition-colors duration-150"
          style="background-color: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);"
        />
      </label>
      <button
        type="submit"
        class="px-6 py-2 rounded-md text-sm font-medium transition-colors duration-150"
        style="background-color: var(--color-accent); color: var(--color-surface);"
      >
        Start Run
      </button>
    </form>
  </section>

  <section>
    {#if data.runs.length === 0}
      <div
        class="rounded-lg p-8 text-center"
        style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
      >
        <p class="text-lg mb-2" style="color: var(--color-text-muted);">
          No campaign runs yet.
        </p>
        <p class="text-sm" style="color: var(--color-text-muted);">
          Start a new campaign run above to begin your journey.
        </p>
      </div>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each data.runs as run (run.run_id)}
          <a
            href="/sessions/{run.run_id}"
            class="group block rounded-lg p-5 transition-colors duration-150"
            style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
          >
            <div class="flex items-center justify-between mb-3">
              <span
                class="text-sm font-mono font-semibold transition-colors duration-150 group-hover:text-[var(--color-accent)]"
                style="color: var(--color-text);"
              >
                {formatUuidShort(run.run_id)}
              </span>
              <span
                class="text-xs px-2 py-0.5 rounded-full"
                style="background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);"
              >
                v{run.version}
              </span>
            </div>

            <div class="space-y-1 text-sm" style="color: var(--color-text-muted);">
              <p>
                <span class="font-medium" style="color: var(--color-text);">Campaign:</span>
                {run.campaign_id ? formatUuidShort(run.campaign_id) : 'None'}
              </p>
              <p>
                <span class="font-medium" style="color: var(--color-text);">Checkpoints:</span>
                {run.checkpoint_count}
              </p>
            </div>
          </a>
        {/each}
      </div>
    {/if}
  </section>
</div>
