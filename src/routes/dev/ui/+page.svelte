<!--
	UI kit gallery — every component in every state, both languages via the app
	setting. Open /dev/ui in `pnpm tauri dev` (dev builds only).
-->
<script lang="ts">
	import {
		Download,
		Gamepad2,
		LayoutGrid,
		List,
		RefreshCw,
		Search,
		Settings,
		Trash2
	} from '@lucide/svelte';
	import {
		Badge,
		Button,
		Card,
		DataPanel,
		EmptyState,
		GameCard,
		IconButton,
		InlineNotice,
		Input,
		Modal,
		Popover,
		SegmentedControl,
		Select,
		Skeleton,
		Stat,
		StatGroup,
		toast
	} from '$lib/components/ui';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import DailyBarChart from '$lib/components/charts/DailyBarChart.svelte';
	import RankingChart from '$lib/components/charts/RankingChart.svelte';
	import ActivityHeatmap from '$lib/components/charts/ActivityHeatmap.svelte';
	import { formatPlaytimeParts } from '$lib/format';
	import { t } from '$lib/i18n.svelte';

	const now = Math.floor(Date.now() / 1000);
	const dayKey = (offset: number) => {
		const d = new Date(Date.now() - offset * 86_400_000);
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	};
	const seeded = (i: number) => Math.abs(Math.sin(i * 12.9898) * 43758.5453) % 1;
	const daily = Array.from({ length: 30 }, (_, i) => ({
		day: dayKey(29 - i),
		seconds: seeded(i) < 0.25 ? 0 : Math.round(seeded(i + 99) * 4 * 3600)
	}));
	const year = Array.from({ length: 365 }, (_, i) => ({
		day: dayKey(364 - i),
		seconds: seeded(i) < 0.45 ? 0 : Math.round(seeded(i + 7) * 5 * 3600),
		formatted: ''
	}));
	const ranking = [
		['Elden Ring', 116640],
		['Counter-Strike 2', 78120],
		['Cyberpunk 2077', 65880],
		['Hades II', 41400],
		['Baldur’s Gate 3', 30240],
		['Stardew Valley', 18000],
		['Balatro', 9000]
	].map(([name, seconds], i) => ({
		id: i,
		name: String(name),
		seconds: Number(seconds),
		sessions: 10 + i
	}));
	const games = ranking.slice(0, 5).map((g, i) => ({ ...g, last: now - i * 86_400 * 2 - 3600 }));

	let view = $state<'grid' | 'list'>('grid');
	let range = $state<number>(30);
	let game = $state('all');
	let query = $state('');
	let modalOpen = $state(false);
	let confirmOpen = $state(false);
	let popoverOpen = $state(false);
	let syncing = $state(false);
	let panelState = $state<'ready' | 'loading' | 'empty' | 'error'>('ready');

	const total = formatPlaytimeParts(513360);

	function fakeSync() {
		syncing = true;
		window.setTimeout(() => {
			syncing = false;
			toast.success({
				title: t('同步完成', 'Sync complete'),
				description: t('更新了 12 个游戏', '12 games updated')
			});
		}, 1200);
	}
</script>

<div class="app-page h-full overflow-y-auto">
	<div class="app-page-inner flex flex-col gap-8 pb-16">
		<header class="page-header mb-0!">
			<div>
				<h1 class="page-title">UI kit</h1>
				<p class="page-subtitle">DOIT.md §21–§29, §35 — every component and state.</p>
			</div>
		</header>

		<section class="gallery-section">
			<h2>Buttons</h2>
			<div class="row">
				<Button variant="primary">Primary</Button>
				<Button>Secondary</Button>
				<Button variant="ghost">Ghost</Button>
				<Button variant="danger" icon={Trash2}>{t('删除', 'Delete')}</Button>
				<Button icon={RefreshCw} loading={syncing} onclick={fakeSync}
					>{t('立即同步', 'Sync now')}</Button
				>
				<Button disabled>Disabled</Button>
			</div>
			<div class="row">
				<Button size="sm">Small 28</Button>
				<Button size="md">Medium 32</Button>
				<Button size="lg">Large 36</Button>
				<Button variant="primary" icon={Download} size="lg">{t('导出数据', 'Export data')}</Button>
			</div>
			<div class="row">
				<IconButton
					icon={RefreshCw}
					label={t('同步 Steam 数据', 'Sync Steam data')}
					shortcut="Ctrl+R"
				/>
				<IconButton
					icon={Settings}
					label={t('设置', 'Settings')}
					shortcut="Ctrl+,"
					variant="secondary"
				/>
				<IconButton icon={Search} label={t('搜索', 'Search')} active />
				<IconButton icon={RefreshCw} label="Loading" loading />
				<IconButton icon={Trash2} label="Disabled" disabled />
				<span class="hint">← hover or Tab to see tooltips</span>
			</div>
		</section>

		<section class="gallery-section">
			<h2>Inputs, selects, segmented</h2>
			<div class="row">
				<Input
					bind:value={query}
					icon={Search}
					clearable
					placeholder={t('搜索游戏…', 'Search games…')}
					class="w-64"
				/>
				<Input value="76561190000000" error={t('Steam ID 无效', 'Invalid Steam ID')} class="w-64" />
				<Input value="Disabled" disabled class="w-48" />
			</div>
			<div class="row">
				<Select
					bind:value={game}
					ariaLabel={t('游戏', 'Game')}
					options={[
						{ value: 'all', label: t('全部游戏', 'All games') },
						...ranking.map((g) => ({ value: String(g.id), label: g.name }))
					]}
					class="w-52"
				/>
				<Select
					bind:value={game}
					variant="ghost"
					ariaLabel="Month"
					options={[
						{ value: 'all', label: '2026年10月' },
						{ value: '0', label: '2026年9月' }
					]}
				/>
				<SegmentedControl
					bind:value={range}
					ariaLabel="Range"
					options={[
						{ value: 7, label: t('7 天', '7 days') },
						{ value: 30, label: t('30 天', '30 days') },
						{ value: 90, label: t('90 天', '90 days') },
						{ value: 365, label: t('1 年', '1 year') },
						{ value: 0, label: t('全部', 'All') }
					]}
				/>
				<SegmentedControl
					bind:value={view}
					ariaLabel="View"
					options={[
						{ value: 'grid', label: t('网格', 'Grid'), icon: LayoutGrid, iconOnly: true },
						{ value: 'list', label: t('列表', 'List'), icon: List, iconOnly: true }
					]}
				/>
			</div>
		</section>

		<section class="gallery-section">
			<h2>Overlays & feedback</h2>
			<div class="row">
				<Button onclick={() => (modalOpen = true)}>Modal</Button>
				<Button variant="danger" onclick={() => (confirmOpen = true)}>Confirm (danger)</Button>
				<Popover bind:open={popoverOpen} placement="bottom-start">
					{#snippet trigger(props)}
						<Button {...props}>Popover</Button>
					{/snippet}
					<div class="flex w-56 flex-col gap-2 p-2 text-sm text-fg-muted">
						<p class="font-medium text-fg">{t('筛选', 'Filters')}</p>
						<p>{t('浮层使用玻璃材质（§9）。', 'Floating layers use glass material (§9).')}</p>
					</div>
				</Popover>
				<Button onclick={() => toast.success(t('同步完成', 'Sync complete'))}>Toast success</Button>
				<Button
					onclick={() =>
						toast.info({
							title: 'Version 0.2.0 is available',
							action: { label: 'Settings', onClick: () => {} }
						})}
				>
					Toast + action
				</Button>
				<Button onclick={() => toast.error(t('同步失败', 'Sync failed'))}>Toast error</Button>
			</div>
			<div class="grid max-w-2xl gap-2">
				<InlineNotice tone="danger" title={t('Steam API Key 无效', 'Invalid Steam API key')}>
					{t('请检查密钥后重试。', 'Check the key and try again.')}
				</InlineNotice>
				<InlineNotice tone="warning">
					{t('部分数据可用：封面仍在同步中。', 'Partial data: covers are still syncing.')}
				</InlineNotice>
				<InlineNotice tone="info"
					>{t('数据每 5 秒刷新。', 'Data refreshes every 5 seconds.')}</InlineNotice
				>
			</div>
			<div class="row">
				<Badge>Neutral</Badge>
				<Badge tone="accent">Accent</Badge>
				<Badge tone="success">{t('已安装', 'Installed')}</Badge>
				<Badge tone="warning">Warning</Badge>
				<Badge tone="danger">Danger</Badge>
			</div>
		</section>

		<section class="gallery-section">
			<h2>Cards & stats (§21)</h2>
			<StatGroup boxed>
				<Stat label={t('总时长', 'Total time')} value={total.value} unit={total.unit} />
				<Stat label={t('日均', 'Daily avg')} value="1h 35m" />
				<Stat label={t('平均会话', 'Avg session')} value="1h 18m" />
				<Stat label={t('活跃天数', 'Active days')} value="57" />
			</StatGroup>
			<StatGroup boxed>
				<Stat label="Loading" value="" loading />
				<Stat label="Loading" value="" loading />
				<Stat label={t('英雄指标', 'Hero metric')} value="01:42:36" size="hero" />
			</StatGroup>
			<div class="grid grid-cols-3 gap-4">
				<Card><p class="text-sm text-fg-muted">Surface card — static information.</p></Card>
				<Card variant="interactive" onclick={() => toast.info('Clicked')}>
					<p class="text-sm text-fg-muted">Interactive card — hover tints, no lift.</p>
				</Card>
				<Card padding="lg"><Skeleton lines={3} /></Card>
			</div>
		</section>

		<section class="gallery-section">
			<h2>Game cards (§4)</h2>
			<div class="game-grid">
				{#each games as g, i (g.id)}
					<GameCard
						name={g.name}
						playtimeSeconds={g.seconds}
						lastPlayedAt={g.last}
						muted={i === 4}
						onclick={() => {}}
					/>
				{/each}
				<GameCard name="Never played" playtimeSeconds={0} onclick={() => {}} />
			</div>
		</section>

		<section class="gallery-section">
			<h2>Charts & panel states (§22–§26)</h2>
			<div class="row">
				<SegmentedControl
					bind:value={panelState}
					ariaLabel="Panel state"
					size="sm"
					options={[
						{ value: 'ready', label: 'Ready' },
						{ value: 'loading', label: 'Loading' },
						{ value: 'empty', label: 'Empty' },
						{ value: 'error', label: 'Error' }
					]}
				/>
			</div>
			<DailyBarChart
				items={panelState === 'empty' ? [] : daily}
				showAverage
				height="lg"
				description={t('柱：每日 · 线：7 日均值', 'Bars: daily · Line: 7-day average')}
				loading={panelState === 'loading'}
				error={panelState === 'error'}
				onRetry={() => (panelState = 'ready')}
			/>
			<div class="grid gap-4 lg:grid-cols-2">
				<RankingChart
					items={panelState === 'empty' ? [] : ranking}
					loading={panelState === 'loading'}
					error={panelState === 'error'}
				/>
				<DataPanel
					title="DataPanel sm"
					description="180px body"
					height="sm"
					state={panelState === 'ready' ? 'empty' : panelState}
					skeleton="bars"
				/>
			</div>
			<ActivityHeatmap
				days={panelState === 'empty' ? [] : year}
				loading={panelState === 'loading'}
				error={panelState === 'error'}
			/>
		</section>

		<section class="gallery-section">
			<h2>Empty / error / offline (§27)</h2>
			<div class="grid grid-cols-3 gap-4">
				<EmptyState
					bordered
					compact
					icon={Gamepad2}
					title={t('还没有游戏', 'No games yet')}
					description={t('连接或同步你的 Steam 游戏库。', 'Connect or sync your Steam library.')}
				>
					{#snippet action()}<Button size="sm" icon={RefreshCw}>{t('立即同步', 'Sync now')}</Button
						>{/snippet}
				</EmptyState>
				<EmptyState
					bordered
					compact
					tone="error"
					title={t('加载失败', 'Couldn’t load this')}
					description="database is locked"
				/>
				<EmptyState
					bordered
					compact
					tone="offline"
					title={t('离线', 'You’re offline')}
					description={t(
						'Steam 同步将在恢复网络后继续。',
						'Steam sync resumes when you’re back online.'
					)}
				/>
			</div>
		</section>
	</div>
</div>

<Modal
	open={modalOpen}
	title={t('导出数据', 'Export data')}
	description={t(
		'导出包含所有游玩记录的数据库副本。',
		'Export a copy of the database with all play history.'
	)}
	onClose={() => (modalOpen = false)}
>
	<Input placeholder="playarc-backup.db" />
	{#snippet footer()}
		<Button onclick={() => (modalOpen = false)}>{t('取消', 'Cancel')}</Button>
		<Button variant="primary" icon={Download} onclick={() => (modalOpen = false)}
			>{t('导出', 'Export')}</Button
		>
	{/snippet}
</Modal>

<ConfirmDialog
	open={confirmOpen}
	variant="danger"
	title={t('清除本地历史？', 'Clear local history?')}
	message={t('所有游玩记录将被永久删除。', 'All play sessions will be permanently deleted.')}
	confirmText={t('清除', 'Clear')}
	cancelText={t('取消', 'Cancel')}
	onConfirm={() => (confirmOpen = false)}
	onCancel={() => (confirmOpen = false)}
/>

<style>
	.gallery-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.gallery-section h2 {
		font-size: var(--text-xl);
		font-weight: 600;
		color: var(--text-primary);
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.hint {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.game-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(132px, 1fr));
		gap: var(--space-5) var(--space-4);
	}
</style>
