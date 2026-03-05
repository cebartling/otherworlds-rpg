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
  <title>Resolution {formatUuidDisplay(data.resolution.resolution_id)}</title>
</svelte:head>

<div class="space-y-8">
  <section class="flex items-center justify-between">
    <div>
      <a
        href="/rules"
        class="text-sm transition-colors duration-150"
        style="color: var(--color-text-muted);"
      >
        &larr; All Resolutions
      </a>
      <h1
        class="text-3xl font-bold tracking-tight mt-1"
        style="color: var(--color-accent);"
      >
        Resolution {formatUuidDisplay(data.resolution.resolution_id)}
      </h1>
    </div>
    <span class="text-xs" style="color: var(--color-text-muted);">
      Version {data.resolution.version}
    </span>
  </section>

  <!-- Phase -->
  <section
    class="rounded-lg p-6"
    style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
  >
    <div class="flex items-center justify-between">
      <div>
        <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
          Current Phase
        </span>
        <p class="text-2xl font-bold" style="color: var(--color-accent);">
          {data.resolution.phase}
        </p>
      </div>
    </div>
  </section>

  <!-- Intent -->
  {#if data.resolution.intent}
    <section
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
        Declared Intent
      </h2>

      <dl class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Action Type</dt>
          <dd style="color: var(--color-text);">{data.resolution.intent.action_type}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Skill</dt>
          <dd style="color: var(--color-text);">{data.resolution.intent.skill ?? 'None'}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Difficulty Class</dt>
          <dd class="font-bold" style="color: var(--color-accent);">{data.resolution.intent.difficulty_class}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Modifier</dt>
          <dd style="color: var(--color-text);">{data.resolution.intent.modifier >= 0 ? '+' : ''}{data.resolution.intent.modifier}</dd>
        </div>
        {#if data.resolution.intent.target_id}
          <div>
            <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Target</dt>
            <dd class="font-mono text-xs" style="color: var(--color-text);">{data.resolution.intent.target_id}</dd>
          </div>
        {/if}
      </dl>
    </section>
  {/if}

  <!-- Check Result -->
  {#if data.resolution.check_result}
    <section
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
        Check Result
      </h2>

      <dl class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-sm">
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Natural Roll</dt>
          <dd class="text-xl font-bold" style="color: var(--color-accent);">{data.resolution.check_result.natural_roll}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Modifier</dt>
          <dd class="text-xl" style="color: var(--color-text);">{data.resolution.check_result.modifier >= 0 ? '+' : ''}{data.resolution.check_result.modifier}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">Total</dt>
          <dd class="text-xl font-bold" style="color: var(--color-text);">{data.resolution.check_result.total}</dd>
        </div>
        <div>
          <dt class="font-medium mb-1" style="color: var(--color-text-muted);">DC</dt>
          <dd class="text-xl" style="color: var(--color-text);">{data.resolution.check_result.difficulty_class}</dd>
        </div>
      </dl>

      <div class="mt-4 pt-4" style="border-top: 1px solid var(--color-border);">
        <span class="text-xs uppercase tracking-wider" style="color: var(--color-text-muted);">
          Outcome
        </span>
        <p class="text-lg font-bold" style="color: var(--color-accent);">
          {data.resolution.check_result.outcome}
        </p>
      </div>
    </section>
  {/if}

  <!-- Effects -->
  {#if data.resolution.effects.length > 0}
    <section
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2 class="text-lg font-semibold mb-4" style="color: var(--color-text);">
        Produced Effects
      </h2>

      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr style="border-bottom: 1px solid var(--color-border);">
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                #
              </th>
              <th class="text-left py-2 pr-4 font-medium" style="color: var(--color-text-muted);">
                Effect Type
              </th>
              <th class="text-left py-2 font-medium" style="color: var(--color-text-muted);">
                Target
              </th>
            </tr>
          </thead>
          <tbody>
            {#each data.resolution.effects as effect, i (i)}
              <tr style="border-bottom: 1px solid var(--color-border);">
                <td class="py-2 pr-4" style="color: var(--color-text-muted);">
                  {i + 1}
                </td>
                <td class="py-2 pr-4" style="color: var(--color-text);">
                  {effect.effect_type}
                </td>
                <td class="py-2 font-mono text-xs" style="color: var(--color-text);">
                  {effect.target_id ?? 'N/A'}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}

  <!-- Actions -->
  <section class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Declare Intent -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Declare Intent
      </h3>

      {#if form?.action === 'declareIntent' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'declareIntent' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Intent declared.</p>
      {/if}

      <form method="POST" action="?/declareIntent" use:enhance class="space-y-3">
        <div>
          <label
            for="action-type"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Action Type
          </label>
          <input
            id="action-type"
            type="text"
            name="action_type"
            required
            placeholder="e.g. melee_attack"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div>
          <label
            for="skill"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Skill (optional)
          </label>
          <input
            id="skill"
            type="text"
            name="skill"
            placeholder="e.g. swordsmanship"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div>
          <label
            for="target-id"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Target ID (optional)
          </label>
          <input
            id="target-id"
            type="text"
            name="target_id"
            placeholder="UUID of the target"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label
              for="difficulty-class"
              class="block text-xs font-medium mb-1"
              style="color: var(--color-text-muted);"
            >
              DC
            </label>
            <input
              id="difficulty-class"
              type="number"
              name="difficulty_class"
              required
              value="10"
              class="w-full px-3 py-2 rounded-md text-sm"
              style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
            />
          </div>
          <div>
            <label
              for="modifier"
              class="block text-xs font-medium mb-1"
              style="color: var(--color-text-muted);"
            >
              Modifier
            </label>
            <input
              id="modifier"
              type="number"
              name="modifier"
              required
              value="0"
              class="w-full px-3 py-2 rounded-md text-sm"
              style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
            />
          </div>
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Declare Intent
        </button>
      </form>
    </div>

    <!-- Resolve Check -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Resolve Check
      </h3>

      {#if form?.action === 'resolveCheck' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'resolveCheck' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Check resolved.</p>
      {/if}

      <p class="text-sm mb-4" style="color: var(--color-text-muted);">
        Roll the d20 and resolve the check against the declared intent's difficulty class.
      </p>

      <form method="POST" action="?/resolveCheck" use:enhance>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Roll &amp; Resolve
        </button>
      </form>
    </div>

    <!-- Produce Effects -->
    <div
      class="rounded-lg p-6"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h3 class="text-base font-semibold mb-4" style="color: var(--color-text);">
        Produce Effects
      </h3>

      {#if form?.action === 'produceEffects' && form?.error}
        <p class="mb-3 text-sm" style="color: #e57373;">{form.error}</p>
      {/if}
      {#if form?.action === 'produceEffects' && form?.success}
        <p class="mb-3 text-sm" style="color: #81c784;">Effects produced.</p>
      {/if}

      <form method="POST" action="?/produceEffects" use:enhance class="space-y-3">
        <div>
          <label
            for="effect-type"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Effect Type
          </label>
          <input
            id="effect-type"
            type="text"
            name="effect_type"
            required
            placeholder="e.g. damage"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <div>
          <label
            for="effect-target-id"
            class="block text-xs font-medium mb-1"
            style="color: var(--color-text-muted);"
          >
            Target ID (optional)
          </label>
          <input
            id="effect-target-id"
            type="text"
            name="target_id"
            placeholder="UUID of the target"
            class="w-full px-3 py-2 rounded-md text-sm"
            style="background-color: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border);"
          />
        </div>
        <button
          type="submit"
          class="w-full px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150"
          style="background-color: var(--color-accent); color: var(--color-surface);"
        >
          Produce Effects
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
          Archive Resolution
        </h3>
        <p class="text-sm mt-1" style="color: var(--color-text-muted);">
          Archiving removes this resolution from active use. This action cannot be easily undone.
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
