<!--
	Card (DOIT.md §21) — the only three card variants in the app:

	  surface      static information          (default)
	  interactive  clickable item               (renders <a> with href, <button> with onclick)
	  panel        chart / data container       (prefer DataPanel, which adds header + states)

	Hover on interactive cards only strengthens the border and tints the
	background — no lift, no shadow growth.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		variant?: 'surface' | 'interactive' | 'panel';
		padding?: 'none' | 'sm' | 'md' | 'lg';
		href?: string;
		onclick?: (event: MouseEvent) => void;
		ariaLabel?: string;
		class?: string;
		children?: Snippet;
	}

	let {
		variant = 'surface',
		padding = 'md',
		href,
		onclick,
		ariaLabel,
		class: className = '',
		children
	}: Props = $props();

	const classes = $derived(`ui-card ui-card--${variant} ui-card--pad-${padding} ${className}`);
</script>

{#if variant === 'interactive' && href}
	<!-- `href` must already be resolve()'d by the caller. -->
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
	<a {href} class={classes} aria-label={ariaLabel}>{@render children?.()}</a>
{:else if variant === 'interactive' && onclick}
	<button type="button" class={classes} {onclick} aria-label={ariaLabel}>
		{@render children?.()}
	</button>
{:else}
	<div class={classes}>{@render children?.()}</div>
{/if}

<style>
	.ui-card {
		display: block;
		position: relative;
		min-width: 0;
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
		color: inherit;
		text-align: left;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			border-color var(--duration-fast) var(--ease-standard);
	}

	.ui-card--pad-none {
		padding: 0;
	}
	.ui-card--pad-sm {
		padding: var(--space-3);
	}
	.ui-card--pad-md {
		padding: var(--space-4);
	}
	.ui-card--pad-lg {
		padding: var(--space-5);
	}

	.ui-card--interactive {
		cursor: pointer;
		width: 100%;
	}
	.ui-card--interactive:hover {
		border-color: var(--border-strong);
		background: color-mix(in oklab, var(--bg-surface) 97%, var(--text-primary));
	}
	.ui-card--interactive:active {
		background: color-mix(in oklab, var(--bg-surface) 94%, var(--text-primary));
	}
</style>
