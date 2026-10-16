<!--
	Tray menu — what right-clicking the tray icon opens (Settings → General →
	Tray menu style → PlayArc). A plain list of actions in the app's own style;
	no status or other information.

	A separate, borderless, transparent window (`tray-panel`, created hidden by
	src-tauri/src/system/tray.rs) that stays loaded so it opens instantly; the
	backend emits `tray-panel-open` right before showing it. The menu is drawn
	inside a transparent margin that holds its shadow; clicking that margin,
	pressing Esc, or focusing anything else closes it.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import {
		AppWindow,
		Check,
		CircleAlert,
		LoaderCircle,
		Power,
		RefreshCw,
		Settings
	} from '@lucide/svelte';
	import { mergeSettings, type AppSettings } from '$lib/settings';
	import { setLanguage, t } from '$lib/i18n.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';

	type SyncView = 'idle' | 'syncing' | 'done' | 'error';

	let settings = $state<AppSettings>(mergeSettings({}));
	let sync = $state<SyncView>('idle');
	let syncError = $state('');
	let menu = $state<HTMLElement | null>(null);
	let root = $state<HTMLElement | null>(null);

	const steamConfigured = $derived(Boolean(settings.apiKey && settings.steam64Id));
	const syncIcon = $derived(
		sync === 'syncing'
			? LoaderCircle
			: sync === 'done'
				? Check
				: sync === 'error'
					? CircleAlert
					: RefreshCw
	);

	function act(action: string) {
		void invoke('tray_panel_action', { action }).catch(() => {});
	}

	function applyAppearance(next: AppSettings) {
		setLanguage(next.language);
		const dark =
			next.themeMode === 'dark' ||
			(next.themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
		document.documentElement.classList.toggle('dark', dark);
		document.documentElement.classList.toggle('animations-disabled', !next.animationsEnabled);
	}

	/** Re-read on every open: other windows may have changed theme, language or the toggles. */
	async function refresh() {
		try {
			settings = mergeSettings((await invoke<Partial<AppSettings>>('load_app_settings')) ?? {});
		} catch {
			// Keep the last known settings.
		}
		applyAppearance(settings);
	}

	function startSync() {
		if (sync === 'syncing') return;
		sync = 'syncing';
		syncError = '';
		act('sync');
	}

	function toggleAutostart() {
		settings.launchOnStartup = !settings.launchOnStartup; // optimistic; confirmed by settings-updated
		act('autostart');
	}

	/** Arrow keys move between items, like a menu; Esc closes. */
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			act('hide');
			return;
		}
		if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key) || !menu) return;
		const items = [...menu.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')];
		if (!items.length) return;
		event.preventDefault();
		const index = items.indexOf(document.activeElement as HTMLButtonElement);
		const next =
			event.key === 'Home'
				? 0
				: event.key === 'End'
					? items.length - 1
					: event.key === 'ArrowDown'
						? (index + 1) % items.length
						: (index - 1 + items.length) % items.length;
		items[next].focus();
	}

	onMount(() => {
		document.documentElement.classList.add('tray-panel-window');
		void refresh();

		// Fit the window to the menu (labels differ in length by language).
		const reportHeight = () => {
			if (root) void invoke('tray_panel_resize', { height: root.offsetHeight }).catch(() => {});
		};
		const resizeObserver = new ResizeObserver(reportHeight);
		if (root) resizeObserver.observe(root);

		const media = window.matchMedia('(prefers-color-scheme: dark)');
		const onSystemTheme = () => applyAppearance(settings);
		media.addEventListener('change', onSystemTheme);

		const preventContextMenu = (event: MouseEvent) => event.preventDefault();
		window.addEventListener('contextmenu', preventContextMenu);

		let disposed = false;
		let unlisten: (() => void)[] = [];
		let syncReset: number | null = null;
		void (async () => {
			try {
				const { listen } = await import('@tauri-apps/api/event');
				const handles = await Promise.all([
					listen('tray-panel-open', () => {
						if (sync !== 'syncing') sync = 'idle';
						void refresh();
						menu?.focus();
					}),
					listen<Partial<AppSettings>>('settings-updated', (event) => {
						settings = mergeSettings({ ...settings, ...event.payload });
						applyAppearance(settings);
					}),
					listen<{ state: string; message?: string }>('tray-sync-result', (event) => {
						const { state, message } = event.payload;
						if (state === 'busy') return; // already running: keep the spinner
						sync = state === 'done' ? 'done' : state === 'error' ? 'error' : 'idle';
						syncError = message ?? '';
						if (syncReset !== null) window.clearTimeout(syncReset);
						syncReset = window.setTimeout(() => {
							if (sync !== 'syncing') sync = 'idle';
						}, 3000);
					})
				]);
				if (disposed) handles.forEach((fn) => fn());
				else unlisten = handles;
			} catch {
				// Browser preview: no backend events.
			}
		})();

		return () => {
			disposed = true;
			unlisten.forEach((fn) => fn());
			if (syncReset !== null) window.clearTimeout(syncReset);
			resizeObserver.disconnect();
			media.removeEventListener('change', onSystemTheme);
			window.removeEventListener('contextmenu', preventContextMenu);
		};
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div
	class="tray-root"
	role="presentation"
	bind:this={root}
	onmousedown={(event) => {
		if (event.target === root) act('hide');
	}}
>
	<div class="tray-menu" bind:this={menu} tabindex="-1" role="menu" aria-label="PlayArc">
		<button type="button" class="tray-item" role="menuitem" onclick={() => act('open')}>
			<Icon icon={AppWindow} size="button" />
			<span class="tray-label">{t('显示主窗口', 'Show window')}</span>
		</button>
		<button
			type="button"
			class="tray-item"
			role="menuitem"
			disabled={!steamConfigured}
			title={sync === 'error' ? syncError : undefined}
			onclick={startSync}
		>
			<span
				class="tray-icon"
				class:tray-spin={sync === 'syncing'}
				class:tray-ok={sync === 'done'}
				class:tray-error={sync === 'error'}
			>
				<Icon icon={syncIcon} size="button" />
			</span>
			<span class="tray-label">{t('同步 Steam 数据', 'Sync Steam data')}</span>
		</button>

		<div class="tray-divider" role="separator"></div>

		<button
			type="button"
			class="tray-item"
			role="menuitemcheckbox"
			aria-checked={settings.launchOnStartup}
			onclick={toggleAutostart}
		>
			<Icon icon={Power} size="button" />
			<span class="tray-label">{t('开机启动', 'Launch at startup')}</span>
			<span class="tray-switch" class:on={settings.launchOnStartup} aria-hidden="true"></span>
		</button>
		<button type="button" class="tray-item" role="menuitem" onclick={() => act('settings')}>
			<Icon icon={Settings} size="button" />
			<span class="tray-label">{t('设置', 'Settings')}</span>
		</button>

		<div class="tray-divider" role="separator"></div>

		<button type="button" class="tray-item tray-quit" role="menuitem" onclick={() => act('quit')}>
			<span class="tray-icon-spacer" aria-hidden="true"></span>
			<span class="tray-label">{t('退出', 'Quit')}</span>
		</button>
	</div>
</div>

<style>
	/* The window is transparent; only the menu is painted. */
	:global(html.tray-panel-window),
	:global(html.tray-panel-window body) {
		background: transparent;
		overflow: hidden;
	}

	.tray-root {
		/* Room for the shadow, inside the window. Matches PANEL_WIDTH (232) in tray.rs. */
		width: 232px;
		padding: 10px;
		box-sizing: border-box;
		user-select: none;
	}

	.tray-menu {
		display: flex;
		flex-direction: column;
		padding: 4px;
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-lg);
		background: var(--bg-surface);
		box-shadow:
			0 8px 22px rgba(0, 0, 0, 0.14),
			0 1px 4px rgba(0, 0, 0, 0.08);
		outline: none;
		animation: tray-in var(--duration-fast) var(--ease-standard);
	}
	:global(.dark) .tray-menu {
		border-color: var(--border-strong);
		box-shadow:
			0 8px 22px rgba(0, 0, 0, 0.45),
			0 1px 4px rgba(0, 0, 0, 0.3);
	}
	@keyframes tray-in {
		from {
			opacity: 0;
			transform: translateY(3px);
		}
	}
	:global(.animations-disabled) .tray-menu {
		animation: none;
	}

	.tray-item {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 32px;
		padding: 0 10px;
		border: 0;
		border-radius: var(--radius-md);
		background: transparent;
		color: var(--text-primary);
		font: inherit;
		font-size: var(--text-sm);
		text-align: left;
		cursor: pointer;
	}
	.tray-item :global(svg.lucide) {
		color: var(--text-secondary);
	}
	.tray-item:hover:not(:disabled),
	.tray-item:focus-visible {
		background: var(--state-hover);
		outline: none;
	}
	.tray-item:active:not(:disabled) {
		background: var(--state-active);
	}
	.tray-item:disabled {
		cursor: default;
		color: var(--text-tertiary);
	}
	.tray-item:disabled :global(svg.lucide) {
		color: var(--text-tertiary);
	}
	.tray-label {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.tray-icon {
		display: inline-flex;
	}
	.tray-icon-spacer {
		width: 16px;
		flex-shrink: 0;
	}
	.tray-spin {
		animation: tray-spin 0.9s linear infinite;
	}
	@keyframes tray-spin {
		to {
			transform: rotate(360deg);
		}
	}
	.tray-ok :global(svg.lucide) {
		color: var(--success);
	}
	.tray-error :global(svg.lucide) {
		color: var(--danger);
	}

	.tray-divider {
		height: 1px;
		margin: 4px 6px;
		background: var(--border-subtle);
	}

	/* Same look as the app's Switch, smaller. */
	.tray-switch {
		position: relative;
		flex-shrink: 0;
		width: 28px;
		height: 16px;
		box-sizing: border-box;
		border: 1px solid var(--border-strong);
		border-radius: 999px;
		background: var(--state-active);
		transition: background-color var(--duration-fast) var(--ease-standard);
	}
	.tray-switch::after {
		content: '';
		position: absolute;
		top: 2px;
		left: 2px;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--text-secondary);
		transition: transform var(--duration-fast) var(--ease-standard);
	}
	.tray-switch.on {
		border-color: transparent;
		background: var(--accent);
	}
	.tray-switch.on::after {
		transform: translateX(12px);
		background: var(--on-accent);
	}

	.tray-quit:hover,
	.tray-quit:focus-visible {
		background: var(--danger-soft);
		color: var(--danger);
	}
</style>
