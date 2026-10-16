<!-- A titled group of setting rows inside one quiet card. -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title?: string;
		description?: string;
		children: Snippet;
	}

	let { title, description, children }: Props = $props();
</script>

<section class="group">
	{#if title}
		<header class="group-header">
			<h3 class="group-title">{title}</h3>
			{#if description}<p class="group-description">{description}</p>{/if}
		</header>
	{/if}
	<div class="group-card">{@render children()}</div>
</section>

<style>
	.group + :global(.group) {
		margin-top: var(--space-6);
	}
	.group-header {
		margin-bottom: var(--space-2);
		padding: 0 var(--space-1);
	}
	.group-title {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--text-primary);
	}
	.group-description {
		margin-top: 2px;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
	.group-card {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
	}
	.group-card > :global(.row + .row) {
		border-top: 1px solid var(--border-subtle);
	}
</style>
