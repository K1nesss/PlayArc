<!--
	Empty / error / offline state (DOIT.md §25, §27). Never leave an area blank.

	<EmptyState title="No games yet" description="Connect or sync your Steam library.">
		{#snippet action()}<Button>Sync now</Button>{/snippet}
	</EmptyState>
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CircleAlert, Inbox, WifiOff, type LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';

	interface Props {
		title: string;
		description?: string;
		tone?: 'empty' | 'error' | 'offline';
		icon?: LucideIcon;
		/** Compact = inside a panel; default = page-level. */
		compact?: boolean;
		bordered?: boolean;
		action?: Snippet;
		class?: string;
	}

	let {
		title,
		description,
		tone = 'empty',
		icon,
		compact = false,
		bordered = false,
		action,
		class: className = ''
	}: Props = $props();

	const DEFAULT_ICONS: Record<NonNullable<Props['tone']>, LucideIcon> = {
		empty: Inbox,
		error: CircleAlert,
		offline: WifiOff
	};
</script>

<div
	class={`ui-empty ${compact ? 'ui-empty--compact' : ''} ${bordered ? 'ui-empty--bordered' : ''} ui-empty--${tone} ${className}`}
	role={tone === 'error' ? 'alert' : 'status'}
>
	<span class="ui-empty-icon">
		<Icon icon={icon ?? DEFAULT_ICONS[tone]} size={compact ? 24 : 'empty'} />
	</span>
	<p class="ui-empty-title">{title}</p>
	{#if description}<p class="ui-empty-description">{description}</p>{/if}
	{#if action}<div class="ui-empty-action">{@render action()}</div>{/if}
</div>

<style>
	.ui-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-1);
		padding: var(--space-12) var(--space-6);
		text-align: center;
	}
	.ui-empty--compact {
		height: 100%;
		padding: var(--space-6) var(--space-4);
	}
	.ui-empty--bordered {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
	}

	.ui-empty-icon {
		display: inline-flex;
		margin-bottom: var(--space-2);
		color: var(--text-tertiary);
	}
	.ui-empty--error .ui-empty-icon {
		color: var(--danger);
	}

	.ui-empty-title {
		font-size: var(--text-base);
		font-weight: 600;
		color: var(--text-primary);
	}
	.ui-empty-description {
		max-width: 360px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.ui-empty-action {
		margin-top: var(--space-3);
	}
</style>
