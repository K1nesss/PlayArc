<!--
	Icon-only button (DOIT.md §29). `label` is required: it becomes the tooltip
	and the accessible name, so an icon's meaning is never guess-work.
-->
<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { LoaderCircle, type LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import Tooltip from './Tooltip.svelte';
	import type { Placement } from '$lib/ui/floating';

	interface Props extends Omit<HTMLButtonAttributes, 'class'> {
		icon: LucideIcon;
		label: string;
		shortcut?: string;
		variant?: 'ghost' | 'secondary';
		size?: 'sm' | 'md' | 'lg';
		active?: boolean;
		loading?: boolean;
		tooltipPlacement?: Placement;
		class?: string;
	}

	let {
		icon,
		label,
		shortcut,
		variant = 'ghost',
		size = 'md',
		active = false,
		loading = false,
		disabled = false,
		tooltipPlacement = 'top',
		type = 'button',
		class: className = '',
		...rest
	}: Props = $props();
</script>

<Tooltip text={label} {shortcut} placement={tooltipPlacement}>
	<button
		{type}
		class={`ui-icon-button ui-icon-button--${variant} ui-icon-button--${size} ${active ? 'is-active' : ''} ${className}`}
		aria-label={label}
		aria-pressed={active || undefined}
		aria-busy={loading || undefined}
		disabled={disabled || loading}
		{...rest}
	>
		<Icon icon={loading ? LoaderCircle : icon} size="button" class={loading ? 'ui-spin' : ''} />
	</button>
</Tooltip>

<style>
	.ui-icon-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		border: 1px solid transparent;
		border-radius: var(--radius-md);
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			border-color var(--duration-fast) var(--ease-standard),
			color var(--duration-fast) var(--ease-standard);
	}

	.ui-icon-button--sm {
		width: var(--control-sm);
		height: var(--control-sm);
	}
	.ui-icon-button--md {
		width: var(--control-md);
		height: var(--control-md);
	}
	.ui-icon-button--lg {
		width: var(--control-lg);
		height: var(--control-lg);
	}

	.ui-icon-button--secondary {
		background: var(--bg-surface);
		border-color: var(--border-normal);
	}

	.ui-icon-button:hover:not(:disabled) {
		background: var(--state-hover);
		color: var(--text-primary);
	}
	.ui-icon-button--secondary:hover:not(:disabled) {
		background: var(--bg-subtle);
		border-color: var(--border-strong);
	}

	.ui-icon-button:active:not(:disabled),
	.ui-icon-button.is-active {
		background: var(--state-active);
		color: var(--text-primary);
	}

	.ui-icon-button:disabled {
		cursor: not-allowed;
		opacity: 0.45;
	}
</style>
