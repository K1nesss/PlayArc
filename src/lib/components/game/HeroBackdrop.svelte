<!--
	Hero background from game artwork (DOIT.md §3, §5): blurred art + a subtle
	tint of the artwork's dominant color + a very faint gradient.
	Scoped to the hero only — the app theme never changes per game.

	intensity: 'subtle' (Home) | 'rich' (Game Detail, the one place colors may be
	used most liberally).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { assetSrc } from '$lib/media';

	interface Props {
		artPath?: string | null;
		accent?: string | null;
		intensity?: 'subtle' | 'rich';
		class?: string;
		children: Snippet;
	}

	let { artPath, accent, intensity = 'subtle', class: className = '', children }: Props = $props();

	const src = $derived(assetSrc(artPath));
	let loaded = $state(false);
	$effect(() => {
		void src;
		loaded = false;
	});
</script>

<div
	class={`hero hero--${intensity} ${className}`}
	class:has-accent={Boolean(accent)}
	style:--hero-accent={accent ?? 'transparent'}
>
	{#if src}
		<img
			class="hero-art"
			class:is-loaded={loaded}
			{src}
			alt=""
			aria-hidden="true"
			decoding="async"
			onload={() => (loaded = true)}
		/>
	{/if}
	<div class="hero-scrim" aria-hidden="true"></div>
	<div class="hero-content">{@render children()}</div>
</div>

<style>
	.hero {
		position: relative;
		overflow: hidden;
		isolation: isolate;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
	}
	.hero.has-accent {
		background: color-mix(in oklab, var(--hero-accent) 7%, var(--bg-surface));
	}
	.hero--rich.has-accent {
		background: color-mix(in oklab, var(--hero-accent) 14%, var(--bg-surface));
	}
	:global(.dark) .hero.has-accent {
		background: color-mix(in oklab, var(--hero-accent) 10%, var(--bg-surface));
	}
	:global(.dark) .hero--rich.has-accent {
		background: color-mix(in oklab, var(--hero-accent) 18%, var(--bg-surface));
	}

	.hero-art {
		position: absolute;
		inset: -48px;
		z-index: -2;
		width: calc(100% + 96px);
		height: calc(100% + 96px);
		object-fit: cover;
		filter: blur(40px) saturate(1.1);
		opacity: 0;
		transition: opacity var(--duration-slow) var(--ease-standard);
	}
	.hero-art.is-loaded {
		opacity: 0.22;
	}
	.hero--rich .hero-art.is-loaded {
		opacity: 0.38;
	}
	:global(.dark) .hero-art.is-loaded {
		opacity: 0.3;
	}
	:global(.dark) .hero--rich .hero-art.is-loaded {
		opacity: 0.45;
	}

	/* Keeps text on the left readable; fades the art toward the content side. */
	.hero-scrim {
		position: absolute;
		inset: 0;
		z-index: -1;
		background: linear-gradient(
			100deg,
			color-mix(in oklab, var(--bg-surface) 72%, transparent) 0%,
			color-mix(in oklab, var(--bg-surface) 30%, transparent) 55%,
			transparent 100%
		);
	}

	.hero-content {
		position: relative;
	}
</style>
