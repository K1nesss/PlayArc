<!-- Small status label. 6px radius (§17), minimal, never a capsule. -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
		class?: string;
		children: Snippet;
	}

	let { tone = 'neutral', class: className = '', children }: Props = $props();
</script>

<span class={`ui-badge ui-badge--${tone} ${className}`}>{@render children()}</span>

<style>
	.ui-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		height: 20px;
		padding: 0 6px;
		border-radius: var(--radius-sm);
		font-size: var(--text-xs);
		font-weight: 500;
		line-height: 1;
		white-space: nowrap;
		background: var(--state-hover);
		color: var(--text-secondary);
	}
	.ui-badge--accent {
		background: var(--accent-soft);
		color: var(--accent);
	}
	.ui-badge--success {
		background: color-mix(in oklab, var(--success) 12%, transparent);
		color: var(--success);
	}
	.ui-badge--warning {
		background: color-mix(in oklab, var(--warning) 14%, transparent);
		color: var(--warning);
	}
	.ui-badge--danger {
		background: var(--danger-soft);
		color: var(--danger);
	}
</style>
