<script lang="ts">
  import { PUBLIC_APP_NAME } from '$env/static/public';
  import ImageCarousel from '$lib/components/ImageCarousel.svelte';

  const images = [
    { src: '/images/female-warrior-lich-battle.png', alt: 'A warrior clashing with a lich in battle' },
    { src: '/images/female-archer-lich-force-field.png', alt: 'An archer confronting a lich behind a force field' },
    { src: '/images/female-warrior-entering-lich-tomb.png', alt: 'A warrior entering a dark lich tomb' },
    { src: '/images/scorcer.png', alt: 'A sorcerer channeling arcane power' },
  ];
</script>

<svelte:head>
  <title>{PUBLIC_APP_NAME}</title>
</svelte:head>

<div class="space-y-10">
  <section class="text-center py-8">
    <h1
      class="text-4xl font-bold tracking-tight mb-4"
      style="color: var(--color-accent);"
    >
      Welcome to {PUBLIC_APP_NAME}
    </h1>
    <p
      class="text-lg max-w-2xl mx-auto"
      style="color: var(--color-text-muted);"
    >
      A deterministic, event-driven tabletop RPG engine. Manage your campaigns,
      build characters, and run sessions with full replay and narrative control.
    </p>
  </section>

  <section>
    <ImageCarousel {images} />
  </section>

  <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
    <a
      href="/campaigns"
      class="group block rounded-lg p-6 transition-colors duration-150"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2
        class="text-xl font-semibold mb-2 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
        style="color: var(--color-text);"
      >
        Campaigns
      </h2>
      <p style="color: var(--color-text-muted);">
        Create and manage your campaign worlds. Define settings, track story arcs,
        and organize your narrative threads.
      </p>
    </a>

    <a
      href="/characters"
      class="group block rounded-lg p-6 transition-colors duration-150"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2
        class="text-xl font-semibold mb-2 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
        style="color: var(--color-text);"
      >
        Characters
      </h2>
      <p style="color: var(--color-text-muted);">
        Build and maintain character sheets. Track abilities, inventory, and
        progression across sessions.
      </p>
    </a>

    <a
      href="/sessions"
      class="group block rounded-lg p-6 transition-colors duration-150 sm:col-span-2 lg:col-span-1"
      style="background-color: var(--color-surface-alt); border: 1px solid var(--color-border);"
    >
      <h2
        class="text-xl font-semibold mb-2 transition-colors duration-150 group-hover:text-[var(--color-accent)]"
        style="color: var(--color-text);"
      >
        Sessions
      </h2>
      <p style="color: var(--color-text-muted);">
        Review past sessions, replay events, and prepare for upcoming gameplay.
        Every action is recorded and replayable.
      </p>
    </a>
  </section>
</div>
