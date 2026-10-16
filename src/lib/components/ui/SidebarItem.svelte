<!--
	Sidebar navigation item (DOIT.md §9, §10).
	38px tall · 10px padding · 18px icon · 14px text · 10px gap · 8px radius.
	Rest: icon tertiary, text secondary. Hover: neutral tint. Active: neutral
	highlight + accent icon only (no blue blocks). Collapsed: icon + tooltip.
-->
<script lang="ts">
	import type { LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import Tooltip from './Tooltip.svelte';

	interface Props {
		/** Omit to render a button (e.g. the collapse toggle) instead of a link. */
		href?: string;
		icon: LucideIcon;
		label: string;
		active?: boolean;
		collapsed?: boolean;
		shortcut?: string;
		onclick?: (event: MouseEvent) => void;
	}

	let { href, icon, label, active = false, collapsed = false, shortcut, onclick }: Props = $props();
</script>

{#snippet link()}
	{#if href}
		<!-- `href` is resolve()'d by the Sidebar. -->
		<!-- eslint-disable svelte/no-navigation-without-resolve -->
		<a
			{href}
			class="ui-nav-item"
			class:is-active={active}
			class:is-collapsed={collapsed}
			aria-current={active ? 'page' : undefined}
			aria-label={collapsed ? label : undefined}
			{onclick}
		>
			<Icon {icon} size="sidebar" class="ui-nav-icon" />
			<span class="ui-nav-label">{label}</span>
		</a>
		<!-- eslint-enable svelte/no-navigation-without-resolve -->
	{:else}
		<button
			type="button"
			class="ui-nav-item"
			class:is-collapsed={collapsed}
			aria-label={collapsed ? label : undefined}
			{onclick}
		>
			<Icon {icon} size="sidebar" class="ui-nav-icon" />
			<span class="ui-nav-label">{label}</span>
		</button>
	{/if}
{/snippet}

{#if collapsed}
	<Tooltip text={label} {shortcut} placement="right" delay={150}>{@render link()}</Tooltip>
{:else}
	{@render link()}
{/if}

<style>
	.ui-nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		height: var(--sidebar-item-height);
		padding: 0 10px;
		border-radius: var(--radius-md);
		color: var(--text-secondary);
		font-size: var(--text-base);
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			color var(--duration-fast) var(--ease-standard);
	}
	.ui-nav-item :global(.ui-nav-icon) {
		color: var(--text-tertiary);
		transition: color var(--duration-fast) var(--ease-standard);
	}
	.ui-nav-item:hover {
		background: var(--state-hover);
		color: var(--text-primary);
	}
	.ui-nav-item:hover :global(.ui-nav-icon) {
		color: var(--text-secondary);
	}
	.ui-nav-item:active {
		background: var(--state-active);
	}
	.ui-nav-item.is-active {
		background: var(--state-active);
		color: var(--text-primary);
	}
	.ui-nav-item.is-active :global(.ui-nav-icon) {
		color: var(--accent);
	}
	.ui-nav-item.is-collapsed {
		width: 40px;
		padding: 0 11px;
	}
	.ui-nav-label {
		overflow: hidden;
		text-overflow: ellipsis;
		transition: opacity var(--duration-normal) var(--ease-standard);
	}
	.is-collapsed .ui-nav-label {
		opacity: 0;
	}
</style>
