<!--
	Settings (DOIT.md §8) — Raycast-style: section list on the left (168px),
	content capped at 680px. Changes apply immediately and save automatically.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { beforeNavigate } from '$app/navigation';
	import { Check, CircleAlert, Database, Info, Palette, Settings2, Gamepad2 } from '@lucide/svelte';
	import {
		DEFAULT_SETTINGS,
		getCachedSettings,
		loadSettingsFromStorage,
		mergeSettings,
		saveSettingsToStorage,
		type AppSettings
	} from '$lib/settings';
	import { applyAnimations, applyLanguage, applyTheme, broadcastSettings } from '$lib/preferences';
	import { t } from '$lib/i18n.svelte';
	import { Icon } from '$lib/components/ui';
	import GeneralSection from '$lib/components/settings/GeneralSection.svelte';
	import AppearanceSection from '$lib/components/settings/AppearanceSection.svelte';
	import SteamSection from '$lib/components/settings/SteamSection.svelte';
	import DataSection from '$lib/components/settings/DataSection.svelte';
	import AboutSection from '$lib/components/settings/AboutSection.svelte';

	type Section = 'general' | 'appearance' | 'steam' | 'data' | 'about';
	const SECTION_KEY = 'velaris_settings_section_v1';
	const SAVE_DELAY_MS = 450;

	const initial = getCachedSettings();
	let settings = $state<AppSettings>(initial ?? { ...DEFAULT_SETTINGS });
	let loaded = $state(initial !== null);
	let lastSaved = initial ? JSON.stringify(mergeSettings(initial)) : '';
	let saveState = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
	let saveError = $state('');
	let section = $state<Section>('general');
	let saveTimer: number | null = null;
	let savedTimer: number | null = null;

	// What has already been applied to the running app, so typing in a text field
	// doesn't re-dispatch theme/language events on every keystroke.
	let applied = { theme: '', language: '', animations: null as boolean | null };

	const sections = $derived([
		{ key: 'general' as Section, label: t('通用', 'General'), icon: Settings2 },
		{ key: 'appearance' as Section, label: t('外观', 'Appearance'), icon: Palette },
		{ key: 'steam' as Section, label: 'Steam', icon: Gamepad2 },
		{ key: 'data' as Section, label: t('数据', 'Data'), icon: Database },
		{ key: 'about' as Section, label: t('关于', 'About'), icon: Info }
	]);
	const current = $derived(sections.find((item) => item.key === section) ?? sections[0]);

	function applyLive(next: AppSettings) {
		if (applied.theme !== next.themeMode) {
			applied.theme = next.themeMode;
			applyTheme(next.themeMode);
		}
		if (applied.language !== next.language) {
			applied.language = next.language;
			applyLanguage(next.language);
		}
		if (applied.animations !== next.animationsEnabled) {
			applied.animations = next.animationsEnabled;
			applyAnimations(next.animationsEnabled);
		}
	}

	async function save() {
		if (saveTimer !== null) {
			window.clearTimeout(saveTimer);
			saveTimer = null;
		}
		const snapshot = JSON.stringify(mergeSettings(settings));
		if (snapshot === lastSaved) return;
		saveState = 'saving';
		try {
			const saved = await saveSettingsToStorage(mergeSettings(settings));
			lastSaved = JSON.stringify(saved);
			broadcastSettings(saved);
			saveState = 'saved';
			saveError = '';
			if (savedTimer !== null) window.clearTimeout(savedTimer);
			savedTimer = window.setTimeout(() => (saveState = 'idle'), 2000);
		} catch (error) {
			saveState = 'error';
			saveError = error instanceof Error ? error.message : String(error);
		}
	}

	/** Save now if a change is waiting (before a sync, navigation, or reset). */
	async function flush() {
		if (saveTimer !== null) await save();
	}

	async function resetSettings() {
		settings = { ...DEFAULT_SETTINGS };
		await save();
	}

	$effect(() => {
		if (!loaded) return;
		const snapshot = JSON.stringify(mergeSettings(settings));
		applyLive(settings);
		if (snapshot === lastSaved) return;
		if (saveTimer !== null) window.clearTimeout(saveTimer);
		saveTimer = window.setTimeout(() => void save(), SAVE_DELAY_MS);
	});

	function selectSection(next: Section) {
		section = next;
		try {
			sessionStorage.setItem(SECTION_KEY, next);
		} catch {
			// Ignore.
		}
	}

	beforeNavigate(() => {
		void flush();
	});

	onMount(() => {
		try {
			const stored = sessionStorage.getItem(SECTION_KEY) as Section | null;
			if (stored && ['general', 'appearance', 'steam', 'data', 'about'].includes(stored)) {
				section = stored;
			}
		} catch {
			// Ignore.
		}

		void (async () => {
			const fresh = await loadSettingsFromStorage();
			settings = fresh;
			lastSaved = JSON.stringify(mergeSettings(fresh));
			applied = {
				theme: fresh.themeMode,
				language: fresh.language,
				animations: fresh.animationsEnabled
			};
			loaded = true;
		})();

		// "Launch at startup" can also be toggled from the tray menu. Take that value
		// without discarding edits that are still waiting to be saved.
		const onExternalSettings = (event: Event) => {
			const next = (event as CustomEvent<AppSettings>).detail;
			settings.launchOnStartup = next.launchOnStartup;
			if (lastSaved) {
				lastSaved = JSON.stringify(
					mergeSettings({ ...JSON.parse(lastSaved), launchOnStartup: next.launchOnStartup })
				);
			}
		};
		window.addEventListener('pt-settings-external', onExternalSettings);

		// Follow the OS theme live while "System" is selected.
		const media = window.matchMedia('(prefers-color-scheme: dark)');
		const onSystemTheme = () => settings.themeMode === 'system' && applyTheme('system');
		media.addEventListener('change', onSystemTheme);

		return () => {
			window.removeEventListener('pt-settings-external', onExternalSettings);
			media.removeEventListener('change', onSystemTheme);
			void flush();
			if (savedTimer !== null) window.clearTimeout(savedTimer);
		};
	});
</script>

<div class="app-page h-full overflow-y-auto">
	<div class="app-page-inner">
		<header class="page-header mb-5!">
			<h1 class="page-title">{t('设置', 'Settings')}</h1>
			<span class="save-state" aria-live="polite">
				{#if saveState === 'saving'}
					{t('正在保存…', 'Saving…')}
				{:else if saveState === 'saved'}
					<Icon icon={Check} size="status" />{t('已保存', 'Saved')}
				{:else if saveState === 'error'}
					<span class="inline-flex items-center gap-1 text-(--danger)">
						<Icon icon={CircleAlert} size="status" />{t('保存失败', 'Couldn’t save')}
					</span>
				{/if}
			</span>
		</header>

		<div class="settings">
			<nav class="settings-nav" aria-label={t('设置分类', 'Settings sections')}>
				{#each sections as item (item.key)}
					<button
						type="button"
						class="nav-item"
						class:is-active={section === item.key}
						aria-current={section === item.key ? 'page' : undefined}
						onclick={() => selectSection(item.key)}
					>
						<Icon icon={item.icon} size="button" class="nav-icon" />
						{item.label}
					</button>
				{/each}
			</nav>

			<div class="settings-content">
				<h2 class="section-title">{current.label}</h2>
				{#if saveState === 'error'}
					<p class="save-error" role="alert">{saveError}</p>
				{/if}
				{#if !loaded}
					<div class="loading" aria-busy="true"></div>
				{:else if section === 'general'}
					<GeneralSection bind:settings />
				{:else if section === 'appearance'}
					<AppearanceSection bind:settings />
				{:else if section === 'steam'}
					<SteamSection bind:settings {flush} />
				{:else if section === 'data'}
					<DataSection onResetSettings={resetSettings} />
				{:else}
					<AboutSection />
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.save-state {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}

	.settings {
		display: flex;
		align-items: flex-start;
		gap: var(--space-8);
		padding-bottom: var(--space-8);
	}
	.settings-nav {
		position: sticky;
		top: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
		width: 168px;
		flex-shrink: 0;
	}
	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 34px;
		padding: 0 10px;
		border-radius: var(--radius-md);
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-secondary);
		text-align: left;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			color var(--duration-fast) var(--ease-standard);
	}
	.nav-item :global(.nav-icon) {
		color: var(--text-tertiary);
	}
	.nav-item:hover {
		background: var(--state-hover);
		color: var(--text-primary);
	}
	.nav-item.is-active {
		background: var(--state-active);
		color: var(--text-primary);
	}
	.nav-item.is-active :global(.nav-icon) {
		color: var(--accent);
	}

	.settings-content {
		flex: 1;
		min-width: 0;
		max-width: var(--settings-max-width);
	}
	.section-title {
		margin-bottom: var(--space-4);
		font-size: var(--text-xl);
		font-weight: 600;
		color: var(--text-primary);
	}
	.save-error {
		margin-bottom: var(--space-4);
		font-size: var(--text-sm);
		color: var(--danger);
	}
	.loading {
		height: 240px;
	}

	@media (max-width: 860px) {
		.settings {
			flex-direction: column;
			gap: var(--space-4);
		}
		.settings-nav {
			position: static;
			flex-direction: row;
			flex-wrap: wrap;
			width: auto;
		}
	}
</style>
