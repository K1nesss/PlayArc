<!--
	Text input (DOIT.md §35). Optional leading icon, clear button and invalid state.
	Pair invalid inputs with an <InlineNotice tone="danger"> or `error` text (§28).
-->
<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { X, type LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props extends Omit<HTMLInputAttributes, 'class' | 'size' | 'value'> {
		value?: string;
		icon?: LucideIcon;
		clearable?: boolean;
		invalid?: boolean;
		error?: string;
		size?: 'sm' | 'md';
		class?: string;
		input?: HTMLInputElement | null;
		onclear?: () => void;
	}

	let {
		value = $bindable(''),
		icon,
		clearable = false,
		invalid = false,
		error,
		size = 'md',
		type = 'text',
		class: className = '',
		input = $bindable(null),
		onclear,
		...rest
	}: Props = $props();

	const hasError = $derived(invalid || Boolean(error));
</script>

<div class={`ui-input-wrap ${className}`}>
	<div class={`ui-input ui-input--${size} ${hasError ? 'is-invalid' : ''}`}>
		{#if icon}<Icon {icon} size="button" class="ui-input-icon" />{/if}
		<input bind:this={input} bind:value {type} aria-invalid={hasError || undefined} {...rest} />
		{#if clearable && value}
			<button
				type="button"
				class="ui-input-clear"
				aria-label={t('清空', 'Clear')}
				onclick={() => {
					value = '';
					onclear?.();
					input?.focus();
				}}
			>
				<Icon icon={X} size="status" />
			</button>
		{/if}
	</div>
	{#if error}<p class="ui-input-error" role="alert">{error}</p>{/if}
</div>

<style>
	.ui-input-wrap {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		min-width: 0;
	}
	.ui-input {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		color: var(--text-primary);
		transition: border-color var(--duration-fast) var(--ease-standard);
	}
	.ui-input--md {
		height: var(--control-md);
		padding: 0 var(--space-2) 0 var(--space-3);
	}
	.ui-input--sm {
		height: var(--control-sm);
		padding: 0 6px 0 var(--space-2);
	}
	.ui-input:hover {
		border-color: var(--border-strong);
	}
	.ui-input:focus-within {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.ui-input.is-invalid {
		border-color: var(--danger);
	}
	.ui-input.is-invalid:focus-within {
		box-shadow: 0 0 0 3px var(--danger-soft);
	}
	.ui-input input {
		flex: 1;
		min-width: 0;
		height: 100%;
		border: 0;
		background: transparent;
		color: inherit;
		font-size: var(--text-base);
	}
	.ui-input--sm input {
		font-size: var(--text-sm);
	}
	/* The wrapper shows focus; the bare input does not need a second ring. */
	.ui-input input:focus-visible {
		outline: none;
	}
	.ui-input input::placeholder {
		color: var(--text-tertiary);
	}
	.ui-input :global(.ui-input-icon) {
		color: var(--text-tertiary);
	}
	.ui-input-clear {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--radius-sm);
		color: var(--text-tertiary);
	}
	.ui-input-clear:hover {
		background: var(--state-hover);
		color: var(--text-primary);
	}
	.ui-input-error {
		font-size: var(--text-xs);
		color: var(--danger);
	}
</style>
