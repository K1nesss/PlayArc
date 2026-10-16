<!--
	Data (DOIT.md §8). No full-width red "danger zone": red is reserved for the
	individual destructive buttons.
-->
<script lang="ts">
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Download, FileText, RotateCcw, Trash2, Upload } from '@lucide/svelte';
	import {
		clearPlayHistory,
		exportDatabase,
		getHiddenGamesList,
		importDatabase,
		openAppLogFile,
		restoreHiddenGame,
		type HiddenGameItem
	} from '$lib/api';
	import { t } from '$lib/i18n.svelte';
	import { Button, InlineNotice, toast } from '$lib/components/ui';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingsRow from './SettingsRow.svelte';
	import RestoreListModal, { type RestoreItem } from './RestoreListModal.svelte';

	interface Props {
		onResetSettings: () => Promise<void>;
	}

	let { onResetSettings }: Props = $props();

	let exporting = $state(false);
	let importing = $state(false);
	let importDone = $state(false);
	let importError = $state('');
	let openingLog = $state(false);
	let fileInput = $state<HTMLInputElement | null>(null);

	let hiddenOpen = $state(false);
	let hidden = $state<HiddenGameItem[] | null>(null);
	let restoring = $state<number | null>(null);

	type Pending = 'clear' | 'reset' | null;
	let pending = $state<Pending>(null);
	let confirming = $state(false);

	const hiddenItems = $derived<RestoreItem[] | null>(
		hidden?.map((game) => ({ key: game.id, name: game.name, at: game.updated_at })) ?? null
	);

	function base64ToBlob(base64: string) {
		const binary = atob(base64);
		const bytes = new Uint8Array(binary.length);
		for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
		return new Blob([bytes], { type: 'application/octet-stream' });
	}

	function bufferToBase64(buffer: ArrayBuffer) {
		const bytes = new Uint8Array(buffer);
		let binary = '';
		for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i]);
		return btoa(binary);
	}

	async function exportData() {
		exporting = true;
		try {
			const payload = await exportDatabase();
			const url = URL.createObjectURL(base64ToBlob(payload));
			const link = document.createElement('a');
			const stamp = new Date().toISOString().slice(0, 10);
			link.href = url;
			link.download = `playarc-backup-${stamp}.db`;
			link.click();
			URL.revokeObjectURL(url);
			toast.success(t('已导出数据', 'Data exported'));
		} catch (error) {
			toast.error({ title: t('导出失败', 'Export failed'), description: String(error) });
		} finally {
			exporting = false;
		}
	}

	async function importData(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		importing = true;
		importError = '';
		try {
			await importDatabase(bufferToBase64(await file.arrayBuffer()));
			importDone = true;
		} catch (error) {
			importError = error instanceof Error ? error.message : String(error);
		} finally {
			importing = false;
			input.value = '';
		}
	}

	async function openLog() {
		openingLog = true;
		try {
			const path = await openAppLogFile();
			toast.success({ title: t('已打开日志文件', 'Opened log file'), description: path });
		} catch (error) {
			toast.error({
				title: t('无法打开日志', 'Couldn’t open the log'),
				description: String(error)
			});
		} finally {
			openingLog = false;
		}
	}

	async function openHidden() {
		hiddenOpen = true;
		hidden = null;
		try {
			hidden = await getHiddenGamesList();
		} catch (error) {
			hidden = [];
			toast.error({ title: t('无法加载', 'Couldn’t load the list'), description: String(error) });
		}
	}

	async function restoreHidden(item: RestoreItem) {
		restoring = item.key;
		try {
			await restoreHiddenGame(item.key);
			hidden = hidden?.filter((game) => game.id !== item.key) ?? null;
			toast.success(t(`已恢复“${item.name}”`, `Restored “${item.name}”`));
		} catch (error) {
			toast.error({ title: t('恢复失败', 'Couldn’t restore'), description: String(error) });
		} finally {
			restoring = null;
		}
	}

	async function runPending() {
		if (!pending || confirming) return;
		confirming = true;
		try {
			if (pending === 'clear') {
				const count = await clearPlayHistory();
				toast.success(t(`已删除 ${count} 条游玩记录`, `Deleted ${count} play sessions`));
			} else {
				await onResetSettings();
				toast.success(t('已恢复默认设置', 'Settings reset to defaults'));
			}
			pending = null;
		} catch (error) {
			pending = null;
			toast.error({ title: t('操作失败', 'Something went wrong'), description: String(error) });
		} finally {
			confirming = false;
		}
	}
</script>

<SettingsGroup title={t('备份', 'Backup')}>
	<SettingsRow
		label={t('导出数据', 'Export data')}
		description={t(
			'保存包含全部游玩记录的数据库副本',
			'Save a copy of the database with all your play history'
		)}
	>
		<Button icon={Download} loading={exporting} onclick={exportData}>{t('导出', 'Export')}</Button>
	</SettingsRow>
	<SettingsRow
		label={t('导入数据', 'Import data')}
		description={t(
			'用备份替换当前数据，导入后需要重启',
			'Replace current data with a backup; requires a restart'
		)}
		error={importError}
	>
		<input
			bind:this={fileInput}
			type="file"
			accept=".db,.sqlite,.sqlite3"
			class="hidden"
			onchange={importData}
		/>
		<Button icon={Upload} loading={importing} onclick={() => fileInput?.click()}
			>{t('导入…', 'Import…')}</Button
		>
		{#snippet extra()}
			{#if importDone}
				<InlineNotice tone="success" title={t('导入完成', 'Import complete')}>
					{t(
						'重启 PlayArc 以使用导入的数据。',
						'Restart PlayArc to start using the imported data.'
					)}
					{#snippet action()}
						<Button size="sm" variant="primary" icon={RotateCcw} onclick={() => relaunch()}>
							{t('立即重启', 'Restart now')}
						</Button>
					{/snippet}
				</InlineNotice>
			{/if}
		{/snippet}
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('游戏库', 'Library')}>
	<SettingsRow
		label={t('已隐藏的游戏', 'Hidden games')}
		description={t(
			'从游戏库隐藏的游戏，记录仍会保留',
			'Games hidden from your library — their history is kept'
		)}
	>
		<Button onclick={openHidden}>{t('管理…', 'Manage…')}</Button>
	</SettingsRow>
</SettingsGroup>

<SettingsGroup title={t('维护', 'Maintenance')}>
	<SettingsRow
		label={t('日志文件', 'Log file')}
		description={t('排查问题时使用', 'Useful when troubleshooting')}
	>
		<Button icon={FileText} loading={openingLog} onclick={openLog}>{t('打开', 'Open')}</Button>
	</SettingsRow>
	<SettingsRow
		label={t('恢复默认设置', 'Reset settings')}
		description={t(
			'恢复全部设置，游玩记录不受影响',
			'Restore every setting to its default; play history is untouched'
		)}
	>
		<Button onclick={() => (pending = 'reset')}>{t('恢复默认', 'Reset')}</Button>
	</SettingsRow>
	<SettingsRow
		label={t('清除本地历史', 'Clear local history')}
		description={t(
			'永久删除全部已结束的游玩记录，游戏库保留',
			'Permanently delete every finished session; your game library is kept'
		)}
	>
		<Button variant="danger" icon={Trash2} onclick={() => (pending = 'clear')}>
			{t('清除…', 'Clear…')}
		</Button>
	</SettingsRow>
</SettingsGroup>

<RestoreListModal
	open={hiddenOpen}
	title={t('已隐藏的游戏', 'Hidden games')}
	description={t('恢复后会重新出现在游戏库中。', 'Restored games reappear in your library.')}
	emptyTitle={t('没有隐藏的游戏', 'No hidden games')}
	items={hiddenItems}
	{restoring}
	onRestore={restoreHidden}
	onClose={() => (hiddenOpen = false)}
/>

<ConfirmDialog
	open={pending !== null}
	variant={pending === 'clear' ? 'danger' : 'default'}
	title={pending === 'clear'
		? t('清除全部游玩记录？', 'Clear all play history?')
		: t('恢复默认设置？', 'Reset all settings?')}
	message={pending === 'clear'
		? t(
				'所有已结束的会话将被永久删除，无法撤销。建议先导出备份。',
				'Every finished session will be permanently deleted. This can’t be undone — consider exporting a backup first.'
			)
		: t(
				'Steam 密钥和所有偏好都会恢复为默认值。',
				'Your Steam key and all preferences return to their defaults.'
			)}
	confirmText={pending === 'clear' ? t('清除记录', 'Clear history') : t('恢复默认', 'Reset')}
	cancelText={t('取消', 'Cancel')}
	loading={confirming}
	onConfirm={runPending}
	onCancel={() => !confirming && (pending = null)}
/>
