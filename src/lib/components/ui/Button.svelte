<!--
	Button (DOIT.md §25, §35).
	States: default · hover · active · focus-visible · disabled · loading.
	Renders an <a> when `href` is set.
-->
<script lang="ts" module>
	export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
	export type ButtonSize = 'sm' | 'md' | 'lg';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { LoaderCircle, type LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';

	interface Props extends Omit<HTMLButtonAttributes, 'class'> {
		variant?: ButtonVariant;
		size?: ButtonSize;
		icon?: LucideIcon;
		iconRight?: LucideIcon;
		loading?: boolean;
		href?: string;
		fullWidth?: boolean;
		class?: string;
		children?: Snippet;
	}

	let {
		variant = 'secondary',
		size = 'md',
		icon,
		iconRight,
		loading = false,
		disabled = false,
		href,
		fullWidth = false,
		type = 'button',
		class: className = '',
		children,
		...rest
	}: Props = $props();

	const classes = $derived(
		[
			'ui-button',
			`ui-button--${variant}`,
			`ui-button--${size}`,
			fullWidth ? 'w-full' : '',
			loading ? 'is-loading' : '',
			className
		]
			.filter(Boolean)
			.join(' ')
	);
</script>

{#if href && !disabled}
	<!-- `href` must already be resolve()'d by the caller. -->
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
	<a {href} class={classes}>
		{#if icon}<Icon {icon} size="button" />{/if}
		{@render children?.()}
		{#if iconRight}<Icon icon={iconRight} size="button" />{/if}
	</a>
{:else}
	<button
		{type}
		class={classes}
		disabled={disabled || loading}
		aria-busy={loading || undefined}
		{...rest}
	>
		{#if loading}
			<Icon icon={LoaderCircle} size="button" class="ui-spin" />
		{:else if icon}
			<Icon {icon} size="button" />
		{/if}
		{@render children?.()}
		{#if iconRight && !loading}<Icon icon={iconRight} size="button" />{/if}
	</button>
{/if}

<style>
	.ui-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		border: 1px solid transparent;
		border-radius: var(--radius-md);
		font-size: var(--text-base);
		font-weight: 500;
		line-height: 1;
		white-space: nowrap;
		cursor: pointer;
		user-select: none;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			border-color var(--duration-fast) var(--ease-standard),
			color var(--duration-fast) var(--ease-standard);
	}

	.ui-button--sm {
		height: var(--control-sm);
		padding: 0 var(--space-2);
		font-size: var(--text-sm);
		gap: 6px;
	}
	.ui-button--md {
		height: var(--control-md);
		padding: 0 var(--space-3);
	}
	.ui-button--lg {
		height: var(--control-lg);
		padding: 0 var(--space-4);
	}

	.ui-button--primary {
		background: var(--accent);
		color: var(--on-accent);
	}
	.ui-button--primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.ui-button--secondary {
		background: var(--bg-surface);
		border-color: var(--border-normal);
		color: var(--text-primary);
	}
	.ui-button--secondary:hover:not(:disabled) {
		border-color: var(--border-strong);
		background: var(--bg-subtle);
	}

	.ui-button--ghost {
		background: transparent;
		color: var(--text-secondary);
	}
	.ui-button--ghost:hover:not(:disabled) {
		background: var(--state-hover);
		color: var(--text-primary);
	}

	/* §8: red is reserved for individual destructive buttons */
	.ui-button--danger {
		background: var(--bg-surface);
		border-color: var(--border-normal);
		color: var(--danger);
	}
	.ui-button--danger:hover:not(:disabled) {
		background: var(--danger-soft);
		border-color: color-mix(in oklab, var(--danger) 40%, transparent);
	}

	.ui-button:active:not(:disabled) {
		background-image: linear-gradient(var(--state-active), var(--state-active));
	}

	.ui-button:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}
	.ui-button.is-loading {
		cursor: progress;
		opacity: 0.8;
	}

	:global(.ui-spin) {
		animation: ui-spin 0.9s linear infinite;
	}
	@keyframes ui-spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
