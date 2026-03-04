<script lang="ts">
  let { images }: { images: { src: string; alt: string }[] } = $props();

  let currentIndex = $state(0);

  function next() {
    currentIndex = (currentIndex + 1) % images.length;
  }

  function prev() {
    currentIndex = (currentIndex - 1 + images.length) % images.length;
  }

  function goTo(index: number) {
    currentIndex = index;
  }

  $effect(() => {
    const interval = setInterval(next, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="carousel" role="group" aria-roledescription="carousel" aria-label="Fantasy art gallery">
  <div class="carousel-viewport" aria-live="polite">
    {#each images as image, i}
      <img
        src={image.src}
        alt={image.alt}
        class="carousel-slide"
        class:active={i === currentIndex}
        aria-hidden={i !== currentIndex}
      />
    {/each}
  </div>

  <button class="carousel-btn carousel-btn-prev" onclick={prev} aria-label="Previous image">
    &#8249;
  </button>
  <button class="carousel-btn carousel-btn-next" onclick={next} aria-label="Next image">
    &#8250;
  </button>

  <div class="carousel-dots" role="tablist" aria-label="Slide navigation">
    {#each images as _, i}
      <button
        class="carousel-dot"
        class:active={i === currentIndex}
        onclick={() => goTo(i)}
        role="tab"
        aria-selected={i === currentIndex}
        aria-label="Go to slide {i + 1}"
      ></button>
    {/each}
  </div>
</div>

<style>
  .carousel {
    position: relative;
    max-width: 56rem;
    margin: 0 auto;
    border-radius: 0.5rem;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }

  .carousel-viewport {
    position: relative;
    aspect-ratio: 16 / 9;
    background-color: var(--color-surface-alt);
  }

  .carousel-slide {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 500ms ease-in-out;
  }

  .carousel-slide.active {
    opacity: 1;
  }

  .carousel-btn {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    background-color: var(--color-surface-alt);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 50%;
    width: 2.5rem;
    height: 2.5rem;
    font-size: 1.5rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 200ms ease;
  }

  .carousel-btn:hover {
    opacity: 1;
    background-color: var(--color-surface-hover, var(--color-surface-alt));
  }

  .carousel-btn-prev {
    left: 0.75rem;
  }

  .carousel-btn-next {
    right: 0.75rem;
  }

  .carousel-dots {
    position: absolute;
    bottom: 0.75rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 0.5rem;
  }

  .carousel-dot {
    width: 0.625rem;
    height: 0.625rem;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    background-color: var(--color-surface-alt);
    cursor: pointer;
    padding: 0;
    transition: background-color 200ms ease;
  }

  .carousel-dot.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
  }

  .carousel-dot:hover {
    background-color: var(--color-accent);
  }
</style>
