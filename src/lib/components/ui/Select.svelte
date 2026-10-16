<!--
	Select (DOIT.md §35). Custom listbox so it matches both themes (native <select>
	menus ignore dark mode on Windows). Keyboard: ↑ ↓ Home End to move, Enter /
	Space to pick, Esc to close, type a letter to jump.

	<Select bind:value options={[{ value: 'all', label: 'All games' }, …]} ariaLabel="Game" />
-->
<script lang="ts" module>
	export type SelectOption<T extends string | number = string> = {
		value: T;
		label: string;
		description?: string;
	};
</script>

<script lang="ts" generics="T extends string | number">
	import { Check, ChevronDown } from '@lucide/svelte';
	import Icon from './Icon.svelte';
	import Popover from './Popover.svelte';
	import { uid } from '$lib/ui/floating';

	interface Props {
		value: T;
		options: SelectOption<T>[];
		ariaLabel: string;
		placeholder?: string;
		size?: 'sm' | 'md';
		/** ghost = borderless trigger, for page headers ("Sep 2026 ▾"). */
		variant?: 'default' | 'ghost';
		disabled?: boolean;
		placement?: 'bottom-start' | 'bottom-end';
		onchange?: (value: T) => void;
		class?: string;
	}

	let {
		value = $bindable(),
		options,
		ariaLabel,
		placeholder = '',
		size = 'md',
		variant = 'default',
		disabled = false,
		placement = 'bottom-start',
		onchange,
		class: className = ''
	}: Props = $props();

	const listId = uid('select');
	let open = $state(false);
	let activeIndex = $state(0);
	let list = $state<HTMLElement | null>(null);

	const selected = $derived(options.find((option) => option.value === value));

	function pick(option: SelectOption<T>) {
		value = option.value;
		onchange?.(option.value);
		open = false;
	}

	$effect(() => {
		if (!open) return;
		activeIndex = Math.max(
			0,
			options.findIndex((option) => option.value === value)
		);
		queueMicrotask(() => list?.focus());
	});

	$effect(() => {
		if (!open || !list) return;
		list
			.querySelector<HTMLElement>(`[data-index="${activeIndex}"]`)
			?.scrollIntoView({ block: 'nearest' });
	});

	function handleListKeydown(event: KeyboardEvent) {
		const last = options.length - 1;
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			activeIndex = Math.min(last, activeIndex + 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			activeIndex = Math.max(0, activeIndex - 1);
		} else if (event.key === 'Home') {
			event.preventDefault();
			activeIndex = 0;
		} else if (event.key === 'End') {
			event.preventDefault();
			activeIndex = last;
		} else if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			const option = options[activeIndex];
			if (option) pick(option);
		} else if (event.key === 'Tab') {
			open = false;
		} else if (event.key.length === 1) {
			const letter = event.key.toLowerCase();
			const start = activeIndex + 1;
			const ordered = [...options.slice(start), ...options.slice(0, start)];
			const match = ordered.find((option) => option.label.toLowerCase().startsWith(letter));
			if (match) activeIndex = options.indexOf(match);
		}
	}

	function handleTriggerKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			open = true;
		}
	}
</script>

<Popover bind:open role="listbox" {placement} matchWidth={variant === 'default'} minWidth={160}>
	{#snippet trigger(props)}
		<button
			type="button"
			class={`ui-select ui-select--${size} ui-select--${variant} ${className}`}
			aria-label={ariaLabel}
			{disabled}
			{...props}
			onkeydown={handleTriggerKeydown}
		>
			<span class={`truncate ${selected ? '' : 'ui-select-placeholder'}`}>
				{selected?.label ?? placeholder}
			</span>
			<Icon icon={ChevronDown} size="status" class="ui-select-chevron" />
		</button>
	{/snippet}

	<div
		bind:this={list}
		role="listbox"
		tabindex="-1"
		aria-label={ariaLabel}
		aria-activedescendant={`${listId}-${activeIndex}`}
		class="ui-select-list"
		onkeydown={handleListKeydown}
	>
		{#each options as option, index (option.value)}
			<div
				id={`${listId}-${index}`}
				data-index={index}
				role="option"
				tabindex="-1"
				aria-selected={option.value === value}
				class={`ui-select-option ${index === activeIndex ? 'is-active' : ''}`}
				onpointermove={() => (activeIndex = index)}
				onclick={() => pick(option)}
				onkeydown={() => {}}
			>
				<span class="min-w-0 flex-1">
					<span class="block truncate">{option.label}</span>
					{#if option.description}
						<span class="ui-select-option-description">{option.description}</span>
					{/if}
				</span>
				{#if option.value === value}
					<Icon icon={Check} size="status" class="text-(--accent)" />
				{/if}
			</div>
		{/each}
	</div>
</Popover>

<style>
	.ui-select {
		display: inline-flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		min-width: 0;
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		color: var(--text-primary);
		font-size: var(--text-base);
		cursor: pointer;
		transition:
			border-color var(--duration-fast) var(--ease-standard),
			background-color var(--duration-fast) var(--ease-standard);
	}
	.ui-select--md {
		height: var(--control-md);
		padding: 0 var(--space-2) 0 var(--space-3);
	}
	.ui-select--sm {
		height: var(--control-sm);
		padding: 0 6px 0 var(--space-2);
		font-size: var(--text-sm);
	}
	.ui-select--ghost {
		border-color: transparent;
		background: transparent;
		color: var(--text-secondary);
	}
	.ui-select:hover:not(:disabled) {
		border-color: var(--border-strong);
	}
	.ui-select--ghost:hover:not(:disabled) {
		border-color: transparent;
		background: var(--state-hover);
		color: var(--text-primary);
	}
	.ui-select[aria-expanded='true'] {
		border-color: var(--accent);
	}
	.ui-select--ghost[aria-expanded='true'] {
		border-color: transparent;
		background: var(--state-active);
	}
	.ui-select:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}
	.ui-select-placeholder {
		color: var(--text-tertiary);
	}
	.ui-select :global(.ui-select-chevron) {
		color: var(--text-tertiary);
	}

	.ui-select-list:focus {
		outline: none;
	}
	.ui-select-option {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-height: 32px;
		padding: 6px var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-base);
		color: var(--text-primary);
		cursor: pointer;
	}
	.ui-select-option.is-active {
		background: var(--state-hover);
	}
	.ui-select-option[aria-selected='true'] {
		font-weight: 500;
	}
	.ui-select-option-description {
		display: block;
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
</style>
