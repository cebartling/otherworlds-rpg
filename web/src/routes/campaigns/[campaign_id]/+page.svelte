<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidDisplay } from '$lib/utils';
  import type { PageData } from './$types';

  let { data, form }: {
    data: PageData;
    form: { action?: string; error?: string; success?: boolean } | null;
  } = $props();

  let confirmArchive = $state(false);

  let canValidate = $derived(data.campaign.ingested && !data.campaign.validated);
  let canCompile = $derived(data.campaign.validated && !data.campaign.compiled);

  function toggleConfirmArchive() {
    confirmArchive = !confirmArchive;
  }
</script>

<svelte:head>
  <title>Campaign {formatUuidDisplay(data.campaign.campaign_id)}</title>
</svelte:head>

<div class="space-y-8">
  <!-- Header -->
  <section class="flex items-center justify-between">
    <div>
      <a
        href="/campaigns"
        class="text-sm transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        &larr; All Campaigns
      </a>
      <h1
        class="text-3xl font-bold tracking-tight font-mono mt-1"
        style="color: var(--color-accent);"
      >
        {formatUuidDisplay(data.campaign.campaign_id)}
      </h1>
    </div>
    <span class="text-xs" style="color: var(--color-text-muted);">
      Version {data.campaign.version}
    </span>
  </section>

  <!-- Status Pipeline -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-6" style="color: var(--color-text);">
      Content Pipeline
    </h2>

    <div class="flex items-center justify-between">
      <!-- Step 1: Ingested -->
      <div class="flex flex-col items-center flex-1">
        <div
          class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold"
          style={data.campaign.ingested
            ? 'background-color: #2e7d32; color: #e8f5e9;'
            : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
        >
          1
        </div>
        <span
          class="mt-2 text-xs font-medium uppercase tracking-wider"
          style="color: {data.campaign.ingested ? '#81c784' : 'var(--color-text-muted)'};"
        >
          Ingested
        </span>
      </div>

      <!-- Connector -->
      <div
        class="flex-1 h-0.5 -mt-5"
        style="background-color: {data.campaign.validated ? '#2e7d32' : 'var(--color-border)'};"
      ></div>

      <!-- Step 2: Validated -->
      <div class="flex flex-col items-center flex-1">
        <div
          class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold"
          style={data.campaign.validated
            ? 'background-color: #2e7d32; color: #e8f5e9;'
            : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
        >
          2
        </div>
        <span
          class="mt-2 text-xs font-medium uppercase tracking-wider"
          style="color: {data.campaign.validated ? '#81c784' : 'var(--color-text-muted)'};"
        >
          Validated
        </span>
      </div>

      <!-- Connector -->
      <div
        class="flex-1 h-0.5 -mt-5"
        style="background-color: {data.campaign.compiled ? '#2e7d32' : 'var(--color-border)'};"
      ></div>

      <!-- Step 3: Compiled -->
      <div class="flex flex-col items-center flex-1">
        <div
          class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold"
          style={data.campaign.compiled
            ? 'background-color: #2e7d32; color: #e8f5e9;'
            : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
        >
          3
        </div>
        <span
          class="mt-2 text-xs font-medium uppercase tracking-wider"
          style="color: {data.campaign.compiled ? '#81c784' : 'var(--color-text-muted)'};"
        >
          Compiled
        </span>
      </div>
    </div>
  </section>

  <!-- Version Info -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
      Version Details
    </h2>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <div>
        <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
          Version Number
        </span>
        <p class="text-lg font-bold" style="color: var(--color-accent);">
          v{data.campaign.version}
        </p>
      </div>
      <div>
        <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
          Version Hash
        </span>
        <p class="text-sm font-mono" style="color: var(--color-text);">
          {data.campaign.version_hash ?? 'N/A'}
        </p>
      </div>
    </div>
  </section>

  <!-- Actions -->
  {#if form?.error}
    <div
      class="rounded-lg p-4"
      style="background-color: rgba(198, 40, 40, 0.15); border: 1px solid #c62828;"
    >
      <p class="text-sm" style="color: #e57373;">
        {form.error}
      </p>
    </div>
  {/if}

  {#if form?.success}
    <div
      class="rounded-lg p-4"
      style="background-color: rgba(46, 125, 50, 0.15); border: 1px solid #2e7d32;"
    >
      <p class="text-sm" style="color: #81c784;">
        {#if form.action === 'validate'}
          Campaign validated successfully.
        {:else if form.action === 'compile'}
          Campaign compiled successfully.
        {/if}
      </p>
    </div>
  {/if}

  <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Validate Action -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-2" style="color: var(--color-text);">
        Validate Campaign
      </h3>
      <p class="text-sm mb-4" style="color: var(--color-text-muted);">
        Check campaign content for structural correctness and rule compliance.
      </p>
      <form method="POST" action="?/validate" use:enhance>
        <button
          type="submit"
          disabled={!canValidate}
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style={canValidate
            ? 'background-color: var(--color-accent); color: var(--color-surface); cursor: pointer;'
            : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border); cursor: not-allowed;'}
        >
          Validate
        </button>
      </form>
    </div>

    <!-- Compile Action -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-2" style="color: var(--color-text);">
        Compile Campaign
      </h3>
      <p class="text-sm mb-4" style="color: var(--color-text-muted);">
        Compile the validated campaign into a playable format.
      </p>
      <form method="POST" action="?/compile" use:enhance>
        <button
          type="submit"
          disabled={!canCompile}
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style={canCompile
            ? 'background-color: var(--color-accent); color: var(--color-surface); cursor: pointer;'
            : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border); cursor: not-allowed;'}
        >
          Compile
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
          Archive Campaign
        </h3>
        <p class="text-sm mt-1" style="color: var(--color-text-muted);">
          Archiving removes this campaign from active use. This action cannot be easily undone.
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
