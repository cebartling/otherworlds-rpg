<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidShort, formatUuidDisplay } from '$lib/utils';

  let { data, form } = $props();

  let showBranchForm = $state(false);
  let showArchiveConfirm = $state(false);
  let branchCheckpointId = $state('');

  let successMessage = $derived(
    form?.success && form?.action === 'checkpoint'
      ? 'Checkpoint created successfully.'
      : null
  );
</script>

<svelte:head>
  <title>Session {formatUuidShort(data.run.run_id)} | Otherworlds</title>
</svelte:head>

<div class="space-y-8">
  <section>
    <a
      href="/sessions"
      class="text-sm transition-colors duration-150 hover:underline"
      style="color: var(--color-text-muted);"
    >
      &larr; Back to Sessions
    </a>

    <h1
      class="text-3xl font-bold tracking-tight mt-3 mb-2"
      style="color: var(--color-accent);"
    >
      Campaign Run
    </h1>
    <p class="font-mono text-sm" style="color: var(--color-text-muted);">
      {data.run.run_id}
    </p>
  </section>

  {#if successMessage}
    <div
      class="rounded-md px-4 py-3 text-sm"
      style="background-color: rgba(76, 175, 80, 0.15); border: 1px solid rgba(76, 175, 80, 0.3); color: #81c784;"
    >
      {successMessage}
    </div>
  {/if}

  {#if form?.error}
    <div
      class="rounded-md px-4 py-3 text-sm"
      style="background-color: rgba(229, 115, 115, 0.15); border: 1px solid rgba(229, 115, 115, 0.3); color: #e57373;"
    >
      {form.error}
    </div>
  {/if}

  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Run Details
    </h2>

    <dl class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
      <div>
        <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Run ID</dt>
        <dd class="font-mono" style="color: var(--color-text);">
          {formatUuidDisplay(data.run.run_id)}
        </dd>
      </div>
      <div>
        <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Campaign ID</dt>
        <dd class="font-mono" style="color: var(--color-text);">
          {data.run.campaign_id ? formatUuidDisplay(data.run.campaign_id) : 'None'}
        </dd>
      </div>
      <div>
        <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Version</dt>
        <dd style="color: var(--color-text);">
          {data.run.version}
        </dd>
      </div>
      <div>
        <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Checkpoints</dt>
        <dd style="color: var(--color-text);">
          {data.run.checkpoint_ids.length}
        </dd>
      </div>
    </dl>

    {#if data.run.checkpoint_ids.length > 0}
      <div class="mt-4 pt-4" style="border-top: 1px solid var(--color-border);">
        <h3 class="text-sm font-medium mb-2" style="color: var(--color-text-muted);">
          Checkpoint IDs
        </h3>
        <ul class="space-y-1">
          {#each data.run.checkpoint_ids as checkpointId (checkpointId)}
            <li class="text-xs font-mono" style="color: var(--color-text);">
              {checkpointId}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </section>

  <section class="space-y-4">
    <h2 class="text-lg font-semibold" style="color: var(--color-text);">
      Actions
    </h2>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <a
        href="/play/{data.run.run_id}"
        class="flex items-center justify-center gap-2 rounded-lg px-6 py-4 text-sm font-semibold transition-colors duration-150"
        style="background-color: var(--color-accent); color: var(--color-surface);"
      >
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653Z" />
        </svg>
        Enter Play
      </a>

      <form method="POST" action="?/checkpoint" use:enhance>
        <button
          type="submit"
          class="w-full rounded-lg px-6 py-4 text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border); color: var(--color-text);"
        >
          Create Checkpoint
        </button>
      </form>
    </div>

    <div
      class="rounded-lg p-5"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold" style="color: var(--color-text);">
          Branch Timeline
        </h3>
        <button
          type="button"
          class="text-xs px-3 py-1 rounded-md transition-colors duration-150"
          style="border: 1px solid var(--color-accent); color: var(--color-accent);"
          onclick={() => { showBranchForm = !showBranchForm; }}
        >
          {showBranchForm ? 'Cancel' : 'New Branch'}
        </button>
      </div>
      <p class="text-xs mb-3" style="color: var(--color-text-muted);">
        Create a divergent timeline from a checkpoint. This spawns a new campaign run
        that branches from the selected point in history.
      </p>

      {#if showBranchForm}
        <form method="POST" action="?/branch" use:enhance class="flex flex-col sm:flex-row gap-3">
          <label class="flex-1">
            <span class="sr-only">Checkpoint ID</span>
            <input
              type="text"
              name="from_checkpoint_id"
              bind:value={branchCheckpointId}
              placeholder="Enter checkpoint ID..."
              required
              class="w-full rounded-md px-4 py-2 text-sm transition-colors duration-150"
              style="background-color: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);"
            />
          </label>
          <button
            type="submit"
            class="px-6 py-2 rounded-md text-sm font-medium transition-colors duration-150"
            style="border: 1px solid var(--color-accent); color: var(--color-accent);"
          >
            Branch
          </button>
        </form>
      {/if}
    </div>

    <div
      class="rounded-lg p-5"
      style="border: 1px solid rgba(229, 115, 115, 0.3);"
    >
      {#if showArchiveConfirm}
        <p class="text-sm mb-3" style="color: #e57373;">
          Are you sure you want to archive this campaign run? This action cannot be undone.
        </p>
        <div class="flex gap-3">
          <form method="POST" action="?/archive" use:enhance>
            <button
              type="submit"
              class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
              style="background-color: rgba(229, 115, 115, 0.2); border: 1px solid rgba(229, 115, 115, 0.5); color: #e57373;"
            >
              Confirm Archive
            </button>
          </form>
          <button
            type="button"
            class="px-4 py-2 rounded-md text-sm transition-colors duration-150"
            style="color: var(--color-text-muted);"
            onclick={() => { showArchiveConfirm = false; }}
          >
            Cancel
          </button>
        </div>
      {:else}
        <button
          type="button"
          class="text-sm font-medium transition-colors duration-150"
          style="color: #e57373;"
          onclick={() => { showArchiveConfirm = true; }}
        >
          Archive Campaign Run
        </button>
      {/if}
    </div>
  </section>
</div>
