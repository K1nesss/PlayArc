<!--
	Tooltip (DOIT.md §29). Wrap any trigger; shows on hover (after a short delay)
	and on keyboard focus, hides on Escape / leave / blur.

	<Tooltip text="Sync Steam data" shortcut="Ctrl+R"><IconButton … /></Tooltip>

	IconButton already wraps itself in a Tooltip, so it is rarely used directly.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { computePosition, portal, uid, type Placement } from '$lib/ui/floating';

	interface Props {
		text: string;
		shortcut?: string;
		placement?: Placement;
		/** Delay before showing on hover, ms. Focus shows immediately. */
		delay?: number;
		disabled?: boolean;
		children: Snippet;
	}

	let {
		text,
		shortcut,
		placement = 'top',
		delay = 400,
		disabled = false,
		children
	}: Props = $props();

	const id = uid('tooltip');
	let anchor = $state<HTMLElement | null>(null);
	let bubble = $state<HTMLElement | null>(null);
	let visible = $state(false);
	let position = $state({ top: -9999, left: -9999 });
	let timer: number | null = null;

	function clearTimer() {
		if (timer !== null) {
			window.clearTimeout(timer);
			timer = null;
		}
	}

	function show(immediate = false) {
		if (disabled || !text) return;
		clearTimer();
		if (immediate) {
			visible = true;
		} else {
			timer = window.setTimeout(() => (visible = true), delay);
		}
	}

	function hide() {
		clearTimer();
		visible = false;
	}

	$effect(() => {
		if (!visible || !anchor || !bubble) return;
		const target = (anchor.firstElementChild as HTMLElement | null) ?? anchor;
		position = computePosition(
			target.getBoundingClientRect(),
			{ width: bubble.offsetWidth, height: bubble.offsetHeight },
			placement
		);
	});

	$effect(() => clearTimer);
</script>

<svelte:window
	onkeydown={(event) => {
		if (event.key === 'Escape') hide();
	}}
	onscroll={hide}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
	bind:this={anchor}
	class="ui-tooltip-anchor"
	aria-describedby={visible ? id : undefined}
	onpointerenter={() => show()}
	onpointerleave={hide}
	onpointerdown={hide}
	onfocusin={(event) => {
		if ((event.target as HTMLElement).matches(':focus-visible')) show(true);
	}}
	onfocusout={hide}
>
	{@render children()}
</span>

{#if visible}
	<div
		use:portal
		bind:this={bubble}
		{id}
		role="tooltip"
		class="ui-tooltip"
		style:top={`${position.top}px`}
		style:left={`${position.left}px`}
	>
		<span>{text}</span>
		{#if shortcut}<kbd class="ui-tooltip-kbd">{shortcut}</kbd>{/if}
	</div>
{/if}

<style>
	.ui-tooltip-anchor {
		display: inline-flex;
	}

	.ui-tooltip {
		position: fixed;
		z-index: var(--z-tooltip);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		max-width: 280px;
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-sm);
		background: var(--bg-elevated);
		backdrop-filter: blur(16px) saturate(130%);
		box-shadow: var(--elevation-2);
		color: var(--text-primary);
		font-size: var(--text-xs);
		line-height: 16px;
		pointer-events: none;
		animation: ui-tooltip-in var(--duration-fast) var(--ease-standard);
	}

	.ui-tooltip-kbd {
		font-family: inherit;
		font-size: 11px;
		color: var(--text-tertiary);
	}

	@keyframes ui-tooltip-in {
		from {
			opacity: 0;
		}
	}
</style>
