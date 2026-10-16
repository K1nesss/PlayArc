<!--
	Segmented control — used for the Analysis range picker
	([7 days] [30 days] [90 days] [1 year] [All]) and Grid / List toggles.
	Neutral highlight, no accent fill (§10). Arrow keys move the selection.
-->
<script lang="ts" generics="T extends string | number">
	import type { LucideIcon } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import Tooltip from './Tooltip.svelte';

	type Option = { value: T; label: string; icon?: LucideIcon; iconOnly?: boolean };

	interface Props {
		value: T;
		options: Option[];
		ariaLabel: string;
		size?: 'sm' | 'md';
		onchange?: (value: T) => void;
	}

	let { value = $bindable(), options, ariaLabel, size = 'md', onchange }: Props = $props();

	let root = $state<HTMLElement | null>(null);

	function select(next: T) {
		if (next === value) return;
		value = next;
		onchange?.(next);
	}

	function handleKeydown(event: KeyboardEvent) {
		const index = options.findIndex((option) => option.value === value);
		let nextIndex = index;
		if (event.key === 'ArrowRight' || event.key === 'ArrowDown') nextIndex = index + 1;
		else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') nextIndex = index - 1;
		else return;
		event.preventDefault();
		nextIndex = (nextIndex + options.length) % options.length;
		select(options[nextIndex].value);
		root?.querySelectorAll<HTMLElement>('[role="radio"]')[nextIndex]?.focus();
	}
</script>

<div
	bind:this={root}
	class={`ui-segmented ui-segmented--${size}`}
	role="radiogroup"
	aria-label={ariaLabel}
	tabindex="-1"
	onkeydown={handleKeydown}
>
	{#each options as option (option.value)}
		{@const checked = option.value === value}
		{#if option.iconOnly && option.icon}
			<Tooltip text={option.label}>
				<button
					type="button"
					role="radio"
					aria-checked={checked}
					aria-label={option.label}
					tabindex={checked ? 0 : -1}
					class="ui-segment ui-segment--icon"
					onclick={() => select(option.value)}
				>
					<Icon icon={option.icon} size="button" />
				</button>
			</Tooltip>
		{:else}
			<button
				type="button"
				role="radio"
				aria-checked={checked}
				tabindex={checked ? 0 : -1}
				class="ui-segment"
				onclick={() => select(option.value)}
			>
				{#if option.icon}<Icon icon={option.icon} size="button" />{/if}
				{option.label}
			</button>
		{/if}
	{/each}
</div>

<style>
	.ui-segmented {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 2px;
		border-radius: var(--radius-md);
		background: var(--state-hover);
	}
	.ui-segmented:focus {
		outline: none;
	}
	.ui-segment {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 0 var(--space-3);
		border: 1px solid transparent;
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-size: var(--text-sm);
		font-weight: 500;
		white-space: nowrap;
		cursor: pointer;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			color var(--duration-fast) var(--ease-standard);
	}
	.ui-segmented--md .ui-segment {
		height: 28px;
	}
	.ui-segmented--sm .ui-segment {
		height: 24px;
		padding: 0 var(--space-2);
		font-size: var(--text-xs);
	}
	.ui-segment--icon {
		padding: 0;
		aspect-ratio: 1;
	}
	.ui-segment:hover {
		color: var(--text-primary);
	}
	.ui-segment[aria-checked='true'] {
		border-color: var(--border-subtle);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
		color: var(--text-primary);
	}
</style>
