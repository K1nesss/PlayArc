<!--
	One setting: label + optional description on the left, control on the right.
	`error` renders inline under the row (§28: critical errors are shown next to
	the setting they concern, not only in a toast). `stacked` puts the control
	below the label (wide inputs).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CircleAlert } from '@lucide/svelte';
	import Icon from '$lib/components/ui/Icon.svelte';

	interface Props {
		label: string;
		description?: string;
		error?: string;
		stacked?: boolean;
		/** id of the control, so clicking the label focuses it. */
		for?: string;
		children?: Snippet;
		extra?: Snippet;
	}

	let { label, description, error, stacked = false, for: forId, children, extra }: Props = $props();
</script>

<div class="row" class:is-stacked={stacked}>
	<div class="row-main">
		<div class="row-text">
			{#if forId}
				<label class="row-label" for={forId}>{label}</label>
			{:else}
				<p class="row-label">{label}</p>
			{/if}
			{#if description}<p class="row-description">{description}</p>{/if}
		</div>
		{#if children}<div class="row-control">{@render children()}</div>{/if}
	</div>
	{#if extra}<div class="row-extra">{@render extra()}</div>{/if}
	{#if error}
		<p class="row-error" role="alert"><Icon icon={CircleAlert} size="status" />{error}</p>
	{/if}
</div>

<style>
	.row {
		padding: var(--space-3) var(--space-4);
	}
	.row-main {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		min-height: 32px;
	}
	.is-stacked .row-main {
		flex-direction: column;
		align-items: stretch;
		gap: var(--space-2);
	}
	.row-text {
		min-width: 0;
	}
	.row-label {
		display: block;
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-primary);
	}
	.row-description {
		margin-top: 2px;
		font-size: var(--text-xs);
		line-height: 16px;
		color: var(--text-secondary);
	}
	.row-control {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-shrink: 0;
	}
	.is-stacked .row-control {
		flex-shrink: 1;
	}
	.row-extra {
		margin-top: var(--space-3);
	}
	.row-error {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: var(--space-2);
		font-size: var(--text-xs);
		color: var(--danger);
	}
</style>
