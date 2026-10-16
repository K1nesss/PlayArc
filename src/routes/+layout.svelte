<script lang="ts">
	import './layout.css';
	import Sidebar from '$lib/components/shell/Sidebar.svelte';
	import Titlebar from '$lib/components/shell/Titlebar.svelte';
	import CommandPalette from '$lib/components/shell/CommandPalette.svelte';
	import { initWindowState, setMaterialEnabled } from '$lib/shell/window.svelte';
	import { initSyncStatus, registerSyncHandler, syncStatus } from '$lib/shell/sync-status.svelte';
	import { syncSteamOwnedGameIcons } from '$lib/api';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/stores';
	import { checkForAppUpdate } from '$lib/updater';
	import { setLanguage, t } from '$lib/i18n.svelte';
	import Toaster from '$lib/components/ui/Toaster.svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import { ChevronUp } from '@lucide/svelte';
	import { toast } from '$lib/ui/toast.svelte';
	import {
		loadSettingsFromStorage,
		mergeSettings,
		saveSettingsToStorage,
		type AppLanguage,
		type ThemeMode
	} from '$lib/settings';
	let { children } = $props();

	// 状态管理
	let darkMode = $state(false);
	let language = $state<AppLanguage>('zh-CN');
	let animationsEnabled = $state(true);
	let mainElement = $state<HTMLElement | null>(null);
	let activeScrollElement: HTMLElement | null = null;
	let showBackToTop = $state(false);
	let scrollingBackToTop = false;
	let hideBackToTopTimer: number | null = null;
	const scrollPositions = new WeakMap<HTMLElement, number>();
	const GAMES_RESTORE_FROM_DETAIL_KEY = 'games_restore_from_detail_v1';
	const SIDEBAR_COLLAPSED_KEY = 'velaris_sidebar_collapsed_v1';

	// §33: below 1000px the sidebar is always compact; above, the user decides.
	// Collapsed by default; an explicit expand is remembered.
	let narrowWindow = $state(false);
	let userCollapsed = $state(true);
	const sidebarCollapsed = $derived(narrowWindow || userCollapsed);

	// §30: Ctrl+K palette, Ctrl+R refresh (re-mounts the page so it refetches).
	let paletteOpen = $state(false);
	let refreshKey = $state(0);
	const UPDATE_TOAST_STATE_KEY = 'velaris_update_toast_state_v1';

	function scheduleBackToTopHide() {
		if (hideBackToTopTimer !== null) {
			window.clearTimeout(hideBackToTopTimer);
		}

		hideBackToTopTimer = window.setTimeout(() => {
			showBackToTop = false;
			hideBackToTopTimer = null;
		}, 1800);
	}

	function handleMainScroll(event: Event) {
		if (!(event.target instanceof HTMLElement)) {
			return;
		}

		const target = event.target;
		const nextScrollTop = target.scrollTop;
		const previousScrollTop = scrollPositions.get(target) ?? nextScrollTop;
		const isScrollingUp = nextScrollTop < previousScrollTop;

		activeScrollElement = target;
		scrollPositions.set(target, nextScrollTop);

		if (scrollingBackToTop) {
			showBackToTop = false;
			if (nextScrollTop <= 4) {
				scrollingBackToTop = false;
			}
			return;
		}

		showBackToTop = isScrollingUp && nextScrollTop > 80;

		if (showBackToTop) {
			scheduleBackToTopHide();
		}
	}

	function scrollMainToTop() {
		const target = activeScrollElement ?? mainElement;
		if (!target) {
			return;
		}

		if (hideBackToTopTimer !== null) {
			window.clearTimeout(hideBackToTopTimer);
			hideBackToTopTimer = null;
		}

		scrollingBackToTop = true;
		showBackToTop = false;

		const active = document.activeElement;
		if (active instanceof HTMLElement) {
			active.blur();
		}

		target.scrollTo({
			top: 0,
			behavior: animationsEnabled ? 'smooth' : 'auto'
		});

		window.setTimeout(
			() => {
				if (target.scrollTop <= 4) {
					scrollingBackToTop = false;
				}
			},
			animationsEnabled ? 700 : 0
		);
	}

	function applyAnimationPreference(enabled: boolean) {
		animationsEnabled = enabled;
		document.documentElement.classList.toggle('animations-disabled', !enabled);
	}

	function todayKey() {
		return new Date().toISOString().slice(0, 10);
	}

	function hasShownUpdateToastToday(version: string) {
		try {
			const raw = localStorage.getItem(UPDATE_TOAST_STATE_KEY);
			if (!raw) {
				return false;
			}
			const parsed = JSON.parse(raw) as { date?: string; version?: string };
			return parsed.date === todayKey() && parsed.version === version;
		} catch {
			return false;
		}
	}

	function markUpdateToastShown(version: string) {
		try {
			localStorage.setItem(
				UPDATE_TOAST_STATE_KEY,
				JSON.stringify({
					date: todayKey(),
					version
				})
			);
		} catch {
			// Ignore update reminder storage failures.
		}
	}

	function showUpdateToast(version: string) {
		markUpdateToastShown(version);
		toast.info({
			title: t(`发现新版本 ${version}`, `Version ${version} is available`),
			description: t('可前往设置页查看更新内容', 'Open Settings to review the update'),
			action: { label: t('去设置', 'Settings'), onClick: goToSettingsForUpdate },
			duration: 8000
		});
	}

	async function checkUpdateReminder() {
		try {
			const update = await checkForAppUpdate();
			if (!update || hasShownUpdateToastToday(update.version)) {
				return;
			}
			showUpdateToast(update.version);
			await update.close();
		} catch {
			// Automatic update checks should stay silent.
		}
	}

	function goToSettingsForUpdate() {
		void goto(resolve('/settings'));
	}

	// 主题切换函数
	function isDarkByTheme(themeMode: ThemeMode) {
		if (themeMode === 'dark') {
			return true;
		}
		if (themeMode === 'light') {
			return false;
		}
		return window.matchMedia('(prefers-color-scheme: dark)').matches;
	}

	function applyResolvedTheme(shouldUseDark: boolean) {
		darkMode = shouldUseDark;
		document.documentElement.classList.toggle('dark', darkMode);
		localStorage.setItem('theme', darkMode ? 'dark' : 'light');
	}

	async function saveThemeMode(themeMode: ThemeMode) {
		const currentSettings = await loadSettingsFromStorage();
		const nextSettings = mergeSettings({
			...currentSettings,
			themeMode
		});

		await saveSettingsToStorage(nextSettings);
	}

	function toggleTheme() {
		const nextThemeMode: ThemeMode = darkMode ? 'light' : 'dark';

		void saveThemeMode(nextThemeMode);
		applyResolvedTheme(nextThemeMode === 'dark');
		window.dispatchEvent(
			new CustomEvent('pt-theme-change', {
				detail: { darkMode }
			})
		);
	}

	function handleSidebarNavigate() {
		try {
			sessionStorage.removeItem(GAMES_RESTORE_FROM_DETAIL_KEY);
		} catch {
			// Ignore navigation state cleanup failures.
		}
		window.requestAnimationFrame(() => mainElement?.scrollTo({ top: 0 }));
	}

	function toggleSidebar() {
		userCollapsed = !userCollapsed;
		try {
			localStorage.setItem(SIDEBAR_COLLAPSED_KEY, userCollapsed ? '1' : '0');
		} catch {
			// Per-device convenience only.
		}
	}

	async function refreshSteamConfigured() {
		const settings = await loadSettingsFromStorage();
		syncStatus.configured = Boolean(settings.apiKey && settings.steam64Id);
		return settings;
	}

	// §8 Steam → Auto Sync. One scheduler for the whole app (pages no longer sync on load).
	const AUTO_SYNC_STARTUP_DELAY_MS = 15_000;
	const AUTO_SYNC_CHECK_MS = 5 * 60_000;

	async function autoSyncIfDue() {
		const settings = await refreshSteamConfigured();
		if (!settings.steamAutoSync || !syncStatus.configured || syncStatus.state === 'syncing') {
			return;
		}
		const intervalSeconds = settings.steamSyncIntervalMinutes * 60;
		const last = syncStatus.lastSyncedAt ?? 0;
		if (Date.now() / 1000 - last < intervalSeconds) return;
		try {
			const summary = await syncSteamOwnedGameIcons(
				settings.apiKey,
				settings.steam64Id,
				'auto_sync'
			);
			// Only re-render the page when new artwork actually arrived.
			if (summary.updated_games > 0) refreshKey += 1;
		} catch {
			// Background failures stay quiet; the sidebar footer shows "Sync failed".
		}
	}

	async function syncSteamNow() {
		const settings = await refreshSteamConfigured();
		if (!syncStatus.configured) {
			toast.info({
				title: t('尚未连接 Steam', 'Steam isn’t connected'),
				description: t(
					'请在设置中填写 API Key 和 Steam ID',
					'Add your API key and Steam ID in Settings'
				),
				action: { label: t('去设置', 'Settings'), onClick: () => void goto(resolve('/settings')) }
			});
			return;
		}
		if (syncStatus.state === 'syncing') return;
		try {
			const summary = await syncSteamOwnedGameIcons(
				settings.apiKey,
				settings.steam64Id,
				'shortcut_refresh'
			);
			if (summary.skipped_reason !== 'sync_already_running') {
				toast.success(t('同步完成', 'Sync complete'));
				refreshKey += 1;
			}
		} catch {
			toast.error({
				title: t('同步失败', 'Sync failed'),
				description: syncStatus.error,
				action: { label: t('去设置', 'Settings'), onClick: () => void goto(resolve('/settings')) }
			});
		}
	}

	function refreshPage() {
		refreshKey += 1;
		if (syncStatus.configured) void syncSteamNow();
	}

	function handleGlobalKeydown(event: KeyboardEvent) {
		const mod = event.ctrlKey || event.metaKey;
		const key = event.key.toLowerCase();

		if (mod && key === 'k') {
			event.preventDefault();
			paletteOpen = !paletteOpen;
		} else if (mod && event.key === ',') {
			event.preventDefault();
			void goto(resolve('/settings'));
		} else if ((mod && key === 'r') || event.key === 'F5') {
			// Never let WebView2 reload the whole app.
			event.preventDefault();
			refreshPage();
		} else if (mod && key === 'p') {
			event.preventDefault(); // No print dialog in a desktop app.
		} else if (mod && key === 'b' && !event.shiftKey && !event.altKey) {
			// Ctrl+B toggles the sidebar (it is always compact in narrow windows).
			event.preventDefault();
			if (!narrowWindow) toggleSidebar();
		}
	}

	onMount(() => {
		void (async () => {
			const appSettings = await loadSettingsFromStorage();
			language = appSettings.language;
			setLanguage(language);
			applyAnimationPreference(appSettings.animationsEnabled);
			applyResolvedTheme(isDarkByTheme(appSettings.themeMode));
			initSyncStatus(Boolean(appSettings.apiKey && appSettings.steam64Id));
			setMaterialEnabled(appSettings.sidebarTransparency);
		})();

		const autoSyncStart = window.setTimeout(() => void autoSyncIfDue(), AUTO_SYNC_STARTUP_DELAY_MS);
		const autoSyncTimer = window.setInterval(() => void autoSyncIfDue(), AUTO_SYNC_CHECK_MS);

		// Settings page broadcasts saved settings so the shell can follow along.
		const handleSettingsChange = (event: Event) => {
			const detail = (event as CustomEvent<{ sidebarTransparency?: boolean }>).detail;
			if (typeof detail?.sidebarTransparency === 'boolean') {
				setMaterialEnabled(detail.sidebarTransparency);
			}
			void refreshSteamConfigured();
		};
		window.addEventListener('pt-settings-change', handleSettingsChange);

		const disposeWindowState = initWindowState();
		const disposeSyncHandler = registerSyncHandler(syncSteamNow);

		try {
			userCollapsed = localStorage.getItem(SIDEBAR_COLLAPSED_KEY) !== '0';
		} catch {
			userCollapsed = true;
		}
		const narrowQuery = window.matchMedia('(max-width: 999.98px)');
		const handleNarrow = () => (narrowWindow = narrowQuery.matches);
		handleNarrow();
		narrowQuery.addEventListener('change', handleNarrow);

		window.setTimeout(() => {
			void checkUpdateReminder();
		}, 3500);

		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		const handleSystemThemeChange = () => {
			void loadSettingsFromStorage().then((settings) => {
				if (settings.themeMode === 'system') {
					applyResolvedTheme(mediaQuery.matches);
				}
			});
		};

		const handleThemeChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { darkMode?: boolean } | undefined)
					: undefined;
			if (detail && typeof detail.darkMode === 'boolean') {
				darkMode = detail.darkMode;
			}
		};

		mediaQuery.addEventListener('change', handleSystemThemeChange);
		window.addEventListener('pt-theme-change', handleThemeChange);

		const handleLanguageChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { language?: AppLanguage } | undefined)
					: undefined;
			if (detail && (detail.language === 'zh-CN' || detail.language === 'en-US')) {
				language = detail.language;
				setLanguage(language);
			}
		};

		window.addEventListener('pt-language-change', handleLanguageChange);

		const preventNativeContextMenu = (event: MouseEvent) => {
			event.preventDefault();
		};

		window.addEventListener('contextmenu', preventNativeContextMenu);

		const handleAnimationChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { animationsEnabled?: boolean } | undefined)
					: undefined;
			if (detail && typeof detail.animationsEnabled === 'boolean') {
				applyAnimationPreference(detail.animationsEnabled);
			}
		};

		window.addEventListener('pt-animations-change', handleAnimationChange);

		return () => {
			disposeWindowState();
			disposeSyncHandler();
			window.clearTimeout(autoSyncStart);
			window.clearInterval(autoSyncTimer);
			window.removeEventListener('pt-settings-change', handleSettingsChange);
			narrowQuery.removeEventListener('change', handleNarrow);
			if (hideBackToTopTimer !== null) {
				window.clearTimeout(hideBackToTopTimer);
			}
			mediaQuery.removeEventListener('change', handleSystemThemeChange);
			window.removeEventListener('pt-theme-change', handleThemeChange);
			window.removeEventListener('pt-language-change', handleLanguageChange);
			window.removeEventListener('contextmenu', preventNativeContextMenu);
			window.removeEventListener('pt-animations-change', handleAnimationChange);
		};
	});

	$effect(() => {
		void $page.url.pathname;
		void refreshSteamConfigured().catch(() => {});
		activeScrollElement = null;
		scrollingBackToTop = false;
		showBackToTop = false;
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="app-shell">
	<Sidebar
		pathname={$page.url.pathname}
		collapsed={sidebarCollapsed}
		forcedCollapsed={narrowWindow}
		onToggleCollapse={toggleSidebar}
		onNavigate={handleSidebarNavigate}
		onSync={syncSteamNow}
		onOpenSettings={() => void goto(resolve('/settings'))}
	/>

	<div class="shell-column">
		<Titlebar />
		<main bind:this={mainElement} onscrollcapture={handleMainScroll} class="shell-content">
			{#key refreshKey}
				{@render children()}
			{/key}
		</main>
	</div>

	<button
		type="button"
		class={`back-to-top-button fixed right-6 bottom-6 z-40 flex h-9 w-9 items-center justify-center rounded-lg border border-line-strong bg-elevated text-fg-muted shadow-(--elevation-2) backdrop-blur-md hover:text-fg ${
			showBackToTop
				? 'pointer-events-auto translate-y-0 opacity-100'
				: 'pointer-events-none translate-y-2 opacity-0'
		}`}
		title={language === 'zh-CN' ? '返回顶部' : 'Back to top'}
		aria-label={language === 'zh-CN' ? '返回顶部' : 'Back to top'}
		onclick={scrollMainToTop}
	>
		<Icon icon={ChevronUp} size="button" />
	</button>

	<CommandPalette
		open={paletteOpen}
		{darkMode}
		onClose={() => (paletteOpen = false)}
		onSync={syncSteamNow}
		onToggleTheme={toggleTheme}
	/>
	<Toaster />
</div>
