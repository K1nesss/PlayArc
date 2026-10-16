<!--
	Inline notice (DOIT.md §25 partial-data, §28 inline errors). Sits next to the
	thing it is about — e.g. under the Steam API key field when the key is invalid.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CircleAlert, CircleCheck, Info, TriangleAlert } from '@lucide/svelte';
	import Icon from './Icon.svelte';

	interface Props {
		tone?: 'info' | 'success' | 'warning' | 'danger';
		title?: string;
		children?: Snippet;
		action?: Snippet;
		class?: string;
	}

	let { tone = 'info', title, children, action, class: className = '' }: Props = $props();

	const ICONS = { info: Info, success: CircleCheck, warning: TriangleAlert, danger: CircleAlert };
</script>

<div
	class={`ui-notice ui-notice--${tone} ${className}`}
	role={tone === 'danger' ? 'alert' : 'status'}
>
	<span class="ui-notice-icon"><Icon icon={ICONS[tone]} size="button" /></span>
	<div class="min-w-0 flex-1">
		{#if title}<p class="ui-notice-title">{title}</p>{/if}
		{#if children}<div class="ui-notice-body">{@render children()}</div>{/if}
	</div>
	{#if action}<div class="shrink-0">{@render action()}</div>{/if}
</div>

<style>
	.ui-notice {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-subtle);
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.ui-notice-icon {
		display: inline-flex;
		margin-top: 1px;
	}
	.ui-notice--info .ui-notice-icon {
		color: var(--accent);
	}
	.ui-notice--success .ui-notice-icon {
		color: var(--success);
	}
	.ui-notice--warning .ui-notice-icon {
		color: var(--warning);
	}
	.ui-notice--danger {
		border-color: color-mix(in oklab, var(--danger) 30%, transparent);
		background: var(--danger-soft);
	}
	.ui-notice--danger .ui-notice-icon {
		color: var(--danger);
	}
	.ui-notice-title {
		font-weight: 600;
		color: var(--text-primary);
	}
</style>
