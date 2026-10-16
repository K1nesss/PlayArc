<!--
	Game Detail (DOIT.md §5) — "How have I played this individual game?"
	Order: Hero · Overview · 30-day playtime · Weekday × Hour · Sessions · Facts.
	The hero is the one place game colors are used most liberally.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ArrowLeft, EyeOff, Ellipsis, History, RefreshCw, Trash2 } from '@lucide/svelte';
	import {
		deleteGame,
		deleteGameSession,
		getAnalyticsOverview,
		getGameDetail,
		hideGame,
		type AnalyticsOverview,
		type GameDetail
	} from '$lib/api';
	import {
		formatDayTime,
		formatFullDate,
		formatPlaytime,
		formatPlaytimeParts,
		formatRelativeDay,
		formatTimeRange
	} from '$lib/format';
	import { t } from '$lib/i18n.svelte';
	import {
		Badge,
		Button,
		EmptyState,
		Icon,
		IconButton,
		InlineNotice,
		Popover,
		Skeleton,
		Stat,
		StatGroup,
		toast
	} from '$lib/components/ui';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import Cover from '$lib/components/game/Cover.svelte';
	import HeroBackdrop from '$lib/components/game/HeroBackdrop.svelte';
	import DailyBarChart from '$lib/components/charts/DailyBarChart.svelte';
	import WeekdayHourHeatmap from '$lib/components/charts/WeekdayHourHeatmap.svelte';

	let { data } = $props<{ data: { gameId: number } }>();

	const SESSION_LIMIT = 50;
	const SESSIONS_COLLAPSED = 10;

	let detail = $state<GameDetail | null>(null);
	let overview = $state<AnalyticsOverview | null>(null);
	let loading = $state(true);
	let errorMessage = $state('');
	let menuOpen = $state(false);
	let busy = $state(false);
	let showAllSessions = $state(false);

	type Confirm = {
		title: string;
		message: string;
		confirmText: string;
		variant: 'default' | 'danger';
		run: () => Promise<void>;
	};
	let confirm = $state<Confirm | null>(null);

	const headline = $derived(formatPlaytimeParts(detail?.total_playtime_seconds ?? 0));
	const last30 = $derived(overview ? overview.daily.slice(-30) : []);
	const median = $derived(overview?.summary.median_session_seconds ?? 0);
	const visibleSessions = $derived(
		detail
			? showAllSessions
				? detail.recent_sessions
				: detail.recent_sessions.slice(0, SESSIONS_COLLAPSED)
			: []
	);

	async function load() {
		loading = !detail;
		errorMessage = '';
		try {
			const [nextDetail, nextOverview] = await Promise.all([
				getGameDetail(data.gameId, SESSION_LIMIT),
				getAnalyticsOverview(null, data.gameId, 1)
			]);
			detail = nextDetail;
			overview = nextOverview;
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			loading = false;
		}
	}

	function back() {
		// History back restores the Games page's scroll and filters.
		if (window.history.length > 1) window.history.back();
		else void goto(resolve('/games'));
	}

	function ask(next: Confirm) {
		menuOpen = false;
		confirm = next;
	}

	async function runConfirm() {
		if (!confirm || busy) return;
		busy = true;
		try {
			await confirm.run();
			confirm = null;
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			confirm = null;
			toast.error({
				title: message.toLowerCase().includes('active session')
					? t('无法删除正在进行的会话', 'Can’t delete a running session')
					: t('操作失败', 'Something went wrong'),
				description: message.toLowerCase().includes('active session')
					? t(
							'请先退出游戏，等会话结束后再删除。',
							'Quit the game first, then delete it once the session ends.'
						)
					: message
			});
		} finally {
			busy = false;
		}
	}

	function confirmHide() {
		if (!detail) return;
		const game = detail;
		ask({
			title: t('隐藏这个游戏？', 'Hide this game?'),
			message: t(
				`“${game.name}”会从游戏库中隐藏，游玩记录会保留。可在 设置 → 数据 中恢复。`,
				`“${game.name}” will be hidden from your library. Its play history is kept, and you can restore it in Settings → Data.`
			),
			confirmText: t('隐藏', 'Hide'),
			variant: 'default',
			run: async () => {
				await hideGame(game.id);
				toast.success(t(`已隐藏“${game.name}”`, `Hid “${game.name}”`));
				void goto(resolve('/games'));
			}
		});
	}

	function confirmDeleteGame() {
		if (!detail) return;
		const game = detail;
		ask({
			title: t('删除这个游戏？', 'Delete this game?'),
			message: t(
				`“${game.name}”及其全部 ${game.session_count} 条游玩记录将被永久删除。`,
				`“${game.name}” and all ${game.session_count} of its play sessions will be permanently deleted.`
			),
			confirmText: t('删除游戏', 'Delete game'),
			variant: 'danger',
			run: async () => {
				await deleteGame(game.id);
				toast.success(t(`已删除“${game.name}”`, `Deleted “${game.name}”`));
				void goto(resolve('/games'));
			}
		});
	}

	function confirmDeleteSession(sessionId: number, start: number, duration: number) {
		ask({
			title: t('删除这条会话？', 'Delete this session?'),
			message: t(
				`${formatDayTime(start)} 的 ${formatPlaytime(duration)} 会话将被永久删除。`,
				`The ${formatPlaytime(duration)} session from ${formatDayTime(start)} will be permanently deleted.`
			),
			confirmText: t('删除', 'Delete'),
			variant: 'danger',
			run: async () => {
				await deleteGameSession(sessionId);
				toast.success(t('会话已删除', 'Session deleted'));
				await load();
			}
		});
	}

	onMount(() => {
		void load();
	});
</script>

<div class="app-page h-full overflow-y-auto">
	<div class="app-page-inner detail">
		<nav class="crumbs" aria-label={t('导航', 'Breadcrumb')}>
			<Button variant="ghost" size="sm" icon={ArrowLeft} onclick={back}
				>{t('游戏库', 'Games')}</Button
			>
		</nav>

		{#if errorMessage && !detail}
			<EmptyState
				bordered
				tone="error"
				title={t('无法加载这个游戏', 'Couldn’t load this game')}
				description={errorMessage}
			>
				{#snippet action()}
					<Button size="sm" icon={RefreshCw} onclick={load}>{t('重试', 'Retry')}</Button>
				{/snippet}
			</EmptyState>
		{:else if loading || !detail}
			<div class="hero-skeleton">
				<Skeleton width="128px" height="192px" radius="md" />
				<div class="flex flex-col gap-3">
					<Skeleton width="220px" height="24px" />
					<Skeleton width="120px" height="38px" />
					<Skeleton width="140px" height="12px" />
				</div>
			</div>
		{:else}
			<!-- Hero -->
			<HeroBackdrop
				artPath={detail.hero_path ?? detail.cover_path}
				accent={detail.accent_color}
				intensity="rich"
			>
				<div class="hero">
					<Cover name={detail.name} path={detail.cover_path} size={128} />
					<div class="hero-text">
						<div class="flex flex-wrap items-center gap-2">
							<h1 class="hero-title">{detail.name}</h1>
							{#if !detail.is_installed}<Badge>{t('未安装', 'Not installed')}</Badge>{/if}
						</div>
						<p class="hero-metric">
							{headline.value}<span class="hero-unit">{headline.unit}</span>
						</p>
						<p class="hero-meta">
							{detail.last_played_at
								? t(
										`最后游玩：${formatRelativeDay(detail.last_played_at)}`,
										`Last played ${formatRelativeDay(detail.last_played_at)}`
									)
								: t('尚未游玩', 'Not played yet')}
						</p>
					</div>
					<div class="hero-actions">
						<Popover bind:open={menuOpen} placement="bottom-end" role="menu">
							{#snippet trigger(props)}
								<IconButton
									icon={Ellipsis}
									label={t('更多操作', 'More actions')}
									variant="secondary"
									{...props}
								/>
							{/snippet}
							<div class="menu" role="menu">
								<button type="button" role="menuitem" class="menu-item" onclick={confirmHide}>
									<Icon icon={EyeOff} size="button" />{t('从游戏库隐藏', 'Hide from library')}
								</button>
								<button
									type="button"
									role="menuitem"
									class="menu-item menu-item--danger"
									onclick={confirmDeleteGame}
								>
									<Icon icon={Trash2} size="button" />{t(
										'删除游戏和记录',
										'Delete game and history'
									)}
								</button>
							</div>
						</Popover>
					</div>
				</div>
			</HeroBackdrop>

			<!-- Overview metrics: typography, not boxes (§5). Total is the hero metric above. -->
			<StatGroup class="overview">
				<Stat label={t('会话', 'Sessions')} value={detail.session_count} size="md" />
				<Stat
					label={t('平均会话', 'Average session')}
					value={formatPlaytime(detail.average_session_seconds)}
					size="md"
				/>
				<Stat label={t('中位会话', 'Median session')} value={formatPlaytime(median)} size="md" />
				<Stat
					label={t('最长会话', 'Longest session')}
					value={formatPlaytime(overview?.summary.longest_session_seconds ?? 0)}
					size="md"
				/>
			</StatGroup>

			{#if errorMessage}
				<InlineNotice tone="danger">{errorMessage}</InlineNotice>
			{/if}

			<DailyBarChart
				items={last30}
				title={t('近 30 天', 'Last 30 days')}
				description={t('每日游玩时长', 'Daily playtime')}
				height="sm"
				loading={!overview}
			/>

			<WeekdayHourHeatmap
				grid={overview?.weekday_hour ?? []}
				description={t('全部游玩记录', 'All play history')}
				loading={!overview}
			/>

			<!-- Session history -->
			<section class="panel" aria-labelledby="sessions-title">
				<header class="panel-header">
					<div>
						<h2 id="sessions-title" class="panel-title">{t('会话记录', 'Session history')}</h2>
						{#if detail.session_count > detail.recent_sessions.length}
							<p class="panel-description">
								{t(
									`最近 ${detail.recent_sessions.length} 条，共 ${detail.session_count} 条`,
									`Latest ${detail.recent_sessions.length} of ${detail.session_count}`
								)}
							</p>
						{/if}
					</div>
					<Button
						size="sm"
						variant="ghost"
						icon={History}
						href={`${resolve('/timeline')}?game=${detail.id}`}
					>
						{t('在时间线中查看', 'View in Timeline')}
					</Button>
				</header>
				{#if detail.recent_sessions.length === 0}
					<EmptyState compact title={t('还没有会话', 'No sessions yet')} />
				{:else}
					<ul class="sessions">
						{#each visibleSessions as session (session.session_id)}
							<li class="session">
								<span class="session-day">{formatRelativeDay(session.start_time)}</span>
								<span class="session-range tabular-nums"
									>{formatTimeRange(session.start_time, session.end_time)}</span
								>
								{#if !session.end_time}<Badge tone="success">{t('进行中', 'Live')}</Badge>{/if}
								<span class="session-duration tabular-nums"
									>{formatPlaytime(session.duration_seconds)}</span
								>
								<span class="session-action">
									<IconButton
										icon={Trash2}
										label={t('删除会话', 'Delete session')}
										size="sm"
										disabled={!session.end_time}
										onclick={() =>
											confirmDeleteSession(
												session.session_id,
												session.start_time,
												session.duration_seconds
											)}
									/>
								</span>
							</li>
						{/each}
					</ul>
					{#if detail.recent_sessions.length > SESSIONS_COLLAPSED}
						<div class="mt-2">
							<Button
								size="sm"
								variant="ghost"
								onclick={() => (showAllSessions = !showAllSessions)}
							>
								{showAllSessions
									? t('收起', 'Show less')
									: t(
											`显示更多（${detail.recent_sessions.length}）`,
											`Show more (${detail.recent_sessions.length})`
										)}
							</Button>
						</div>
					{/if}
				{/if}
			</section>

			<!-- Facts -->
			<section class="panel" aria-labelledby="facts-title">
				<h2 id="facts-title" class="panel-title mb-3">{t('详情', 'Details')}</h2>
				<dl class="facts">
					<div>
						<dt>{t('首次游玩', 'First played')}</dt>
						<dd>
							{overview?.summary.first_played_at
								? formatFullDate(new Date(overview.summary.first_played_at * 1000))
								: '—'}
						</dd>
					</div>
					<div>
						<dt>{t('最后游玩', 'Last played')}</dt>
						<dd>{formatDayTime(detail.last_played_at)}</dd>
					</div>
					<div>
						<dt>{t('总时长', 'Total playtime')}</dt>
						<dd>{formatPlaytime(detail.total_playtime_seconds)}</dd>
					</div>
					<div>
						<dt>{t('活跃天数', 'Days played')}</dt>
						<dd>{overview?.summary.active_days ?? '—'}</dd>
					</div>
					<div>
						<dt>{t('加入游戏库', 'Added')}</dt>
						<dd>{formatFullDate(new Date(detail.created_at * 1000))}</dd>
					</div>
					{#if detail.appid}<div>
							<dt>Steam App ID</dt>
							<dd class="tabular-nums">{detail.appid}</dd>
						</div>{/if}
					{#if detail.install_path}
						<div class="facts-wide">
							<dt>{t('安装位置', 'Install location')}</dt>
							<dd class="path" title={detail.install_path}>{detail.install_path}</dd>
						</div>
					{/if}
				</dl>
			</section>
		{/if}
	</div>
</div>

<ConfirmDialog
	open={confirm !== null}
	title={confirm?.title ?? ''}
	message={confirm?.message ?? ''}
	confirmText={confirm?.confirmText ?? ''}
	cancelText={t('取消', 'Cancel')}
	variant={confirm?.variant ?? 'default'}
	loading={busy}
	onConfirm={runConfirm}
	onCancel={() => !busy && (confirm = null)}
/>

<style>
	.detail {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding-bottom: var(--space-8);
	}
	.detail > :global(*) {
		max-width: 1080px;
	}
	.crumbs {
		margin: calc(-1 * var(--space-2)) 0 0 calc(-1 * var(--space-2));
	}

	.hero-skeleton {
		display: flex;
		align-items: center;
		gap: var(--space-6);
		padding: var(--space-6);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
	}
	.hero {
		display: flex;
		align-items: flex-end;
		gap: var(--space-6);
		padding: var(--space-6);
	}
	.hero :global(.cover) {
		box-shadow:
			inset 0 0 0 1px var(--border-subtle),
			var(--elevation-artwork);
	}
	.hero-text {
		flex: 1;
		min-width: 0;
		padding-bottom: var(--space-1);
	}
	.hero-title {
		font-size: var(--text-2xl);
		font-weight: 600;
		line-height: var(--text-2xl--line-height);
		color: var(--text-primary);
	}
	.hero-metric {
		margin-top: var(--space-3);
		font-size: var(--text-4xl);
		font-weight: 600;
		line-height: var(--text-4xl--line-height);
		letter-spacing: -0.015em;
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
	}
	.hero-unit {
		margin-left: 2px;
		font-size: 0.55em;
		font-weight: 500;
		color: var(--text-secondary);
	}
	.hero-meta {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.hero-actions {
		align-self: flex-start;
	}

	.menu {
		display: flex;
		flex-direction: column;
		min-width: 200px;
	}
	.menu-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		height: 32px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-base);
		color: var(--text-primary);
		text-align: left;
	}
	.menu-item:hover,
	.menu-item:focus-visible {
		background: var(--state-hover);
		outline: none;
	}
	.menu-item--danger {
		color: var(--danger);
	}
	.menu-item--danger:hover {
		background: var(--danger-soft);
	}

	.detail :global(.overview) {
		padding: var(--space-2) var(--space-1);
	}

	.panel {
		padding: var(--space-4) var(--space-5) var(--space-5);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
	}
	.panel-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-3);
		margin-bottom: var(--space-2);
	}
	.panel-title {
		font-size: var(--text-base);
		font-weight: 600;
		color: var(--text-primary);
	}
	.panel-description {
		margin-top: 2px;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}

	.sessions {
		margin: 0 calc(-1 * var(--space-2));
	}
	.session {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		height: 40px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-md);
	}
	.session:hover {
		background: var(--state-hover);
	}
	.session-day {
		width: 120px;
		font-size: var(--text-sm);
		color: var(--text-primary);
	}
	.session-range {
		flex: 1;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.session-duration {
		min-width: 64px;
		text-align: right;
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-primary);
	}
	.session-action {
		opacity: 0;
		transition: opacity var(--duration-fast) var(--ease-standard);
	}
	.session:hover .session-action,
	.session:focus-within .session-action {
		opacity: 1;
	}

	.facts {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: var(--space-4) var(--space-6);
	}
	.facts dt {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.facts dd {
		margin-top: 2px;
		font-size: var(--text-base);
		color: var(--text-primary);
	}
	.facts-wide {
		grid-column: 1 / -1;
	}
	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-family: var(--font-mono);
		font-size: var(--text-sm) !important;
		color: var(--text-secondary) !important;
	}
</style>
