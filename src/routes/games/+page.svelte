<!--
	Games (DOIT.md §4) — "What games do I own?". No statistics here.
	Grid (Steam-style 2:3 covers) or List (side-by-side comparison).
-->
<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Gamepad2, LayoutGrid, List, Plus, RefreshCw, Search } from '@lucide/svelte';
	import { getGamesLibrary, type LibraryGame, type LibrarySort } from '$lib/api';
	import { loadSettingsFromStorage } from '$lib/settings';
	import {
		formatNumber,
		formatPlaytimeHeadline,
		formatRelativeDay,
		formatTimeAgo
	} from '$lib/format';
	import { t } from '$lib/i18n.svelte';
	import { assetSrc } from '$lib/media';
	import { requestSteamSync, syncStatus } from '$lib/shell/sync-status.svelte';
	import {
		Badge,
		Button,
		EmptyState,
		GameCard,
		InlineNotice,
		Input,
		SegmentedControl,
		Select,
		Skeleton
	} from '$lib/components/ui';
	import Cover from '$lib/components/game/Cover.svelte';
	import AddGameDialog from '$lib/components/game/AddGameDialog.svelte';

	type View = 'grid' | 'list';
	type Filter = 'all' | 'installed';

	const VIEW_KEY = 'velaris_games_view_v1';
	const RESTORE_FLAG_KEY = 'games_restore_from_detail_v1';
	const RESTORE_STATE_KEY = 'velaris_games_state_v1';
	const SEARCH_DEBOUNCE_MS = 180;

	let items = $state<LibraryGame[]>([]);
	let total = $state(0);
	let loading = $state(true);
	let errorMessage = $state('');
	let search = $state('');
	let sort = $state<LibrarySort>('recent');
	let filter = $state<Filter>('all');
	let view = $state<View>('grid');
	let addOpen = $state(false);
	let scroller = $state<HTMLElement | null>(null);
	let requestId = 0;
	let pendingScrollTop: number | null = null;

	const sortOptions = $derived<{ value: LibrarySort; label: string }[]>([
		{ value: 'recent', label: t('最近游玩', 'Recently played') },
		{ value: 'playtime', label: t('游玩时长', 'Playtime') },
		{ value: 'sessions', label: t('会话数', 'Sessions') },
		{ value: 'name', label: t('名称', 'Name') }
	]);
	const filterOptions = $derived<{ value: Filter; label: string }[]>([
		{ value: 'all', label: t('全部游戏', 'All games') },
		{ value: 'installed', label: t('已安装', 'Installed') }
	]);
	const viewOptions = $derived([
		{ value: 'grid' as View, label: t('网格', 'Grid'), icon: LayoutGrid, iconOnly: true },
		{ value: 'list' as View, label: t('列表', 'List'), icon: List, iconOnly: true }
	]);

	const countLabel = $derived(
		t(`${formatNumber(total)} 款游戏`, `${formatNumber(total)} ${total === 1 ? 'game' : 'games'}`)
	);
	const filtered = $derived(search.trim() !== '' || filter !== 'all');

	async function load(showSkeleton = false) {
		const id = ++requestId;
		if (showSkeleton) loading = true;
		errorMessage = '';
		try {
			const library = await getGamesLibrary({
				search,
				installedOnly: filter === 'installed',
				sort
			});
			if (id !== requestId) return;
			items = library.items;
			total = library.total;
		} catch (error) {
			if (id !== requestId) return;
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (id === requestId) {
				loading = false;
				if (pendingScrollTop !== null) {
					const top = pendingScrollTop;
					pendingScrollTop = null;
					await tick();
					requestAnimationFrame(() => scroller?.scrollTo({ top }));
				}
			}
		}
	}

	let searchTimer: number | null = null;
	function onSearchInput() {
		if (searchTimer !== null) window.clearTimeout(searchTimer);
		searchTimer = window.setTimeout(() => void load(), SEARCH_DEBOUNCE_MS);
	}

	function setView(next: View) {
		try {
			localStorage.setItem(VIEW_KEY, next);
		} catch {
			// Per-device convenience only.
		}
	}

	/** Remember filters + scroll so Back from a game returns to the same spot. */
	function rememberState() {
		try {
			sessionStorage.setItem(RESTORE_FLAG_KEY, '1');
			sessionStorage.setItem(
				RESTORE_STATE_KEY,
				JSON.stringify({ search, sort, filter, scrollTop: scroller?.scrollTop ?? 0 })
			);
		} catch {
			// Ignore.
		}
	}

	function restoreState() {
		try {
			if (sessionStorage.getItem(RESTORE_FLAG_KEY) !== '1') return;
			sessionStorage.removeItem(RESTORE_FLAG_KEY);
			const saved = JSON.parse(sessionStorage.getItem(RESTORE_STATE_KEY) ?? 'null') as {
				search?: string;
				sort?: LibrarySort;
				filter?: Filter;
				scrollTop?: number;
			} | null;
			if (!saved) return;
			search = saved.search ?? '';
			sort = saved.sort ?? 'recent';
			filter = saved.filter ?? 'all';
			pendingScrollTop = saved.scrollTop ?? null;
		} catch {
			// Ignore.
		}
	}

	function openGame(id: number) {
		rememberState();
		void goto(resolve('/games/[id]', { id: String(id) }));
	}

	function sortBy(next: LibrarySort) {
		if (sort === next) return;
		sort = next;
		void load();
	}

	onMount(() => {
		restoreState();
		void (async () => {
			let stored: string | null = null;
			try {
				stored = localStorage.getItem(VIEW_KEY);
			} catch {
				stored = null;
			}
			if (stored === 'grid' || stored === 'list') {
				view = stored;
			} else {
				const settings = await loadSettingsFromStorage();
				view = settings.compactGameList ? 'list' : 'grid';
			}
		})();
		void load(true);
		return () => {
			if (searchTimer !== null) window.clearTimeout(searchTimer);
		};
	});
</script>

<div bind:this={scroller} class="app-page h-full overflow-y-auto">
	<div class="app-page-inner">
		<header class="games-header">
			<div class="title-row">
				<div>
					<h1 class="page-title">{t('游戏库', 'Games')}</h1>
					<p class="page-subtitle tabular-nums">
						{#if loading && total === 0}&nbsp;{:else}{countLabel}{/if}
					</p>
				</div>
				<Button icon={Plus} onclick={() => (addOpen = true)}>{t('添加游戏', 'Add game')}</Button>
			</div>

			<div class="toolbar-row">
				<Input
					bind:value={search}
					icon={Search}
					clearable
					placeholder={t('搜索游戏…', 'Search games…')}
					aria-label={t('搜索游戏', 'Search games')}
					class="search"
					oninput={onSearchInput}
					onclear={() => void load()}
				/>
				<Select
					bind:value={sort}
					options={sortOptions}
					ariaLabel={t('排序', 'Sort by')}
					variant="ghost"
					onchange={() => void load()}
				/>
				<Select
					bind:value={filter}
					options={filterOptions}
					ariaLabel={t('筛选', 'Filter')}
					variant="ghost"
					onchange={() => void load()}
				/>
				<div class="ml-auto">
					<SegmentedControl
						bind:value={view}
						options={viewOptions}
						ariaLabel={t('视图', 'View')}
						onchange={setView}
					/>
				</div>
			</div>
		</header>

		{#if errorMessage}
			<InlineNotice
				tone="danger"
				title={t('无法加载游戏库', 'Couldn’t load your library')}
				class="mb-4"
			>
				{errorMessage}
				{#snippet action()}
					<Button size="sm" icon={RefreshCw} onclick={() => load(true)}>{t('重试', 'Retry')}</Button
					>
				{/snippet}
			</InlineNotice>
		{/if}

		{#if loading}
			{#if view === 'grid'}
				<div class="grid-view" aria-busy="true">
					{#each Array.from({ length: 12 }, (_, i) => i) as i (i)}
						<div class="flex flex-col gap-2">
							<Skeleton height="auto" radius="md" class="aspect-2/3" />
							<Skeleton width="80%" height="12px" />
							<Skeleton width="50%" height="10px" />
						</div>
					{/each}
				</div>
			{:else}
				<div class="list-view" aria-busy="true">
					{#each Array.from({ length: 8 }, (_, i) => i) as i (i)}
						<div class="list-skeleton-row">
							<Skeleton width="32px" height="32px" radius="sm" />
							<Skeleton width={`${30 + ((i * 17) % 30)}%`} height="12px" />
						</div>
					{/each}
				</div>
			{/if}
		{:else if items.length === 0 && !errorMessage}
			{#if filtered}
				<EmptyState
					bordered
					icon={Search}
					title={t('没有匹配的游戏', 'No matching games')}
					description={t('试试其他关键词或清除筛选。', 'Try another search or clear the filter.')}
				>
					{#snippet action()}
						<Button
							size="sm"
							onclick={() => {
								search = '';
								filter = 'all';
								void load();
							}}
						>
							{t('清除筛选', 'Clear filters')}
						</Button>
					{/snippet}
				</EmptyState>
			{:else}
				<EmptyState
					bordered
					icon={Gamepad2}
					title={t('还没有游戏', 'No games yet')}
					description={syncStatus.configured
						? t(
								'同步你的 Steam 游戏库，或手动添加游戏。',
								'Sync your Steam library or add a game manually.'
							)
						: t(
								'在设置中连接 Steam，或手动添加游戏。',
								'Connect Steam in Settings, or add a game manually.'
							)}
				>
					{#snippet action()}
						<div class="flex gap-2">
							{#if syncStatus.configured}
								<Button
									size="sm"
									variant="primary"
									icon={RefreshCw}
									loading={syncStatus.state === 'syncing'}
									onclick={requestSteamSync}
								>
									{t('立即同步', 'Sync now')}
								</Button>
							{:else}
								<Button size="sm" variant="primary" href={resolve('/settings')}>
									{t('连接 Steam', 'Connect Steam')}
								</Button>
							{/if}
							<Button size="sm" icon={Plus} onclick={() => (addOpen = true)}>
								{t('添加游戏', 'Add game')}
							</Button>
						</div>
					{/snippet}
				</EmptyState>
			{/if}
		{:else if view === 'grid'}
			<div class="grid-view">
				{#each items as game (game.id)}
					<GameCard
						name={game.name}
						coverSrc={assetSrc(game.cover_path)}
						playtimeSeconds={game.total_playtime_seconds}
						lastPlayedAt={game.last_played_at}
						muted={!game.is_installed}
						onclick={() => openGame(game.id)}
					/>
				{/each}
			</div>
		{:else}
			<div class="list-view" role="table" aria-label={t('游戏列表', 'Games')}>
				<div class="list-head" role="row">
					{#snippet head(label: string, key: LibrarySort, align: 'start' | 'end')}
						<button
							type="button"
							role="columnheader"
							class={`head-cell ${align === 'end' ? 'justify-end' : ''}`}
							class:is-sorted={sort === key}
							aria-sort={sort === key ? 'descending' : 'none'}
							onclick={() => sortBy(key)}
						>
							{label}
						</button>
					{/snippet}
					{@render head(t('游戏', 'Game'), 'name', 'start')}
					{@render head(t('时长', 'Playtime'), 'playtime', 'end')}
					{@render head(t('最后游玩', 'Last played'), 'recent', 'end')}
					{@render head(t('会话', 'Sessions'), 'sessions', 'end')}
				</div>
				{#each items as game (game.id)}
					<button
						type="button"
						role="row"
						class="list-row"
						class:is-muted={!game.is_installed}
						onclick={() => openGame(game.id)}
					>
						<span class="cell-game" role="cell">
							<Cover name={game.name} path={game.cover_path} shape="square" size={32} radius="sm" />
							<span class="truncate">{game.name}</span>
							{#if !game.is_installed}<Badge>{t('未安装', 'Not installed')}</Badge>{/if}
						</span>
						<span class="cell-num" role="cell"
							>{formatPlaytimeHeadline(game.total_playtime_seconds)}</span
						>
						<span
							class="cell-muted"
							role="cell"
							title={game.last_played_at ? formatTimeAgo(game.last_played_at) : ''}
						>
							{game.last_played_at ? formatRelativeDay(game.last_played_at) : '—'}
						</span>
						<span class="cell-muted cell-num" role="cell">{game.session_count}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

<AddGameDialog
	open={addOpen}
	onClose={() => (addOpen = false)}
	onCreated={(id) => {
		addOpen = false;
		openGame(id);
	}}
/>

<style>
	.games-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		margin-bottom: var(--space-5);
	}
	.title-row {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-4);
	}
	.toolbar-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.toolbar-row :global(.search) {
		width: min(320px, 100%);
	}

	.grid-view {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(136px, 1fr));
		gap: var(--space-6) var(--space-4);
		padding-bottom: var(--space-8);
	}

	.list-view {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
		overflow: hidden;
		margin-bottom: var(--space-8);
	}
	.list-head,
	.list-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 96px 120px 80px;
		align-items: center;
		gap: var(--space-4);
		padding: 0 var(--space-4);
	}
	.list-head {
		height: 36px;
		border-bottom: 1px solid var(--border-subtle);
	}
	.head-cell {
		display: flex;
		align-items: center;
		height: 100%;
		font-size: var(--text-xs);
		font-weight: 500;
		color: var(--text-tertiary);
		cursor: pointer;
		transition: color var(--duration-fast) var(--ease-standard);
	}
	.head-cell:hover,
	.head-cell.is-sorted {
		color: var(--text-primary);
	}
	.list-row {
		width: 100%;
		height: 48px;
		text-align: left;
		color: inherit;
		transition: background-color var(--duration-fast) var(--ease-standard);
	}
	.list-row + .list-row {
		border-top: 1px solid var(--border-subtle);
	}
	.list-row:hover {
		background: var(--state-hover);
	}
	.list-row:active {
		background: var(--state-active);
	}
	.list-row:focus-visible {
		outline-offset: -2px;
	}
	.cell-game {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-width: 0;
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-primary);
	}
	.is-muted .cell-game :global(.cover) {
		opacity: 0.7;
	}
	.cell-num {
		text-align: right;
		font-size: var(--text-base);
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
	}
	.cell-muted {
		text-align: right;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.cell-muted.cell-num {
		color: var(--text-secondary);
	}
	.list-skeleton-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		height: 48px;
		padding: 0 var(--space-4);
	}
	.list-skeleton-row + .list-skeleton-row {
		border-top: 1px solid var(--border-subtle);
	}
</style>
