<!--
	Skeleton placeholder (DOIT.md §26). Prefer this over spinners for page and
	panel loading. `lines` renders a stack of text bars with a shorter last line.
-->
<script lang="ts">
	interface Props {
		width?: string;
		height?: string;
		radius?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
		lines?: number;
		class?: string;
	}

	let {
		width = '100%',
		height = '12px',
		radius = 'sm',
		lines = 0,
		class: className = ''
	}: Props = $props();
</script>

{#if lines > 0}
	<div class={`flex flex-col gap-2 ${className}`} aria-hidden="true">
		{#each Array.from({ length: lines }, (_, i) => i) as index (index)}
			<span
				class={`ui-skeleton ui-skeleton--${radius}`}
				style:height
				style:width={index === lines - 1 && lines > 1 ? '60%' : width}
			></span>
		{/each}
	</div>
{:else}
	<span
		class={`ui-skeleton ui-skeleton--${radius} ${className}`}
		style:width
		style:height
		aria-hidden="true"
	></span>
{/if}

<style>
	.ui-skeleton {
		display: block;
		position: relative;
		overflow: hidden;
		background: var(--skeleton-base);
	}
	.ui-skeleton::after {
		content: '';
		position: absolute;
		inset: 0;
		transform: translateX(-100%);
		background: linear-gradient(90deg, transparent, var(--skeleton-highlight), transparent);
		animation: ui-skeleton 1.4s ease-in-out infinite;
	}
	.ui-skeleton--sm {
		border-radius: var(--radius-sm);
	}
	.ui-skeleton--md {
		border-radius: var(--radius-md);
	}
	.ui-skeleton--lg {
		border-radius: var(--radius-lg);
	}
	.ui-skeleton--xl {
		border-radius: var(--radius-xl);
	}
	.ui-skeleton--full {
		border-radius: 999px;
	}
	@keyframes ui-skeleton {
		to {
			transform: translateX(100%);
		}
	}
</style>
