<!--
	App sidebar (DOIT.md §9–§11).
	228px expanded / 64px collapsed. Brand row doubles as title-bar drag area.
	Material: Mica on Windows 11 (shows through a transparent background),
	solid --bg-sidebar-solid everywhere else.
-->
<script lang="ts">
	import { ChartColumn, Clock, Gamepad2, House, Settings } from '@lucide/svelte';
	import { resolve } from '$app/paths';
	import SidebarItem from '$lib/components/ui/SidebarItem.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import SyncStatus from './SyncStatus.svelte';
	import { t } from '$lib/i18n.svelte';

	interface Props {
		pathname: string;
		collapsed: boolean;
		/** Narrow windows force the compact sidebar; the toggle is hidden then. */
		forcedCollapsed: boolean;
		onToggleCollapse: () => void;
		onNavigate: () => void;
		onSync: () => void;
		onOpenSettings: () => void;
	}

	let {
		pathname,
		collapsed,
		forcedCollapsed,
		onToggleCollapse,
		onNavigate,
		onSync,
		onOpenSettings
	}: Props = $props();

	const nav = $derived([
		{
			key: 'home',
			href: resolve('/home'),
			match: ['/home', '/'],
			icon: House,
			label: t('首页', 'Home')
		},
		{
			key: 'games',
			href: resolve('/games'),
			match: ['/games'],
			icon: Gamepad2,
			label: t('游戏库', 'Games')
		},
		{
			key: 'timeline',
			href: resolve('/timeline'),
			match: ['/timeline'],
			icon: Clock,
			label: t('时间线', 'Timeline')
		},
		{
			key: 'analysis',
			href: resolve('/analytics'),
			match: ['/analytics'],
			icon: ChartColumn,
			label: t('分析', 'Analysis')
		}
	]);

	function isActive(match: string[]) {
		return match.some((prefix) =>
			prefix === '/' ? pathname === '/' : pathname === prefix || pathname.startsWith(`${prefix}/`)
		);
	}
</script>

<aside
	class="shell-sidebar"
	class:is-collapsed={collapsed}
	aria-label={t('主导航', 'Main navigation')}
>
	<div class="sidebar-brand" data-tauri-drag-region>
		<!-- The star from the app icon (current text colour) doubles as the sidebar toggle. -->
		{#snippet star()}
			<svg class="brand-mark" viewBox="-34 -34 68 68" aria-hidden="true">
				<path
					d="M 0 -34 C 3 -10 10 -3 34 0 C 10 3 3 10 0 34 C -3 10 -10 3 -34 0 C -10 -3 -3 -10 0 -34 Z"
					fill="currentColor"
				/>
			</svg>
		{/snippet}
		{#if forcedCollapsed}
			<span class="brand-toggle is-static" data-tauri-drag-region>{@render star()}</span>
		{:else}
			<Tooltip
				text={collapsed ? t('展开侧边栏', 'Expand sidebar') : t('收起侧边栏', 'Collapse sidebar')}
				shortcut="Ctrl+B"
				placement={collapsed ? 'right' : 'bottom'}
				delay={300}
			>
				<button
					type="button"
					class="brand-toggle"
					aria-label={collapsed
						? t('展开侧边栏', 'Expand sidebar')
						: t('收起侧边栏', 'Collapse sidebar')}
					aria-expanded={!collapsed}
					onclick={onToggleCollapse}
				>
					{@render star()}
				</button>
			</Tooltip>
		{/if}
		{#if !collapsed}
			<span class="brand-name" data-tauri-drag-region>PlayArc</span>
		{/if}
	</div>

	<nav class="sidebar-nav">
		{#each nav as item (item.key)}
			<SidebarItem
				href={item.href}
				icon={item.icon}
				label={item.label}
				active={isActive(item.match)}
				{collapsed}
				onclick={onNavigate}
			/>
		{/each}
	</nav>

	<div class="sidebar-spacer" data-tauri-drag-region></div>

	<div class="sidebar-footer">
		<SyncStatus {collapsed} {onSync} onConnect={onOpenSettings} />
		<SidebarItem
			href={resolve('/settings')}
			icon={Settings}
			label={t('设置', 'Settings')}
			shortcut="Ctrl+,"
			active={isActive(['/settings'])}
			{collapsed}
			onclick={onNavigate}
		/>
	</div>
</aside>

<style>
	.shell-sidebar {
		position: relative;
		z-index: var(--z-sidebar);
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
		width: var(--sidebar-width);
		height: 100%;
		padding: 0 var(--space-3) var(--space-3);
		background: var(--shell-chrome-bg);
		transition: width var(--duration-slow) var(--ease-standard);
		overflow: hidden;
	}
	.shell-sidebar.is-collapsed {
		width: var(--sidebar-width-collapsed);
	}

	.sidebar-brand {
		display: flex;
		align-items: center;
		height: var(--titlebar-height);
		/* star centred over the nav icons, name on the same line as the nav labels
		   (the star's 32px button has a 6px inset around the 20px star) */
		gap: 3px;
		padding: 0 6px 0 3px;
		flex-shrink: 0;
	}
	.is-collapsed .sidebar-brand {
		padding: 0 4px;
	}
	.brand-toggle {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		color: var(--text-primary);
		transition: background-color var(--duration-fast) var(--ease-standard);
	}
	.brand-toggle:not(.is-static):hover {
		background: var(--state-hover);
	}
	.brand-toggle:not(.is-static):active {
		background: var(--state-active);
	}
	.brand-toggle.is-static {
		pointer-events: none;
	}
	.brand-mark {
		width: 20px;
		height: 20px;
		flex-shrink: 0;
		pointer-events: none;
	}
	.brand-name {
		font-size: var(--text-xl);
		line-height: var(--text-xl--line-height);
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
	}

	.sidebar-nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: var(--space-2);
	}
	.sidebar-spacer {
		flex: 1;
		min-height: var(--space-4);
	}

	.sidebar-footer {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
</style>
