<script lang="ts">
	import { onMount } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Download, RefreshCw } from '@lucide/svelte';
	import { checkForAppUpdate, formatUpdateNotes, type AppUpdate } from '$lib/updater';
	import { t } from '$lib/i18n.svelte';
	import { Button, InlineNotice, Modal, toast } from '$lib/components/ui';
	import AppIcon from '$lib/assets/favicon.svg?raw';
	import SettingsGroup from './SettingsGroup.svelte';
	import SettingsRow from './SettingsRow.svelte';

	let version = $state('');
	let checking = $state(false);
	let checkError = $state('');
	let update = $state<AppUpdate | null>(null);
	let dialogOpen = $state(false);
	let installing = $state(false);
	let installError = $state('');
	let downloaded = $state(0);
	let contentLength = $state(0);

	const progress = $derived(contentLength > 0 ? Math.min(1, downloaded / contentLength) : 0);
	const notes = $derived(formatUpdateNotes(update?.body));

	onMount(() => {
		getVersion()
			.then((value) => (version = value))
			.catch(() => (version = ''));
	});

	async function check() {
		if (checking || installing) return;
		checking = true;
		checkError = '';
		try {
			const next = await checkForAppUpdate();
			if (!next) {
				toast.info(t('已是最新版本', 'You’re on the latest version'));
				return;
			}
			update = next;
			dialogOpen = true;
		} catch (error) {
			checkError = error instanceof Error ? error.message : String(error);
		} finally {
			checking = false;
		}
	}

	async function install() {
		if (!update || installing) return;
		installing = true;
		installError = '';
		downloaded = 0;
		contentLength = 0;
		try {
			await update.downloadAndInstall((event) => {
				if (event.event === 'Started') contentLength = event.data.contentLength ?? 0;
				else if (event.event === 'Progress') downloaded += event.data.chunkLength;
			});
			await relaunch();
		} catch (error) {
			installError = error instanceof Error ? error.message : String(error);
			installing = false;
		}
	}
</script>

<div class="about-head">
	<span class="about-icon" aria-hidden="true">
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		{@html AppIcon}
	</span>
	<div>
		<p class="about-name">PlayArc</p>
		<p class="about-version">
			{version ? t(`版本 ${version}`, `Version ${version}`) : ' '}
		</p>
	</div>
</div>

<SettingsGroup>
	<SettingsRow
		label={t('软件更新', 'Software update')}
		description={t('检查并安装新版本', 'Check for and install new versions')}
		error={checkError}
	>
		<Button icon={RefreshCw} loading={checking} onclick={check}
			>{t('检查更新', 'Check for updates')}</Button
		>
	</SettingsRow>
</SettingsGroup>

<p class="about-footnote">
	{t(
		'PlayArc 在本机记录游戏时长，不会上传你的游玩数据。',
		'PlayArc records playtime on this computer and never uploads your play data.'
	)}
</p>

<Modal
	open={dialogOpen}
	title={t(
		`PlayArc ${update?.version ?? ''} 可用`,
		`PlayArc ${update?.version ?? ''} is available`
	)}
	description={update
		? t(`当前版本 ${update.currentVersion}`, `You have ${update.currentVersion}`)
		: undefined}
	dismissible={!installing}
	onClose={() => !installing && (dialogOpen = false)}
>
	<div class="notes">
		{#if notes}
			<pre>{notes}</pre>
		{:else}
			<p class="notes-empty">{t('此版本没有更新说明。', 'No release notes for this version.')}</p>
		{/if}
	</div>
	{#if installing}
		<div
			class="progress"
			role="progressbar"
			aria-valuemin="0"
			aria-valuemax="100"
			aria-valuenow={Math.round(progress * 100)}
		>
			<span style:width={`${Math.round(progress * 100)}%`}></span>
		</div>
		<p class="progress-label">
			{progress > 0
				? t(
						`正在下载… ${Math.round(progress * 100)}%`,
						`Downloading… ${Math.round(progress * 100)}%`
					)
				: t('正在准备…', 'Preparing…')}
		</p>
	{/if}
	{#if installError}
		<InlineNotice tone="danger" class="mt-3">{installError}</InlineNotice>
	{/if}
	{#snippet footer()}
		<Button onclick={() => (dialogOpen = false)} disabled={installing}>{t('稍后', 'Later')}</Button>
		<Button variant="primary" icon={Download} loading={installing} onclick={install}>
			{t('安装并重启', 'Install and restart')}
		</Button>
	{/snippet}
</Modal>

<style>
	.about-head {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		margin-bottom: var(--space-6);
	}
	.about-icon {
		display: inline-flex;
		width: 48px;
		height: 48px;
	}
	.about-icon :global(svg) {
		width: 100%;
		height: 100%;
	}
	.about-name {
		font-size: var(--text-xl);
		font-weight: 600;
		color: var(--text-primary);
	}
	.about-version {
		font-size: var(--text-sm);
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}
	.about-footnote {
		margin-top: var(--space-6);
		padding: 0 var(--space-1);
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.notes {
		max-height: 260px;
		overflow-y: auto;
		padding: var(--space-3);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-subtle);
	}
	.notes pre {
		white-space: pre-wrap;
		font-family: inherit;
		font-size: var(--text-sm);
		color: var(--text-primary);
	}
	.notes-empty {
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.progress {
		height: 4px;
		margin-top: var(--space-4);
		border-radius: 2px;
		background: var(--state-active);
		overflow: hidden;
	}
	.progress span {
		display: block;
		height: 100%;
		background: var(--accent);
		transition: width var(--duration-normal) var(--ease-standard);
	}
	.progress-label {
		margin-top: var(--space-2);
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
</style>
