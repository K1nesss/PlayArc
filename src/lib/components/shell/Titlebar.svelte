<!--
	Custom title bar (DOIT.md §13). 44px drag region with Windows-style caption
	buttons. On Windows the buttons use the system's own glyph font (Segoe Fluent
	Icons on 11, Segoe MDL2 Assets on 10), so they match native windows and stay
	crisp at 100 / 125 / 150% scaling. Other platforms get equivalent SVGs.

	Double-clicking the drag region toggles maximize (handled by Tauri).
	Known limitation: the Windows 11 Snap Layouts flyout on hovering maximize is
	only available to native caption buttons; Win+Z still works.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { windowState } from '$lib/shell/window.svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		/** Optional content on the left of the bar (e.g. page context). */
		children?: Snippet;
	}

	let { children }: Props = $props();

	const minimize = () => invoke('window_minimize').catch(() => {});
	const toggleMaximize = () => invoke('window_toggle_maximize').catch(() => {});
	const close = () => invoke('window_close').catch(() => {});

	// Native-feeling: after the window regains focus, don't show a stale hover
	// state on whichever button the cursor happened to be over.
	let suppressHover = $state(false);
	$effect(() => {
		if (!windowState.focused) {
			suppressHover = true;
			return;
		}
		const release = () => (suppressHover = false);
		window.addEventListener('pointermove', release, { once: true });
		const timer = window.setTimeout(release, 800);
		return () => {
			window.removeEventListener('pointermove', release);
			window.clearTimeout(timer);
		};
	});
</script>

<header class="shell-titlebar" data-tauri-drag-region>
	<div class="titlebar-content" data-tauri-drag-region>
		{@render children?.()}
	</div>

	<div
		class="caption-buttons"
		class:suppress-hover={suppressHover}
		class:glyphs={windowState.isWindows}
	>
		<button
			type="button"
			class="caption"
			onclick={minimize}
			aria-label={t('最小化', 'Minimize')}
			title={t('最小化', 'Minimize')}
		>
			{#if windowState.isWindows}
				<span class="glyph">&#xE921;</span>
			{:else}
				<svg viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5h10" /></svg>
			{/if}
		</button>
		<button
			type="button"
			class="caption"
			onclick={toggleMaximize}
			aria-label={windowState.maximized ? t('还原', 'Restore') : t('最大化', 'Maximize')}
			title={windowState.maximized ? t('还原', 'Restore') : t('最大化', 'Maximize')}
		>
			{#if windowState.isWindows}
				<span class="glyph">{windowState.maximized ? '' : ''}</span>
			{:else if windowState.maximized}
				<svg viewBox="0 0 10 10" aria-hidden="true"
					><path d="M2.5 2.5V.5h7v7h-2M.5 2.5h7v7h-7z" /></svg
				>
			{:else}
				<svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5h9v9h-9z" /></svg>
			{/if}
		</button>
		<button
			type="button"
			class="caption caption--close"
			onclick={close}
			aria-label={t('关闭', 'Close')}
			title={t('关闭', 'Close')}
		>
			{#if windowState.isWindows}
				<span class="glyph">&#xE8BB;</span>
			{:else}
				<svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5l9 9M9.5.5l-9 9" /></svg>
			{/if}
		</button>
	</div>
</header>

<style>
	.shell-titlebar {
		display: flex;
		align-items: stretch;
		height: var(--titlebar-height);
		flex-shrink: 0;
		user-select: none;
		-webkit-user-select: none;
	}
	.titlebar-content {
		display: flex;
		align-items: center;
		flex: 1;
		min-width: 0;
		padding: 0 var(--space-4);
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.caption-buttons {
		display: flex;
		align-items: stretch;
		flex-shrink: 0;
	}
	.caption {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 46px;
		color: var(--text-primary);
		cursor: default;
		transition:
			background-color var(--duration-fast) linear,
			color var(--duration-fast) linear;
	}
	:global(html.window-inactive) .caption {
		color: var(--text-tertiary);
	}
	.glyph {
		font-family: 'Segoe Fluent Icons', 'Segoe MDL2 Assets';
		font-size: 10px;
		line-height: 1;
		-webkit-font-smoothing: auto;
	}
	.caption svg {
		width: 10px;
		height: 10px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1;
		shape-rendering: crispEdges;
	}
	.caption:focus-visible {
		outline-offset: -2px;
	}

	/* Windows 11 caption colors */
	.caption-buttons:not(.suppress-hover) .caption:hover {
		background: rgba(0, 0, 0, 0.0373);
	}
	.caption:active {
		background: rgba(0, 0, 0, 0.0241);
	}
	:global(.dark) .caption-buttons:not(.suppress-hover) .caption:hover {
		background: rgba(255, 255, 255, 0.0605);
	}
	:global(.dark) .caption:active {
		background: rgba(255, 255, 255, 0.0419);
	}
	.caption-buttons:not(.suppress-hover) .caption--close:hover,
	:global(.dark) .caption-buttons:not(.suppress-hover) .caption--close:hover {
		background: #c42b1c;
		color: #fff;
	}
	.caption--close:active,
	:global(.dark) .caption--close:active {
		background: rgba(196, 43, 28, 0.9);
		color: rgba(255, 255, 255, 0.7);
	}
</style>
