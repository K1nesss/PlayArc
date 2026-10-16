<!--
	Add a non-Steam game by its executable. Name is inferred from the .exe;
	artwork is optional. Errors stay inline next to the form (§28).
-->
<script lang="ts">
	import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
	import { FolderOpen, Plus } from '@lucide/svelte';
	import { createManualGame } from '$lib/api';
	import { Button, InlineNotice, Input, Modal } from '$lib/components/ui';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
		onCreated: (gameId: number) => void;
	}

	let { open, onClose, onCreated }: Props = $props();

	type ArtKey = 'icon_path' | 'cover_path' | 'hero_path';

	let form = $state({
		name: '',
		executable_path: '',
		icon_path: '',
		cover_path: '',
		hero_path: ''
	});
	let saving = $state(false);
	let error = $state('');

	$effect(() => {
		if (open) {
			form = { name: '', executable_path: '', icon_path: '', cover_path: '', hero_path: '' };
			error = '';
		}
	});

	function firstPath(value: string | string[] | null) {
		if (typeof value === 'string') return value;
		return Array.isArray(value) && typeof value[0] === 'string' ? value[0] : null;
	}

	function nameFromPath(path: string) {
		const file = path.split(/[\\/]/).pop() ?? '';
		return file
			.replace(/\.[^.]+$/, '')
			.replace(/[_-]+/g, ' ')
			.trim();
	}

	async function chooseExecutable() {
		const path = firstPath(
			await openFileDialog({
				multiple: false,
				filters: [{ name: 'Executable', extensions: ['exe'] }]
			})
		);
		if (!path) return;
		form.executable_path = path;
		if (!form.name.trim()) form.name = nameFromPath(path);
	}

	async function chooseImage(key: ArtKey) {
		const extensions =
			key === 'icon_path' ? ['ico', 'png', 'jpg', 'jpeg', 'webp'] : ['png', 'jpg', 'jpeg', 'webp'];
		const path = firstPath(
			await openFileDialog({ multiple: false, filters: [{ name: 'Images', extensions }] })
		);
		if (path) form[key] = path;
	}

	async function submit(event?: SubmitEvent) {
		event?.preventDefault();
		if (saving) return;
		error = '';
		const name = form.name.trim();
		const executable = form.executable_path.trim();
		if (!name) {
			error = t('请填写游戏名称', 'Enter a game name');
			return;
		}
		if (!executable) {
			error = t('请选择游戏的 .exe 文件', 'Choose the game’s .exe file');
			return;
		}
		saving = true;
		try {
			const id = await createManualGame({
				name,
				executable_path: executable,
				icon_path: form.icon_path.trim() || null,
				cover_path: form.cover_path.trim() || null,
				hero_path: form.hero_path.trim() || null
			});
			onCreated(id);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			saving = false;
		}
	}

	const artFields = $derived<{ key: ArtKey; label: string; hint: string }[]>([
		{ key: 'cover_path', label: t('封面', 'Cover'), hint: t('竖版 2:3', 'Portrait, 2:3') },
		{ key: 'hero_path', label: t('背景图', 'Hero image'), hint: t('横版宽图', 'Wide banner') },
		{ key: 'icon_path', label: t('图标', 'Icon'), hint: t('方形', 'Square') }
	]);
</script>

<Modal
	{open}
	title={t('添加游戏', 'Add game')}
	description={t(
		'添加非 Steam 游戏。PlayArc 会在该程序运行时记录时长。',
		'Add a non-Steam game. PlayArc tracks it whenever this program runs.'
	)}
	dismissible={!saving}
	onClose={() => !saving && onClose()}
>
	<form id="add-game-form" class="form" onsubmit={submit}>
		<label class="field">
			<span class="field-label">{t('程序', 'Executable')}</span>
			<div class="file-row">
				<Input
					value={form.executable_path}
					placeholder="C:\Games\Game.exe"
					readonly
					class="flex-1"
				/>
				<Button icon={FolderOpen} onclick={chooseExecutable}>{t('浏览', 'Browse')}</Button>
			</div>
		</label>

		<label class="field">
			<span class="field-label">{t('名称', 'Name')}</span>
			<Input bind:value={form.name} placeholder={t('游戏名称', 'Game name')} data-autofocus />
		</label>

		<fieldset class="field">
			<legend class="field-label">{t('图片（可选）', 'Artwork (optional)')}</legend>
			<div class="art-list">
				{#each artFields as field (field.key)}
					<div class="art-row">
						<div class="min-w-0 flex-1">
							<p class="art-name">{field.label} <span class="art-hint">{field.hint}</span></p>
							<p class="art-path" title={form[field.key]}>
								{form[field.key] || t('未选择', 'Not selected')}
							</p>
						</div>
						<Button size="sm" variant="ghost" onclick={() => chooseImage(field.key)}>
							{t('选择…', 'Choose…')}
						</Button>
					</div>
				{/each}
			</div>
		</fieldset>

		{#if error}
			<InlineNotice tone="danger">{error}</InlineNotice>
		{/if}
	</form>

	{#snippet footer()}
		<Button onclick={onClose} disabled={saving}>{t('取消', 'Cancel')}</Button>
		<Button variant="primary" icon={Plus} type="submit" form="add-game-form" loading={saving}>
			{t('添加', 'Add game')}
		</Button>
	{/snippet}
</Modal>

<style>
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding-bottom: var(--space-1);
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.field-label {
		font-size: var(--text-sm);
		font-weight: 500;
		color: var(--text-primary);
	}
	.file-row {
		display: flex;
		gap: var(--space-2);
	}
	.art-list {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
	}
	.art-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-2) var(--space-2) var(--space-3);
	}
	.art-row + .art-row {
		border-top: 1px solid var(--border-subtle);
	}
	.art-name {
		font-size: var(--text-sm);
		color: var(--text-primary);
	}
	.art-hint {
		margin-left: 4px;
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.art-path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
</style>
