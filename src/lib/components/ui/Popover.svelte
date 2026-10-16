<!--
	Popover (DOIT.md §17 10px radius, §32 normal 180ms). Floating overlay — one of
	the two places glass material is allowed (§9).

	<Popover bind:open placement="bottom-end">
		{#snippet trigger(props)}<Button {...props}>Filters</Button>{/snippet}
		…content…
	</Popover>

	The trigger snippet receives aria props plus an onclick that toggles the popover.
-->
<script lang="ts" module>
	export type PopoverTriggerProps = {
		'aria-expanded': boolean;
		'aria-haspopup': 'dialog' | 'listbox' | 'menu';
		'aria-controls': string;
		onclick: () => void;
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { computePosition, portal, uid, type Placement } from '$lib/ui/floating';

	interface Props {
		open?: boolean;
		placement?: Placement;
		role?: 'dialog' | 'listbox' | 'menu';
		/** Match the trigger width (selects). */
		matchWidth?: boolean;
		minWidth?: number;
		trigger: Snippet<[PopoverTriggerProps]>;
		children: Snippet;
		onOpenChange?: (open: boolean) => void;
		class?: string;
	}

	let {
		open = $bindable(false),
		placement = 'bottom-start',
		role = 'dialog',
		matchWidth = false,
		minWidth = 180,
		trigger,
		children,
		onOpenChange,
		class: className = ''
	}: Props = $props();

	const id = uid('popover');
	let anchor = $state<HTMLElement | null>(null);
	let panel = $state<HTMLElement | null>(null);
	let position = $state({ top: -9999, left: -9999, width: 0 });

	function setOpen(next: boolean) {
		open = next;
		onOpenChange?.(next);
	}

	function reposition() {
		if (!anchor || !panel) return;
		const target = (anchor.firstElementChild as HTMLElement | null) ?? anchor;
		const rect = target.getBoundingClientRect();
		const width = matchWidth ? Math.max(rect.width, minWidth) : 0;
		const { top, left } = computePosition(
			rect,
			{ width: width || panel.offsetWidth, height: panel.offsetHeight },
			placement
		);
		position = { top, left, width };
	}

	$effect(() => {
		if (open && panel) reposition();
	});

	function handlePointerDown(event: PointerEvent) {
		if (!open) return;
		const target = event.target as Node;
		if (panel?.contains(target) || anchor?.contains(target)) return;
		setOpen(false);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (open && event.key === 'Escape') {
			event.stopPropagation();
			setOpen(false);
			((anchor?.firstElementChild as HTMLElement | null) ?? anchor)?.focus();
		}
	}
</script>

<svelte:window
	onpointerdown={handlePointerDown}
	onkeydown={handleKeydown}
	onresize={() => open && reposition()}
/>

<span bind:this={anchor} class="ui-popover-anchor">
	{@render trigger({
		'aria-expanded': open,
		'aria-haspopup': role,
		'aria-controls': id,
		onclick: () => setOpen(!open)
	})}
</span>

{#if open}
	<div
		use:portal
		bind:this={panel}
		{id}
		class={`ui-popover ${className}`}
		role={role === 'dialog' ? 'dialog' : undefined}
		style:top={`${position.top}px`}
		style:left={`${position.left}px`}
		style:width={position.width ? `${position.width}px` : undefined}
		style:min-width={`${minWidth}px`}
	>
		{@render children()}
	</div>
{/if}

<style>
	.ui-popover-anchor {
		display: inline-flex;
	}
	.ui-popover {
		position: fixed;
		z-index: var(--z-dropdown);
		max-height: min(360px, calc(100vh - 32px));
		overflow: auto;
		padding: var(--space-1);
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-lg);
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(130%);
		box-shadow: var(--elevation-overlay);
		animation: ui-popover-in var(--duration-normal) var(--ease-standard);
	}
	@keyframes ui-popover-in {
		from {
			opacity: 0;
			transform: translateY(-2px);
		}
	}
</style>
