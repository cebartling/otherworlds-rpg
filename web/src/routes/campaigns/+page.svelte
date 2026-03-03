<script lang="ts">
  import { enhance } from '$app/forms';
  import { formatUuidShort } from '$lib/utils';
  import type { PageData } from './$types';

  let { data, form }: { data: PageData; form: { error?: string } | null } = $props();

  let showIngestForm = $state(false);

  function toggleIngestForm() {
    showIngestForm = !showIngestForm;
  }
</script>

<svelte:head>
  <title>Campaigns</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <h1
        class="text-3xl font-bold tracking-tight"
        style="color: var(--color-accent);"
      >
        Campaigns
      </h1>
      <p class="mt-1 text-sm" style="color: var(--color-text-muted);">
        Browse and author campaign content.
      </p>
    </div>

    <button
      type="button"
      class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
      style="background-color: var(--color-accent); color: var(--color-surface);"
      onclick={toggleIngestForm}
    >
      {showIngestForm ? 'Cancel' : 'Ingest Campaign'}
    </button>
  </section>

  {#if showIngestForm}
    <section
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
        Ingest Campaign
      </h2>

      {#if form?.error}
        <p class="mb-4 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}

      <form method="POST" action="?/ingest" use:enhance class="space-y-4">
        <div>
          <label
            for="campaign-source"
            class="block text-sm font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Campaign Source (Markdown)
          </label>
          <textarea
            id="campaign-source"
            name="source"
            required
            rows="10"
            placeholder="Paste your campaign markdown content here..."
            class="w-full px-3 py-2 rounded-md text-sm font-mono"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); resize: vertical;"
          ></textarea>
        </div>
        <button
          type="submit"
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Ingest
        </button>
      </form>
    </section>
  {/if}

  {#if data.campaigns.length === 0}
    <section class="text-center py-16">
      <p class="text-lg" style="color: var(--color-text-muted);">
        No campaigns yet. Ingest your first campaign to begin.
      </p>
    </section>
  {:else}
    <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each data.campaigns as campaign (campaign.campaign_id)}
        <a
          href="/campaigns/{campaign.campaign_id}"
          class="group block rounded-lg p-6 transition-colors duration-150"
          style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
        >
          <h2
            class="text-xl font-semibold font-mono mb-3 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
            style="color: var(--color-text);"
          >
            {formatUuidShort(campaign.campaign_id)}
          </h2>

          <div class="flex flex-wrap gap-2 mb-4">
            <span
              class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              style={campaign.ingested
                ? 'background-color: #2e7d32; color: #e8f5e9;'
                : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
            >
              Ingested
            </span>
            <span
              class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              style={campaign.validated
                ? 'background-color: #2e7d32; color: #e8f5e9;'
                : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
            >
              Validated
            </span>
            <span
              class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              style={campaign.compiled
                ? 'background-color: #2e7d32; color: #e8f5e9;'
                : 'background-color: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border);'}
            >
              Compiled
            </span>
          </div>

          <div class="text-right">
            <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
              Version
            </span>
            <p class="text-sm" style="color: var(--color-text-muted);">
              v{campaign.version}
            </p>
          </div>
        </a>
      {/each}
    </section>
  {/if}
</div>
