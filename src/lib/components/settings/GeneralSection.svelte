<script lang="ts">
	import type { AppSettings } from '$lib/settings';
	import { t } from '$lib/i18n.svelte';
	import { Select, Switch } from '$lib/components/ui';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingsRow from './SettingsRow.svelte';

	let { settings = $bindable() }: { settings: AppSettings } = $props();

	const seconds = (values: number[]) =>
		values.map((value) => ({ value, label: t(`${value} 秒`, `${value} sec`) }));

	const rangeOptions = $derived([
		{ value: 7, label: t('7 天', '7 days') },
		{ value: 30, label: t('30 天', '30 days') },
		{ value: 90, label: t('90 天', '90 days') },
		{ value: 365, label: t('1 年', '1 year') },
		{ value: 0, label: t('全部', 'All time') }
	]);
</script>

<SettingsGroup>
	<SettingsRow label={t('语言', 'Language')}>
		<Select
			bind:value={settings.language}
			ariaLabel={t('语言', 'Language')}
			options={[
				{ value: 'zh-CN', label: '简体中文' },
				{ value: 'en-US', label: 'English' }
			]}
			class="w-40"
		/>
	</SettingsRow>
	<SettingsRow
		label={t('开机启动', 'Launch at startup')}
		description={t('登录 Windows 时自动启动 PlayArc', 'Start PlayArc when you sign in to Windows')}
	>
		<Switch bind:checked={settings.launchOnStartup} label={t('开机启动', 'Launch at startup')} />
	</SettingsRow>
	<SettingsRow
		label={t('关闭时最小化到托盘', 'Keep running in the tray')}
		description={t(
			'关闭窗口后继续在后台记录游玩时长',
			'Closing the window keeps tracking in the background'
		)}
	>
		<Switch
			bind:checked={settings.minimizeToTray}
			label={t('关闭时最小化到托盘', 'Keep running in the tray')}
		/>
	</SettingsRow>
	<SettingsRow
		label={t('托盘菜单样式', 'Tray menu style')}
		description={t('右键单击托盘图标时打开的菜单', 'What opens when you right-click the tray icon')}
	>
		<Select
			bind:value={settings.trayStyle}
			ariaLabel={t('托盘菜单样式', 'Tray menu style')}
			options={[
				{ value: 'panel', label: t('PlayArc 面板', 'PlayArc panel') },
				{ value: 'native', label: t('系统菜单', 'System menu') }
			]}
			class="w-40"
		/>
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('记录', 'Tracking')}>
	<SettingsRow
		label={t('空闲时扫描间隔', 'Scan interval when idle')}
		description={t('多久检查一次是否有游戏启动', 'How often to check whether a game has started')}
	>
		<Select
			bind:value={settings.idleScanIntervalSeconds}
			ariaLabel={t('空闲时扫描间隔', 'Scan interval when idle')}
			options={seconds([5, 10, 15, 30])}
			class="w-28"
		/>
	</SettingsRow>
	<SettingsRow
		label={t('游玩中扫描间隔', 'Scan interval while playing')}
		description={t('多久检查一次游戏是否已退出', 'How often to check whether the game has exited')}
	>
		<Select
			bind:value={settings.activeScanIntervalSeconds}
			ariaLabel={t('游玩中扫描间隔', 'Scan interval while playing')}
			options={seconds([2, 3, 5, 10])}
			class="w-28"
		/>
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('页面', 'Pages')}>
	<SettingsRow
		label={t('首页刷新间隔', 'Home refresh interval')}
		description={t('首页“正在游玩”状态的更新频率', 'How often Home checks what you’re playing')}
	>
		<Select
			bind:value={settings.dashboardRefreshSeconds}
			ariaLabel={t('首页刷新间隔', 'Home refresh interval')}
			options={seconds([2, 5, 10, 30])}
			class="w-28"
		/>
	</SettingsRow>
	<SettingsRow label={t('分析默认范围', 'Default Analysis range')}>
		<Select
			bind:value={settings.analyticsDefaultRange}
			ariaLabel={t('分析默认范围', 'Default Analysis range')}
			options={rangeOptions}
			class="w-28"
		/>
	</SettingsRow>
</SettingsGroup>
