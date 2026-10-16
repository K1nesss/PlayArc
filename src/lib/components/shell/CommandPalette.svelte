<!--
	Global search (DOIT.md §30, Ctrl+K). Jump to a page, run an action, or open a
	game. ↑ ↓ move, Enter opens, Esc closes. Games load on first open.
-->
<script lang="ts">
	import {
		ChartColumn,
		Clock,
		Gamepad2,
		House,
		Moon,
		RefreshCw,
		Search,
		Settings,
		Sun,
		type LucideIcon
	} from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { getGamesLibrary, type LibraryGame } from '$lib/api';
	import Icon from '$lib/components/ui/Icon.svelte';
	import { portal } from '$lib/ui/floating';
	import { formatPlaytimeHeadline } from '$lib/format';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		open: boolean;
		darkMode: boolean;
		onClose: () => void;
		onSync: () => void;
		onToggleTheme: () => void;
	}

	let { open, darkMode, onClose, onSync, onToggleTheme }: Props = $props();

	type Entry = {
		id: string;
		group: string;
		label: string;
		hint?: string;
		icon: LucideIcon;
		run: () => void;
	};

	let query = $state('');
	let activeIndex = $state(0);
	let input = $state<HTMLInputElement | null>(null);
	let list = $state<HTMLElement | null>(null);
	let games = $state<LibraryGame[] | null>(null);
	let previouslyFocused: HTMLElement | null = null;

	function go(path: string) {
		onClose();
		// `path` is always built with resolve() below.
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		void goto(path);
	}

	const entries = $derived.by<Entry[]>(() => {
		const pages = t('页面', 'Pages');
		const actions = t('操作', 'Actions');
		const base: Entry[] = [
			{
				id: 'p-home',
				group: pages,
				label: t('首页', 'Home'),
				icon: House,
				run: () => go(resolve('/home'))
			},
			{
				id: 'p-games',
				group: pages,
				label: t('游戏库', 'Games'),
				icon: Gamepad2,
				run: () => go(resolve('/games'))
			},
			{
				id: 'p-timeline',
				group: pages,
				label: t('时间线', 'Timeline'),
				icon: Clock,
				run: () => go(resolve('/timeline'))
			},
			{
				id: 'p-analysis',
				group: pages,
				label: t('分析', 'Analysis'),
				icon: ChartColumn,
				run: () => go(resolve('/analytics'))
			},
			{
				id: 'p-settings',
				group: pages,
				label: t('设置', 'Settings'),
				hint: 'Ctrl+,',
				icon: Settings,
				run: () => go(resolve('/settings'))
			},
			{
				id: 'a-sync',
				group: actions,
				label: t('同步 Steam 数据', 'Sync Steam data'),
				hint: 'Ctrl+R',
				icon: RefreshCw,
				run: () => {
					onClose();
					onSync();
				}
			},
			{
				id: 'a-theme',
				group: actions,
				label: darkMode
					? t('切换到浅色主题', 'Switch to light theme')
					: t('切换到深色主题', 'Switch to dark theme'),
				icon: darkMode ? Sun : Moon,
				run: () => {
					onClose();
					onToggleTheme();
				}
			}
		];

		const q = query.trim().toLowerCase();
		const matches = (text: string) => text.toLowerCase().includes(q);
		const filtered = q ? base.filter((entry) => matches(entry.label)) : base;

		const gameGroup = t('游戏', 'Games');
		const gameEntries: Entry[] = (games ?? [])
			.filter((game) => !q || matches(game.name))
			.sort((a, b) => (b.last_played_at ?? 0) - (a.last_played_at ?? 0))
			.slice(0, q ? 8 : 5)
			.map((game) => ({
				id: `g-${game.id}`,
				group: gameGroup,
				label: game.name,
				hint: formatPlaytimeHeadline(game.total_playtime_seconds),
				icon: Gamepad2,
				run: () => go(resolve('/games/[id]', { id: String(game.id) }))
			}));

		return [...filtered, ...gameEntries];
	});

	$effect(() => {
		if (!open) return;
		previouslyFocused = document.activeElement as HTMLElement | null;
		query = '';
		activeIndex = 0;
		queueMicrotask(() => input?.focus());
		if (games === null) {
			getGamesLibrary({ sort: 'recent' })
				.then((library) => (games = library.items))
				.catch(() => (games = []));
		}
		return () => previouslyFocused?.focus?.();
	});

	$effect(() => {
		void query;
		activeIndex = 0;
	});

	$effect(() => {
		if (!open || !list) return;
		list
			.querySelector<HTMLElement>(`[data-index="${activeIndex}"]`)
			?.scrollIntoView({ block: 'nearest' });
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			activeIndex = Math.min(entries.length - 1, activeIndex + 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			activeIndex = Math.max(0, activeIndex - 1);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			entries[activeIndex]?.run();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			onClose();
		}
	}
</script>

{#if open}
	<div use:portal class="palette-root">
		<div class="palette-backdrop" aria-hidden="true" onclick={onClose}></div>
		<div class="palette" role="dialog" aria-modal="true" aria-label={t('全局搜索', 'Search')}>
			<div class="palette-search">
				<Icon icon={Search} size="button" class="text-fg-subtle" />
				<input
					bind:this={input}
					bind:value={query}
					placeholder={t('搜索页面、操作或游戏…', 'Search pages, actions or games…')}
					role="combobox"
					aria-expanded="true"
					aria-controls="palette-list"
					aria-activedescendant={entries[activeIndex]
						? `palette-${entries[activeIndex].id}`
						: undefined}
					onkeydown={handleKeydown}
				/>
				<kbd class="palette-kbd">Esc</kbd>
			</div>

			<div bind:this={list} id="palette-list" class="palette-list" role="listbox">
				{#if entries.length === 0}
					<p class="palette-empty">{t('没有匹配的结果', 'No results')}</p>
				{/if}
				{#each entries as entry, index (entry.id)}
					{#if index === 0 || entries[index - 1].group !== entry.group}
						<p class="palette-group">{entry.group}</p>
					{/if}
					<div
						id={`palette-${entry.id}`}
						data-index={index}
						role="option"
						tabindex="-1"
						aria-selected={index === activeIndex}
						class="palette-item"
						class:is-active={index === activeIndex}
						onpointermove={() => (activeIndex = index)}
						onclick={() => entry.run()}
						onkeydown={() => {}}
					>
						<Icon icon={entry.icon} size="button" class="palette-icon" />
						<span class="min-w-0 flex-1 truncate">{entry.label}</span>
						{#if entry.hint}<span class="palette-hint">{entry.hint}</span>{/if}
					</div>
				{/each}
			</div>

			<footer class="palette-footer">
				<span><kbd>↑</kbd><kbd>↓</kbd> {t('选择', 'Navigate')}</span>
				<span><kbd>Enter</kbd> {t('打开', 'Open')}</span>
			</footer>
		</div>
	</div>
{/if}

<style>
	.palette-root {
		position: fixed;
		inset: 0;
		z-index: var(--z-modal);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: min(18vh, 140px);
	}
	.palette-backdrop {
		position: absolute;
		inset: 0;
		background: var(--scrim-subtle);
		animation: palette-fade var(--duration-normal) var(--ease-standard);
	}
	.palette {
		position: relative;
		display: flex;
		flex-direction: column;
		width: min(560px, calc(100vw - 48px));
		max-height: min(440px, calc(100vh - 160px));
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-2xl);
		background: var(--bg-elevated);
		backdrop-filter: blur(24px) saturate(130%);
		box-shadow: var(--elevation-overlay);
		overflow: hidden;
		animation: palette-in var(--duration-normal) var(--ease-standard);
	}
	.palette-search {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		height: 48px;
		padding: 0 var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
	}
	.palette-search input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		font-size: var(--text-lg);
		color: var(--text-primary);
	}
	.palette-search input:focus-visible {
		outline: none;
	}
	.palette-search input::placeholder {
		color: var(--text-tertiary);
	}
	.palette-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-1) var(--space-2) var(--space-2);
	}
	.palette-group {
		padding: var(--space-3) var(--space-2) var(--space-1);
		font-size: 11px;
		font-weight: 500;
		color: var(--text-tertiary);
	}
	.palette-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		height: 36px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-md);
		font-size: var(--text-base);
		color: var(--text-primary);
		cursor: pointer;
	}
	.palette-item.is-active {
		background: var(--state-active);
	}
	.palette-item :global(.palette-icon) {
		color: var(--text-tertiary);
	}
	.palette-item.is-active :global(.palette-icon) {
		color: var(--accent);
	}
	.palette-hint {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
		font-variant-numeric: tabular-nums;
	}
	.palette-empty {
		padding: var(--space-6);
		text-align: center;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.palette-footer {
		display: flex;
		gap: var(--space-4);
		padding: var(--space-2) var(--space-4);
		border-top: 1px solid var(--border-subtle);
		font-size: 11px;
		color: var(--text-tertiary);
	}
	kbd {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		margin-right: 2px;
		padding: 0 4px;
		border: 1px solid var(--border-normal);
		border-radius: 4px;
		font-family: inherit;
		font-size: 11px;
		color: var(--text-secondary);
	}
	.palette-kbd {
		margin: 0;
	}
	@keyframes palette-fade {
		from {
			opacity: 0;
		}
	}
	@keyframes palette-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
	}
</style>
