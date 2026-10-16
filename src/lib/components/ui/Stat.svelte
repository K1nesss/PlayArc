<!--
	Typographic metric (DOIT.md §5, §7, §14). No box — hierarchy comes from type
	and spacing. Group several with <StatGroup>.

	<Stat label="Total time" value="142.6" unit="h" />
	<Stat label="Sessions" value="53" size="lg" hint="+4 this week" />
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Skeleton from './Skeleton.svelte';

	interface Props {
		label: string;
		value: string | number;
		unit?: string;
		hint?: string;
		/** md = 18px · lg = metric 30px · hero = 38px */
		size?: 'md' | 'lg' | 'hero';
		loading?: boolean;
		trailing?: Snippet;
		class?: string;
	}

	let {
		label,
		value,
		unit,
		hint,
		size = 'lg',
		loading = false,
		trailing,
		class: className = ''
	}: Props = $props();
</script>

<div class={`ui-stat ui-stat--${size} ${className}`}>
	<span class="ui-stat-label">{label}</span>
	{#if loading}
		<Skeleton
			width="96px"
			height={size === 'md' ? '18px' : size === 'lg' ? '30px' : '38px'}
			radius="md"
		/>
	{:else}
		<span class="ui-stat-value">
			{value}{#if unit}<span class="ui-stat-unit">{unit}</span>{/if}
			{#if trailing}{@render trailing()}{/if}
		</span>
	{/if}
	{#if hint}<span class="ui-stat-hint">{hint}</span>{/if}
</div>

<style>
	.ui-stat {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.ui-stat-label {
		font-size: var(--text-xs);
		font-weight: 500;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.ui-stat-value {
		display: inline-flex;
		align-items: baseline;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
		white-space: nowrap;
	}
	.ui-stat--md .ui-stat-value {
		font-size: var(--text-xl);
		line-height: var(--text-xl--line-height);
	}
	.ui-stat--lg .ui-stat-value {
		font-size: var(--text-3xl);
		line-height: var(--text-3xl--line-height);
		letter-spacing: -0.01em;
	}
	.ui-stat--hero .ui-stat-value {
		font-size: var(--text-4xl);
		line-height: var(--text-4xl--line-height);
		letter-spacing: -0.015em;
	}
	.ui-stat-unit {
		margin-left: 1px;
		font-size: 0.55em;
		font-weight: 500;
		color: var(--text-secondary);
	}
	.ui-stat--md .ui-stat-unit {
		font-size: 0.8em;
	}
	.ui-stat-hint {
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
</style>
