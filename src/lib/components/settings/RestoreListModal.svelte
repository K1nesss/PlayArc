<!-- Lists hidden or ignored games with a Restore action per row. -->
<script lang="ts" module>
	export type RestoreItem = { key: number; name: string; at: number; detail?: string };
</script>

<script lang="ts">
	import { RotateCcw } from '@lucide/svelte';
	import { Button, EmptyState, Modal, Skeleton } from '$lib/components/ui';
	import { formatRelativeDay } from '$lib/format';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		open: boolean;
		title: string;
		description: string;
		emptyTitle: string;
		items: RestoreItem[] | null;
		restoring: number | null;
		onRestore: (item: RestoreItem) => void;
		onClose: () => void;
	}

	let { open, title, description, emptyTitle, items, restoring, onRestore, onClose }: Props =
		$props();
</script>

<Modal {open} {title} {description} size="md" {onClose}>
	{#if items === null}
		<div class="flex flex-col gap-3 pb-2">
			<Skeleton height="14px" width="70%" />
			<Skeleton height="14px" width="50%" />
		</div>
	{:else if items.length === 0}
		<EmptyState compact title={emptyTitle} />
	{:else}
		<ul class="list">
			{#each items as item (item.key)}
				<li class="item">
					<div class="min-w-0 flex-1">
						<p class="name">{item.name}</p>
						<p class="meta">{item.detail ?? formatRelativeDay(item.at)}</p>
					</div>
					<Button
						size="sm"
						icon={RotateCcw}
						loading={restoring === item.key}
						disabled={restoring !== null && restoring !== item.key}
						onclick={() => onRestore(item)}
					>
						{t('恢复', 'Restore')}
					</Button>
				</li>
			{/each}
		</ul>
	{/if}
	{#snippet footer()}
		<Button onclick={onClose}>{t('完成', 'Done')}</Button>
	{/snippet}
</Modal>

<style>
	.list {
		max-height: 320px;
		overflow-y: auto;
		margin: 0 calc(-1 * var(--space-2));
	}
	.item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2);
		border-radius: var(--radius-md);
	}
	.item:hover {
		background: var(--state-hover);
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-base);
		color: var(--text-primary);
	}
	.meta {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
</style>
