<!--
	Game card for the library grid (DOIT.md §4). The cover IS the boundary — no
	card border, no heavy shadow. Hover: lift 2px, slight brightness, text
	secondary → primary. Covers are Steam's 600×900 capsules (2:3).
-->
<script lang="ts">
	import { formatPlaytimeHeadline, formatTimeAgo } from '$lib/format';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		name: string;
		coverSrc?: string | null;
		playtimeSeconds: number;
		lastPlayedAt?: number | null;
		href?: string;
		onclick?: () => void;
		/** Dim uninstalled games slightly. */
		muted?: boolean;
	}

	let {
		name,
		coverSrc,
		playtimeSeconds,
		lastPlayedAt,
		href,
		onclick,
		muted = false
	}: Props = $props();

	let failed = $state(false);
	$effect(() => {
		void coverSrc;
		failed = false;
	});

	const initial = $derived(name.trim().charAt(0).toUpperCase() || '?');
</script>

{#snippet body()}
	<div class="ui-game-cover">
		{#if coverSrc && !failed}
			<img src={coverSrc} alt="" loading="lazy" decoding="async" onerror={() => (failed = true)} />
		{:else}
			<span class="ui-game-placeholder" aria-hidden="true">{initial}</span>
		{/if}
	</div>
	<div class="ui-game-meta">
		<p class="ui-game-name" title={name}>{name}</p>
		<p class="ui-game-line">
			<span class="tabular-nums">{formatPlaytimeHeadline(playtimeSeconds)}</span>
			<span class="ui-game-dot" aria-hidden="true">·</span>
			<span>{lastPlayedAt ? formatTimeAgo(lastPlayedAt) : t('未游玩', 'Never played')}</span>
		</p>
	</div>
{/snippet}

{#if href}
	<!-- `href` must already be resolve()'d by the caller. -->
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
	<a {href} class={`ui-game-card ${muted ? 'is-muted' : ''}`} aria-label={name}>{@render body()}</a>
{:else}
	<button
		type="button"
		class={`ui-game-card ${muted ? 'is-muted' : ''}`}
		{onclick}
		aria-label={name}
	>
		{@render body()}
	</button>
{/if}

<style>
	.ui-game-card {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		min-width: 0;
		text-align: left;
		color: inherit;
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: transform var(--duration-fast) var(--ease-standard);
	}
	.ui-game-card:hover {
		transform: translateY(-2px);
	}
	.ui-game-card:focus-visible {
		outline-offset: 4px;
	}

	.ui-game-cover {
		position: relative;
		aspect-ratio: 2 / 3;
		overflow: hidden;
		border-radius: var(--radius-md);
		background: var(--bg-subtle);
		/* hairline inset so light covers don't bleed into the canvas */
		box-shadow: inset 0 0 0 1px var(--border-subtle);
	}
	.ui-game-cover img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		transition: filter var(--duration-fast) var(--ease-standard);
	}
	.ui-game-card:hover .ui-game-cover img {
		filter: brightness(1.06);
	}
	.is-muted .ui-game-cover img {
		filter: saturate(0.75) brightness(0.92);
	}

	.ui-game-placeholder {
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		font-size: 32px;
		font-weight: 600;
		color: var(--text-tertiary);
	}

	.ui-game-meta {
		min-width: 0;
		padding-inline: 2px;
	}
	.ui-game-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-sm);
		font-weight: 500;
		color: var(--text-secondary);
		transition: color var(--duration-fast) var(--ease-standard);
	}
	.ui-game-card:hover .ui-game-name {
		color: var(--text-primary);
	}
	.ui-game-line {
		display: flex;
		gap: 4px;
		margin-top: 2px;
		overflow: hidden;
		white-space: nowrap;
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.ui-game-dot {
		opacity: 0.6;
	}
</style>
