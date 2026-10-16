<!--
	Sidebar footer Steam status (DOIT.md §11). Deliberately quiet:
	  ● Steam data synced · 2 minutes ago
	  ○ Syncing…
	  ● Sync failed            (danger dot; message in tooltip)
	  ○ Steam not connected    (opens Settings)
	Clicking starts a sync (same as Ctrl+R).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { LoaderCircle } from '@lucide/svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { syncStatus } from '$lib/shell/sync-status.svelte';
	import { formatTimeAgo } from '$lib/format';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		collapsed?: boolean;
		onSync: () => void;
		onConnect: () => void;
	}

	let { collapsed = false, onSync, onConnect }: Props = $props();

	// Re-render "2 minutes ago" without re-reading anything.
	let tick = $state(0);
	onMount(() => {
		const timer = window.setInterval(() => (tick += 1), 30_000);
		return () => window.clearInterval(timer);
	});

	const view = $derived.by(() => {
		void tick;
		if (!syncStatus.configured) {
			return {
				tone: 'off',
				title: t('Steam 未连接', 'Steam not connected'),
				detail: t('前往设置', 'Set up in Settings')
			};
		}
		if (syncStatus.state === 'syncing') {
			return { tone: 'busy', title: t('正在同步…', 'Syncing…'), detail: '' };
		}
		if (syncStatus.state === 'error') {
			return {
				tone: 'error',
				title: t('同步失败', 'Sync failed'),
				detail: t('点击重试', 'Click to retry')
			};
		}
		if (syncStatus.lastSyncedAt) {
			return {
				tone: 'ok',
				title: t('Steam 数据已同步', 'Steam data synced'),
				detail: formatTimeAgo(syncStatus.lastSyncedAt)
			};
		}
		return {
			tone: 'off',
			title: t('尚未同步', 'Not synced yet'),
			detail: t('点击同步', 'Click to sync')
		};
	});

	const tooltip = $derived(
		[view.title, view.detail, syncStatus.state === 'error' ? syncStatus.error : '']
			.filter(Boolean)
			.join(' · ')
	);

	function handleClick() {
		if (!syncStatus.configured) onConnect();
		else if (syncStatus.state !== 'syncing') onSync();
	}
</script>

{#snippet indicator()}
	{#if view.tone === 'busy'}
		<Icon icon={LoaderCircle} size="status" class="ui-spin sync-spinner" />
	{:else}
		<span class={`sync-dot sync-dot--${view.tone}`} aria-hidden="true"></span>
	{/if}
{/snippet}

<Tooltip
	text={tooltip}
	shortcut={syncStatus.configured ? 'Ctrl+R' : undefined}
	placement={collapsed ? 'right' : 'top'}
	delay={300}
>
	<button
		type="button"
		class="sync-status"
		class:is-collapsed={collapsed}
		aria-label={tooltip}
		aria-busy={view.tone === 'busy' || undefined}
		onclick={handleClick}
	>
		<span class="sync-indicator">{@render indicator()}</span>
		{#if !collapsed}
			<span class="sync-text">
				<span class="sync-title">{view.title}</span>
				{#if view.detail}<span class="sync-detail">{view.detail}</span>{/if}
			</span>
		{/if}
	</button>
</Tooltip>

<style>
	.sync-status {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		width: 100%;
		padding: 6px 10px;
		border-radius: var(--radius-md);
		text-align: left;
		color: var(--text-tertiary);
		transition: background-color var(--duration-fast) var(--ease-standard);
	}
	.sync-status:hover {
		background: var(--state-hover);
	}
	.sync-status.is-collapsed {
		justify-content: center;
		width: 40px;
		padding: 10px 0;
	}
	.sync-indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		flex-shrink: 0;
	}
	.sync-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}
	.sync-dot--ok {
		background: var(--success);
	}
	.sync-dot--error {
		background: var(--danger);
	}
	.sync-dot--off {
		border: 1.5px solid var(--text-tertiary);
	}
	.sync-status :global(.sync-spinner) {
		color: var(--text-tertiary);
	}
	.sync-text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		line-height: 16px;
	}
	.sync-title {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
	.sync-detail {
		font-size: 11px;
		color: var(--text-tertiary);
	}
</style>
