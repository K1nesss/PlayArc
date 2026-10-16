<!--
	Game artwork with a quiet fallback (first letter on a neutral tile) when the
	image is missing or fails to load. Shape follows the asset:
	  portrait  2:3  Steam library capsule (Games grid, Home, Game Detail)
	  square    1:1  thumbnails in lists (Timeline 32×32, list view)
-->
<script lang="ts">
	import { assetSrc } from '$lib/media';

	interface Props {
		name: string;
		path?: string | null;
		shape?: 'portrait' | 'square';
		/** Width in px; height follows the shape. Omit to fill the container width. */
		size?: number;
		radius?: 'sm' | 'md' | 'lg';
		class?: string;
	}

	let {
		name,
		path,
		shape = 'portrait',
		size,
		radius = 'md',
		class: className = ''
	}: Props = $props();

	let failed = $state(false);
	const src = $derived(assetSrc(path));
	$effect(() => {
		void src;
		failed = false;
	});

	const initial = $derived(name.trim().charAt(0).toUpperCase() || '?');
	const fontSize = $derived(
		size ? Math.max(11, Math.round(size * (shape === 'square' ? 0.42 : 0.3))) : 28
	);
</script>

<span
	class={`cover cover--${shape} cover--r-${radius} ${className}`}
	style:width={size ? `${size}px` : undefined}
	aria-hidden="true"
>
	{#if src && !failed}
		<img {src} alt="" loading="lazy" decoding="async" onerror={() => (failed = true)} />
	{:else}
		<span class="cover-fallback" style:font-size={`${fontSize}px`}>{initial}</span>
	{/if}
</span>

<style>
	.cover {
		position: relative;
		display: block;
		flex-shrink: 0;
		overflow: hidden;
		background: var(--bg-subtle);
		box-shadow: inset 0 0 0 1px var(--border-subtle);
	}
	.cover--portrait {
		aspect-ratio: 2 / 3;
	}
	.cover--square {
		aspect-ratio: 1;
	}
	.cover--r-sm {
		border-radius: var(--radius-sm);
	}
	.cover--r-md {
		border-radius: var(--radius-md);
	}
	.cover--r-lg {
		border-radius: var(--radius-xl);
	}
	.cover img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.cover-fallback {
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		font-weight: 600;
		color: var(--text-tertiary);
	}
</style>
