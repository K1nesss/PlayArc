<!--
	Home (DOIT.md §3) — real-time status, not a dashboard.
	Answers "What am I playing right now?". Long-term charts live in Analysis.

	Playing: hero (artwork-tinted) with live session clock + today's context.
	Idle:    quiet "No game running" + the last played game, so Home is never blank.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { Clock, Gamepad2 } from '@lucide/svelte';
	import { getHomeSnapshot, type HomeSnapshot } from '$lib/api';
	import { loadSettingsFromStorage } from '$lib/settings';
	import { formatClock, formatDayTime, formatPlaytime, formatTime } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';
	import { EmptyState, Icon, InlineNotice, Skeleton, Stat, StatGroup } from '$lib/components/ui';
	import Cover from '$lib/components/game/Cover.svelte';
	import HeroBackdrop from '$lib/components/game/HeroBackdrop.svelte';

	let snapshot = $state<HomeSnapshot | null>(null);
	let errorMessage = $state('');
	let nowSeconds = $state(Math.floor(Date.now() / 1000));
	// Offset between the backend clock and this webview, so the live timer agrees with it.
	let clockSkew = 0;

	const playing = $derived(snapshot?.now_playing ?? null);
	const elapsed = $derived(playing ? Math.max(0, nowSeconds - playing.start_time) : 0);
	/** Today's total grows live while a session is running. */
	const todaySeconds = $derived.by(() => {
		if (!snapshot) return 0;
		const drift = playing ? Math.max(0, nowSeconds - snapshot.now) : 0;
		return snapshot.today.seconds + drift;
	});
	const todayLabel = $derived(
		new Intl.DateTimeFormat(locale.current, {
			weekday: 'long',
			month: 'long',
			day: 'numeric'
		}).format(new Date(nowSeconds * 1000))
	);

	async function refresh() {
		try {
			const next = await getHomeSnapshot();
			clockSkew = next.now - Math.floor(Date.now() / 1000);
			snapshot = next;
			errorMessage = '';
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		}
	}

	onMount(() => {
		let pollTimer: number | null = null;
		let pollSeconds = 5;

		const schedule = () => {
			if (pollTimer !== null) window.clearTimeout(pollTimer);
			// Poll less when the window is hidden.
			const delay = (document.visibilityState === 'visible' ? pollSeconds : 30) * 1000;
			pollTimer = window.setTimeout(async () => {
				await refresh();
				schedule();
			}, delay);
		};

		const tick = window.setInterval(() => {
			nowSeconds = Math.floor(Date.now() / 1000) + clockSkew;
		}, 1000);

		const onVisible = () => {
			if (document.visibilityState === 'visible') void refresh().then(schedule);
		};
		document.addEventListener('visibilitychange', onVisible);

		void (async () => {
			const settings = await loadSettingsFromStorage();
			pollSeconds = settings.dashboardRefreshSeconds;
			await refresh();
			schedule();
		})();

		return () => {
			window.clearInterval(tick);
			if (pollTimer !== null) window.clearTimeout(pollTimer);
			document.removeEventListener('visibilitychange', onVisible);
		};
	});
</script>

<div class="app-page h-full overflow-y-auto">
	<div class="app-page-inner home">
		<header class="page-header mb-0!">
			<div>
				<h1 class="page-title">{t('首页', 'Home')}</h1>
				<p class="page-subtitle">{todayLabel}</p>
			</div>
		</header>

		{#if errorMessage && !snapshot}
			<InlineNotice tone="danger" title={t('无法加载当前状态', 'Couldn’t load current status')}>
				{errorMessage}
			</InlineNotice>
		{/if}

		{#if !snapshot && !errorMessage}
			<!-- Loading -->
			<div class="hero-skeleton">
				<Skeleton width="104px" height="156px" radius="md" />
				<div class="flex flex-1 flex-col gap-3">
					<Skeleton width="96px" height="12px" />
					<Skeleton width="240px" height="24px" />
					<Skeleton width="180px" height="38px" />
				</div>
			</div>
			<StatGroup boxed>
				<Stat label="—" value="" loading />
				<Stat label="—" value="" loading />
				<Stat label="—" value="" loading />
			</StatGroup>
		{:else if snapshot && playing}
			<!-- Playing -->
			<a
				href={resolve('/games/[id]', { id: String(playing.game.game_id) })}
				class="hero-link"
				aria-label={t(`正在游玩 ${playing.game.name}`, `Now playing ${playing.game.name}`)}
			>
				<HeroBackdrop
					artPath={playing.game.hero_path ?? playing.game.cover_path}
					accent={playing.game.accent_color}
				>
					<div class="hero-body">
						<Cover name={playing.game.name} path={playing.game.cover_path} size={112} />
						<div class="hero-text">
							<p class="hero-eyebrow">
								<span class="live-dot" aria-hidden="true"></span>
								{t('正在游玩', 'Now playing')}
							</p>
							<h2 class="hero-title">{playing.game.name}</h2>
							<p class="hero-clock" aria-live="off">{formatClock(elapsed)}</p>
							<p class="hero-meta">
								{t('当前会话', 'Current session')}
								<span aria-hidden="true">·</span>
								{t(
									`开始于 ${formatTime(playing.start_time)}`,
									`Started at ${formatTime(playing.start_time)}`
								)}
							</p>
						</div>
					</div>
				</HeroBackdrop>
			</a>

			<StatGroup boxed>
				<Stat label={t('今日时长', 'Today’s playtime')} value={formatPlaytime(todaySeconds)} />
				<Stat label={t('当前会话', 'Current session')} value={formatPlaytime(elapsed)} />
				<Stat label={t('今日会话', 'Today’s sessions')} value={snapshot.today.sessions} />
			</StatGroup>
		{:else if snapshot && snapshot.last_session}
			<!-- Idle -->
			<section class="idle" aria-live="polite">
				<span class="idle-icon"><Icon icon={Gamepad2} size={24} /></span>
				<div>
					<p class="idle-title">{t('没有正在运行的游戏', 'No game running')}</p>
					<p class="idle-text">
						{t('启动游戏后，它会自动显示在这里。', 'Your current game will appear here.')}
					</p>
				</div>
			</section>

			<section aria-labelledby="last-played">
				<h2 id="last-played" class="section-label">{t('最近游玩', 'Last played')}</h2>
				<a
					class="last-played"
					href={resolve('/games/[id]', { id: String(snapshot.last_session.game.game_id) })}
				>
					<Cover
						name={snapshot.last_session.game.name}
						path={snapshot.last_session.game.cover_path}
						size={64}
						radius="sm"
					/>
					<div class="min-w-0 flex-1">
						<p class="last-name">{snapshot.last_session.game.name}</p>
						<p class="last-when">
							<Icon icon={Clock} size="status" />
							{formatDayTime(snapshot.last_session.end_time)}
						</p>
					</div>
					<div class="last-session">
						<span class="last-session-label">{t('上次会话', 'Last session')}</span>
						<span class="last-session-value">
							{formatPlaytime(snapshot.last_session.duration_seconds)}
						</span>
					</div>
				</a>
			</section>

			<StatGroup boxed>
				<Stat label={t('今日时长', 'Today’s playtime')} value={formatPlaytime(todaySeconds)} />
				<Stat label={t('今日会话', 'Today’s sessions')} value={snapshot.today.sessions} />
			</StatGroup>
		{:else if snapshot}
			<!-- Nothing recorded yet -->
			<EmptyState
				bordered
				icon={Gamepad2}
				title={t('还没有游玩记录', 'No play history yet')}
				description={t(
					'启动任意游戏，PlayArc 会自动开始记录。',
					'Launch any game and PlayArc will start tracking automatically.'
				)}
			/>
		{/if}
	</div>
</div>

<style>
	.home {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding-bottom: var(--space-8);
	}
	/* Same column and title position as every page; content keeps a reading width. */
	.home > :global(*) {
		max-width: 960px;
	}

	.hero-skeleton {
		display: flex;
		gap: var(--space-6);
		padding: var(--space-6);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
	}

	.hero-link {
		display: block;
		border-radius: var(--radius-xl);
		color: inherit;
	}
	.hero-link :global(.hero) {
		transition: border-color var(--duration-fast) var(--ease-standard);
	}
	.hero-link:hover :global(.hero) {
		border-color: var(--border-strong);
	}

	.hero-body {
		display: flex;
		align-items: center;
		gap: var(--space-6);
		padding: var(--space-6);
	}
	.hero-body :global(.cover) {
		box-shadow:
			inset 0 0 0 1px var(--border-subtle),
			var(--elevation-artwork);
	}
	.hero-text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.hero-eyebrow {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-secondary);
	}
	.live-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--success);
	}
	.hero-title {
		margin-top: var(--space-1);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-2xl);
		font-weight: 600;
		line-height: var(--text-2xl--line-height);
		color: var(--text-primary);
	}
	.hero-clock {
		margin-top: var(--space-3);
		font-size: var(--text-4xl);
		font-weight: 600;
		line-height: var(--text-4xl--line-height);
		letter-spacing: -0.01em;
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
	}
	.hero-meta {
		display: flex;
		gap: 6px;
		margin-top: 2px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.idle {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-6);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
	}
	.idle-icon {
		display: grid;
		place-items: center;
		width: 48px;
		height: 48px;
		border-radius: var(--radius-xl);
		background: var(--bg-subtle);
		color: var(--text-tertiary);
	}
	.idle-title {
		font-size: var(--text-lg);
		font-weight: 600;
		color: var(--text-primary);
	}
	.idle-text {
		margin-top: 2px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.section-label {
		margin: var(--space-2) 0 var(--space-2);
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.last-played {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-3) var(--space-4) var(--space-3) var(--space-3);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		color: inherit;
		transition:
			border-color var(--duration-fast) var(--ease-standard),
			background-color var(--duration-fast) var(--ease-standard);
	}
	.last-played:hover {
		border-color: var(--border-strong);
		background: color-mix(in oklab, var(--bg-surface) 97%, var(--text-primary));
	}
	.last-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-lg);
		font-weight: 600;
		color: var(--text-primary);
	}
	.last-when {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 2px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.last-session {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
	}
	.last-session-label {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.last-session-value {
		font-size: var(--text-xl);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
	}
</style>
