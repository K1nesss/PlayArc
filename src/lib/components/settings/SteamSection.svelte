<script lang="ts">
	import { Eye, EyeOff, ImageDown, RefreshCw } from '@lucide/svelte';
	import {
		getIgnoredGamesList,
		restoreIgnoredGame,
		syncAllSteamGameMedia,
		type IgnoredGameItem
	} from '$lib/api';
	import type { AppSettings } from '$lib/settings';
	import { formatTimeAgo } from '$lib/format';
	import { t } from '$lib/i18n.svelte';
	import { requestSteamSync, syncStatus } from '$lib/shell/sync-status.svelte';
	import { Button, IconButton, Input, Select, Switch, toast } from '$lib/components/ui';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingsRow from './SettingsRow.svelte';
	import RestoreListModal, { type RestoreItem } from './RestoreListModal.svelte';

	let {
		settings = $bindable(),
		flush
	}: {
		settings: AppSettings;
		/** Save pending edits now (so a key typed a moment ago is used by the sync). */
		flush: () => Promise<void>;
	} = $props();

	let showKey = $state(false);
	let refreshingArt = $state(false);
	let ignoredOpen = $state(false);
	let ignored = $state<IgnoredGameItem[] | null>(null);
	let restoring = $state<number | null>(null);

	// Light format checks — the definitive check is Steam answering the sync.
	const keyError = $derived(
		settings.apiKey && !/^[0-9a-f]{32}$/i.test(settings.apiKey)
			? t('API Key 应为 32 位十六进制字符', 'An API key is 32 hexadecimal characters')
			: ''
	);
	const idError = $derived(
		settings.steam64Id && !/^7656119\d{10}$/.test(settings.steam64Id)
			? t(
					'Steam ID 应为以 7656119 开头的 17 位数字',
					'A Steam ID is 17 digits starting with 7656119'
				)
			: ''
	);
	const configured = $derived(
		Boolean(settings.apiKey && settings.steam64Id && !keyError && !idError)
	);

	// §28: a failed sync (e.g. rejected key) is shown next to the credentials, not only in a toast.
	const syncError = $derived(
		syncStatus.state === 'error' && syncStatus.error
			? t(`上次同步失败：${syncStatus.error}`, `Last sync failed: ${syncStatus.error}`)
			: ''
	);

	const syncLine = $derived(
		syncStatus.state === 'syncing'
			? t('正在同步…', 'Syncing…')
			: syncStatus.lastSyncedAt
				? t(
						`上次同步：${formatTimeAgo(syncStatus.lastSyncedAt)}`,
						`Last synced ${formatTimeAgo(syncStatus.lastSyncedAt)}`
					)
				: t('尚未同步', 'Not synced yet')
	);

	const intervalOptions = $derived([
		{ value: 30, label: t('30 分钟', '30 minutes') },
		{ value: 60, label: t('1 小时', '1 hour') },
		{ value: 180, label: t('3 小时', '3 hours') },
		{ value: 360, label: t('6 小时', '6 hours') }
	]);

	const ignoredItems = $derived<RestoreItem[] | null>(
		ignored?.map((game) => ({ key: game.appid, name: game.name, at: game.ignored_at })) ?? null
	);

	async function openIgnored() {
		ignoredOpen = true;
		ignored = null;
		try {
			ignored = await getIgnoredGamesList();
		} catch (error) {
			ignored = [];
			toast.error({ title: t('无法加载', 'Couldn’t load the list'), description: String(error) });
		}
	}

	async function restore(item: RestoreItem) {
		restoring = item.key;
		try {
			await restoreIgnoredGame(item.key);
			ignored = ignored?.filter((game) => game.appid !== item.key) ?? null;
			toast.success(t(`已恢复“${item.name}”`, `Restored “${item.name}”`));
		} catch (error) {
			toast.error({ title: t('恢复失败', 'Couldn’t restore'), description: String(error) });
		} finally {
			restoring = null;
		}
	}

	async function refreshAllArtwork() {
		if (!configured || refreshingArt) return;
		refreshingArt = true;
		try {
			const summary = await syncAllSteamGameMedia(
				settings.apiKey,
				settings.steam64Id,
				'settings_refresh_artwork'
			);
			if (summary.skipped_reason === 'sync_already_running') {
				toast.info(
					t('已有同步正在进行，请稍后再试', 'A sync is already running — try again shortly')
				);
			} else {
				toast.success({
					title: t('图片已更新', 'Artwork refreshed'),
					description: t(
						`更新了 ${summary.updated_games} 个游戏`,
						`${summary.updated_games} games updated`
					)
				});
			}
		} catch {
			// Shown inline via syncError.
		} finally {
			refreshingArt = false;
		}
	}
</script>

<SettingsGroup
	title={t('Steam 账户', 'Steam account')}
	description={t(
		'用于同步游戏库封面和图片。密钥只保存在本机。',
		'Used to sync your library’s covers and artwork. Stored on this computer only.'
	)}
>
	<SettingsRow
		label="Steam Web API Key"
		description={t(
			'在 steamcommunity.com/dev/apikey 获取',
			'Get one at steamcommunity.com/dev/apikey'
		)}
		for="steam-api-key"
		error={keyError}
		stacked
	>
		<Input
			id="steam-api-key"
			bind:value={settings.apiKey}
			type={showKey ? 'text' : 'password'}
			placeholder="XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
			autocomplete="off"
			spellcheck={false}
			invalid={Boolean(keyError)}
			class="flex-1"
		/>
		<IconButton
			icon={showKey ? EyeOff : Eye}
			label={showKey ? t('隐藏密钥', 'Hide key') : t('显示密钥', 'Show key')}
			variant="secondary"
			onclick={() => (showKey = !showKey)}
		/>
	</SettingsRow>
	<SettingsRow
		label="Steam ID"
		description={t('你的 SteamID64，17 位数字', 'Your SteamID64 — 17 digits')}
		for="steam-id"
		error={idError || syncError}
		stacked
	>
		<Input
			id="steam-id"
			bind:value={settings.steam64Id}
			placeholder="7656119XXXXXXXXXX"
			inputmode="numeric"
			autocomplete="off"
			invalid={Boolean(idError)}
			class="flex-1"
		/>
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('同步', 'Sync')}>
	<SettingsRow label={t('立即同步', 'Sync now')} description={syncLine}>
		<Button
			icon={RefreshCw}
			loading={syncStatus.state === 'syncing'}
			disabled={!configured}
			onclick={async () => {
				await flush();
				requestSteamSync();
			}}
		>
			{t('同步', 'Sync')}
		</Button>
	</SettingsRow>
	<SettingsRow
		label={t('自动同步', 'Auto sync')}
		description={t(
			'在后台定期下载新游戏的封面和图片',
			'Periodically fetch artwork for new games in the background'
		)}
	>
		<Switch bind:checked={settings.steamAutoSync} label={t('自动同步', 'Auto sync')} />
	</SettingsRow>
	<SettingsRow label={t('同步间隔', 'Sync interval')}>
		<Select
			bind:value={settings.steamSyncIntervalMinutes}
			ariaLabel={t('同步间隔', 'Sync interval')}
			options={intervalOptions}
			disabled={!settings.steamAutoSync}
			class="w-32"
		/>
	</SettingsRow>
	<SettingsRow
		label={t('重新下载全部图片', 'Re-download all artwork')}
		description={t(
			'覆盖已缓存的封面、背景图和图标',
			'Replace every cached cover, hero image and icon'
		)}
	>
		<Button
			icon={ImageDown}
			loading={refreshingArt}
			disabled={!configured}
			onclick={refreshAllArtwork}
		>
			{t('重新下载', 'Re-download')}
		</Button>
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('扫描', 'Scanning')}>
	<SettingsRow
		label={t('已忽略的 Steam 项目', 'Ignored Steam items')}
		description={t(
			'这些项目（如运行库、工具）不会被添加到游戏库',
			'These items (redistributables, tools…) are never added to your library'
		)}
	>
		<Button onclick={openIgnored}>{t('管理…', 'Manage…')}</Button>
	</SettingsRow>
</SettingsGroup>

<RestoreListModal
	open={ignoredOpen}
	title={t('已忽略的 Steam 项目', 'Ignored Steam items')}
	description={t(
		'恢复后会重新扫描 Steam 并添加到游戏库。',
		'Restoring rescans Steam and adds the item to your library.'
	)}
	emptyTitle={t('没有已忽略的项目', 'Nothing is ignored')}
	items={ignoredItems}
	{restoring}
	onRestore={restore}
	onClose={() => (ignoredOpen = false)}
/>
