<!--
	A row of <Stat>s presented as one cohesive overview block (DOIT.md §7 "avoid four
	oversized independent cards"). Thin dividers between items; wraps on narrow widths.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Render inside a surface card (Analysis overview) or bare (Hero, Game detail). */
		boxed?: boolean;
		class?: string;
		children: Snippet;
	}

	let { boxed = false, class: className = '', children }: Props = $props();
</script>

<div class={`ui-stat-group ${boxed ? 'ui-stat-group--boxed' : ''} ${className}`}>
	{@render children()}
</div>

<style>
	.ui-stat-group {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
		row-gap: var(--space-4);
	}
	.ui-stat-group > :global(*) {
		padding-inline: var(--space-5);
		border-left: 1px solid var(--border-subtle);
	}
	.ui-stat-group > :global(*:first-child) {
		padding-left: 0;
		border-left: 0;
	}
	.ui-stat-group--boxed {
		padding: var(--space-4) var(--space-5);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
	}
</style>
