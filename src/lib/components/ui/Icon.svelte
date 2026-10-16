<!--
	Single entry point for icons (DOIT.md §15). Lucide only, stroke 1.75.

	Usage:
		<script>import { RefreshCw } from '@lucide/svelte';</script>
		<Icon icon={RefreshCw} size="button" />

	Sizes: sidebar 18 · button/toolbar 16 · status 14 · empty 28 (or a number).
-->
<script lang="ts" module>
	export type IconSize = 'status' | 'button' | 'toolbar' | 'sidebar' | 'empty' | number;

	const SIZE_MAP: Record<Exclude<IconSize, number>, number> = {
		status: 14,
		button: 16,
		toolbar: 16,
		sidebar: 18,
		empty: 28
	};
</script>

<script lang="ts">
	import type { LucideIcon } from '@lucide/svelte';

	interface Props {
		icon: LucideIcon;
		size?: IconSize;
		/** Accessible label. Omit for decorative icons next to visible text. */
		label?: string;
		class?: string;
	}

	let { icon: IconComponent, size = 'button', label, class: className = '' }: Props = $props();

	const pixels = $derived(typeof size === 'number' ? size : SIZE_MAP[size]);
</script>

<IconComponent
	size={pixels}
	strokeWidth={1.75}
	class={`shrink-0 ${className}`}
	aria-hidden={label ? undefined : 'true'}
	aria-label={label}
	role={label ? 'img' : undefined}
/>
