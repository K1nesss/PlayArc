<!-- Renders the toast stack (DOIT.md §28). Mounted once in the root layout. -->
<script lang="ts">
	import { CircleAlert, CircleCheck, Info, X } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import { dismissToast, toasts } from '$lib/ui/toast.svelte';
	import { t } from '$lib/i18n.svelte';

	const ICONS = { success: CircleCheck, info: Info, error: CircleAlert };
</script>

<div class="ui-toaster" role="region" aria-label={t('通知', 'Notifications')} aria-live="polite">
	{#each toasts as item (item.id)}
		<div
			class={`ui-toast ui-toast--${item.tone}`}
			role={item.tone === 'error' ? 'alert' : 'status'}
		>
			<span class="ui-toast-icon"><Icon icon={ICONS[item.tone]} size="button" /></span>
			<div class="min-w-0 flex-1">
				<p class="ui-toast-title">{item.title}</p>
				{#if item.description}<p class="ui-toast-description">{item.description}</p>{/if}
			</div>
			{#if item.action}
				<button
					type="button"
					class="ui-toast-action"
					onclick={() => {
						item.action?.onClick();
						dismissToast(item.id);
					}}
				>
					{item.action.label}
				</button>
			{/if}
			<button
				type="button"
				class="ui-toast-close"
				aria-label={t('关闭', 'Dismiss')}
				onclick={() => dismissToast(item.id)}
			>
				<Icon icon={X} size="status" />
			</button>
		</div>
	{/each}
</div>

<style>
	.ui-toaster {
		position: fixed;
		right: var(--space-5);
		bottom: var(--space-5);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: var(--space-2);
		pointer-events: none;
	}
	.ui-toast {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		width: min(360px, calc(100vw - 48px));
		padding: var(--space-3);
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-lg);
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(130%);
		box-shadow: var(--elevation-overlay);
		pointer-events: auto;
		animation: ui-toast-in var(--duration-normal) var(--ease-standard);
	}
	.ui-toast-icon {
		display: inline-flex;
		margin-top: 1px;
		color: var(--text-secondary);
	}
	.ui-toast--success .ui-toast-icon {
		color: var(--success);
	}
	.ui-toast--error .ui-toast-icon {
		color: var(--danger);
	}
	.ui-toast-title {
		font-size: var(--text-base);
		font-weight: 500;
		line-height: 18px;
		color: var(--text-primary);
	}
	.ui-toast-description {
		margin-top: 2px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.ui-toast-action {
		flex-shrink: 0;
		height: 24px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-sm);
		font-weight: 500;
		color: var(--accent);
	}
	.ui-toast-action:hover {
		background: var(--accent-soft);
	}
	.ui-toast-close {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		flex-shrink: 0;
		border-radius: var(--radius-sm);
		color: var(--text-tertiary);
	}
	.ui-toast-close:hover {
		background: var(--state-hover);
		color: var(--text-primary);
	}
	@keyframes ui-toast-in {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
	}
</style>
