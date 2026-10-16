<!--
	Modal dialog (DOIT.md §17 16px radius, §30 Esc, §32 slow 240ms).
	Focus moves into the dialog on open, is trapped while open, and returns to the
	previously focused element on close.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { X } from '@lucide/svelte';
	import IconButton from './IconButton.svelte';
	import { focusableWithin, portal, uid } from '$lib/ui/floating';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		open: boolean;
		title: string;
		description?: string;
		size?: 'sm' | 'md' | 'lg';
		/** Close when the backdrop is clicked. Off for destructive confirmations. */
		dismissible?: boolean;
		onClose: () => void;
		children?: Snippet;
		footer?: Snippet;
	}

	let {
		open,
		title,
		description,
		size = 'md',
		dismissible = true,
		onClose,
		children,
		footer
	}: Props = $props();

	const titleId = uid('modal-title');
	const descriptionId = uid('modal-desc');
	let dialog = $state<HTMLElement | null>(null);
	let previouslyFocused: HTMLElement | null = null;

	$effect(() => {
		if (!open || !dialog) return;
		previouslyFocused = document.activeElement as HTMLElement | null;
		const first =
			dialog.querySelector<HTMLElement>('[data-autofocus]') ?? focusableWithin(dialog)[0] ?? dialog;
		first.focus();
		return () => previouslyFocused?.focus?.();
	});

	function handleKeydown(event: KeyboardEvent) {
		if (!open || !dialog) return;
		if (event.key === 'Escape') {
			event.stopPropagation();
			onClose();
			return;
		}
		if (event.key === 'Tab') {
			const items = focusableWithin(dialog);
			if (items.length === 0) return;
			const first = items[0];
			const last = items[items.length - 1];
			if (event.shiftKey && document.activeElement === first) {
				event.preventDefault();
				last.focus();
			} else if (!event.shiftKey && document.activeElement === last) {
				event.preventDefault();
				first.focus();
			}
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div use:portal class="ui-modal-root">
		<div
			class="ui-modal-backdrop"
			aria-hidden="true"
			onclick={() => dismissible && onClose()}
		></div>
		<div
			bind:this={dialog}
			class={`ui-modal ui-modal--${size}`}
			role="dialog"
			aria-modal="true"
			aria-labelledby={titleId}
			aria-describedby={description ? descriptionId : undefined}
			tabindex="-1"
		>
			<header class="ui-modal-header">
				<div class="min-w-0">
					<h2 id={titleId} class="ui-modal-title">{title}</h2>
					{#if description}
						<p id={descriptionId} class="ui-modal-description">{description}</p>
					{/if}
				</div>
				<IconButton
					icon={X}
					label={t('关闭', 'Close')}
					shortcut="Esc"
					size="sm"
					onclick={onClose}
				/>
			</header>
			{#if children}<div class="ui-modal-body">{@render children()}</div>{/if}
			{#if footer}<footer class="ui-modal-footer">{@render footer()}</footer>{/if}
		</div>
	</div>
{/if}

<style>
	.ui-modal-root {
		position: fixed;
		inset: 0;
		z-index: var(--z-modal);
		display: grid;
		place-items: center;
		padding: var(--space-6);
	}
	.ui-modal-backdrop {
		position: absolute;
		inset: 0;
		background: var(--scrim);
		animation: ui-fade var(--duration-slow) var(--ease-standard);
	}
	.ui-modal {
		position: relative;
		width: 100%;
		max-height: calc(100vh - 96px);
		display: flex;
		flex-direction: column;
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-2xl);
		background: var(--bg-elevated);
		backdrop-filter: blur(24px) saturate(130%);
		box-shadow: var(--elevation-overlay);
		animation: ui-modal-in var(--duration-slow) var(--ease-standard);
	}
	.ui-modal:focus {
		outline: none;
	}
	.ui-modal--sm {
		max-width: 400px;
	}
	.ui-modal--md {
		max-width: 520px;
	}
	.ui-modal--lg {
		max-width: 720px;
	}
	.ui-modal-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-5) var(--space-5) 0;
	}
	.ui-modal-title {
		font-size: var(--text-lg);
		font-weight: 600;
		line-height: 22px;
		color: var(--text-primary);
	}
	.ui-modal-description {
		margin-top: var(--space-1);
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
	.ui-modal-body {
		overflow: auto;
		padding: var(--space-4) var(--space-5) 0;
		font-size: var(--text-base);
		color: var(--text-secondary);
	}
	.ui-modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
		padding: var(--space-5);
	}
	@keyframes ui-fade {
		from {
			opacity: 0;
		}
	}
	@keyframes ui-modal-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
	}
</style>
