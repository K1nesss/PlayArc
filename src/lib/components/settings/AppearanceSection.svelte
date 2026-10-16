<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Monitor, Moon, Sun } from '@lucide/svelte';
	import type { AppSettings, ThemeMode } from '$lib/settings';
	import { t } from '$lib/i18n.svelte';
	import { SegmentedControl, Switch } from '$lib/components/ui';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingsRow from './SettingsRow.svelte';

	let { settings = $bindable() }: { settings: AppSettings } = $props();

	let micaSupported = $state<boolean | null>(null);

	onMount(() => {
		invoke<boolean>('window_supports_mica')
			.then((value) => (micaSupported = value))
			.catch(() => (micaSupported = false));
	});

	const themeOptions = $derived([
		{ value: 'system' as ThemeMode, label: t('跟随系统', 'System'), icon: Monitor },
		{ value: 'light' as ThemeMode, label: t('浅色', 'Light'), icon: Sun },
		{ value: 'dark' as ThemeMode, label: t('深色', 'Dark'), icon: Moon }
	]);
</script>

<SettingsGroup>
	<SettingsRow label={t('主题', 'Theme')}>
		<SegmentedControl
			bind:value={settings.themeMode}
			options={themeOptions}
			ariaLabel={t('主题', 'Theme')}
		/>
	</SettingsRow>
	<SettingsRow
		label={t('侧边栏透明效果', 'Sidebar transparency')}
		description={micaSupported === false
			? t(
					'需要 Windows 11，当前系统使用纯色侧边栏',
					'Requires Windows 11 — a solid sidebar is used on this system'
				)
			: t(
					'在侧边栏和标题栏使用 Windows 11 云母材质',
					'Use Windows 11 Mica behind the sidebar and title bar'
				)}
	>
		<Switch
			bind:checked={settings.sidebarTransparency}
			label={t('侧边栏透明效果', 'Sidebar transparency')}
			disabled={micaSupported === false}
		/>
	</SettingsRow>
	<SettingsRow
		label={t('紧凑游戏列表', 'Compact game list')}
		description={t('游戏库默认使用列表视图', 'Open the Games page in list view')}
	>
		<Switch
			bind:checked={settings.compactGameList}
			label={t('紧凑游戏列表', 'Compact game list')}
			onchange={() => {
				// The Games page remembers the last view per device; let this setting win again.
				try {
					localStorage.removeItem('velaris_games_view_v1');
				} catch {
					// Ignore.
				}
			}}
		/>
	</SettingsRow>
	<SettingsRow
		label={t('动画', 'Animations')}
		description={t('界面过渡与图表动画', 'Transitions and chart animations')}
	>
		<Switch bind:checked={settings.animationsEnabled} label={t('动画', 'Animations')} />
	</SettingsRow>
</SettingsGroup>
