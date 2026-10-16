<!--
	Data Panel (DOIT.md §21, §22, §25) — the chart/data container.
	Owns the header (title 14/600, description 12), a fixed body height
	(sm 180 · md 260 · lg 320) and the loading / empty / error states, so every
	chart in the app looks and behaves the same.

	<DataPanel title="Playtime" description="Daily, last 30 days" state={loading ? 'loading' : 'ready'}>
		<EChart … />
	</DataPanel>
-->
<script lang="ts" module>
	export type PanelState = 'ready' | 'loading' | 'empty' | 'error';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { RotateCw } from '@lucide/svelte';
	import EmptyState from './EmptyState.svelte';
	import Skeleton from './Skeleton.svelte';
	import Button from './Button.svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		title?: string;
		description?: string;
		height?: 'sm' | 'md' | 'lg' | 'auto';
		state?: PanelState;
		emptyTitle?: string;
		emptyDescription?: string;
		errorDescription?: string;
		onRetry?: () => void;
		/** Shape of the loading skeleton. */
		skeleton?: 'bars' | 'rows' | 'block';
		actions?: Snippet;
		children?: Snippet;
		class?: string;
	}

	let {
		title,
		description,
		height = 'md',
		state = 'ready',
		emptyTitle,
		emptyDescription,
		errorDescription,
		onRetry,
		skeleton = 'block',
		actions,
		children,
		class: className = ''
	}: Props = $props();

	const HEIGHTS = { sm: 180, md: 260, lg: 320 };
	const bodyStyle = $derived(height === 'auto' ? undefined : `${HEIGHTS[height]}px`);
	const BAR_HEIGHTS = [38, 62, 45, 80, 52, 70, 34, 58, 66, 42, 74, 50];
</script>

<section class={`ui-panel ${className}`} aria-busy={state === 'loading' || undefined}>
	{#if title || actions}
		<header class="ui-panel-header">
			<div class="min-w-0">
				{#if title}<h3 class="ui-panel-title">{title}</h3>{/if}
				{#if description}<p class="ui-panel-description">{description}</p>{/if}
			</div>
			{#if actions}<div class="ui-panel-actions">{@render actions()}</div>{/if}
		</header>
	{/if}

	<div
		class={`ui-panel-body ${height === 'auto' && state !== 'ready' ? 'ui-panel-body--placeholder' : ''}`}
		style:height={bodyStyle}
	>
		{#if state === 'loading'}
			{#if skeleton === 'bars'}
				<div class="flex h-full items-end gap-2 pt-4">
					{#each BAR_HEIGHTS as h, index (index)}
						<Skeleton height={`${h}%`} radius="sm" class="flex-1" />
					{/each}
				</div>
			{:else if skeleton === 'rows'}
				<div class="flex flex-col gap-3 pt-1">
					{#each [0, 1, 2, 3, 4] as index (index)}
						<div class="flex items-center gap-3">
							<Skeleton width="96px" height="12px" />
							<Skeleton height="12px" width={`${80 - index * 12}%`} />
						</div>
					{/each}
				</div>
			{:else}
				<Skeleton height="100%" radius="md" />
			{/if}
		{:else if state === 'empty'}
			<EmptyState
				compact
				title={emptyTitle ?? t('暂无数据', 'No data yet')}
				description={emptyDescription}
			/>
		{:else if state === 'error'}
			<EmptyState
				compact
				tone="error"
				title={t('加载失败', 'Couldn’t load this')}
				description={errorDescription}
			>
				{#snippet action()}
					{#if onRetry}
						<Button size="sm" icon={RotateCw} onclick={onRetry}>{t('重试', 'Retry')}</Button>
					{/if}
				{/snippet}
			</EmptyState>
		{:else}
			{@render children?.()}
		{/if}
	</div>
</section>

<style>
	.ui-panel {
		display: flex;
		flex-direction: column;
		min-width: 0;
		padding: var(--space-4) var(--space-5) var(--space-5);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
	}
	.ui-panel-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-3);
		margin-bottom: var(--space-3);
	}
	.ui-panel-title {
		font-size: var(--text-base);
		font-weight: 600;
		line-height: 20px;
		color: var(--text-primary);
	}
	.ui-panel-description {
		margin-top: 2px;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
	.ui-panel-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-shrink: 0;
	}
	.ui-panel-body {
		position: relative;
		min-height: 0;
	}
	.ui-panel-body--placeholder {
		min-height: 160px;
	}
</style>
