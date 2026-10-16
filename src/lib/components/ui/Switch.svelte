<!--
	On/off toggle (role="switch"). Accent only for the "on" track — the one
	place a fill color signals state. Space/Enter toggle; disabled is dimmed.
-->
<script lang="ts">
	interface Props {
		checked: boolean;
		/** Accessible name — usually the settings row label. */
		label: string;
		disabled?: boolean;
		onchange?: (checked: boolean) => void;
		id?: string;
	}

	let { checked = $bindable(false), label, disabled = false, onchange, id }: Props = $props();

	function toggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}
</script>

<button
	{id}
	type="button"
	role="switch"
	class="ui-switch"
	aria-checked={checked}
	aria-label={label}
	{disabled}
	onclick={toggle}
>
	<span class="ui-switch-thumb" aria-hidden="true"></span>
</button>

<style>
	.ui-switch {
		position: relative;
		flex-shrink: 0;
		width: 36px;
		height: 20px;
		border: 1px solid var(--border-strong);
		border-radius: 999px;
		background: var(--state-active);
		cursor: pointer;
		transition:
			background-color var(--duration-fast) var(--ease-standard),
			border-color var(--duration-fast) var(--ease-standard);
	}
	.ui-switch:hover:not(:disabled) {
		border-color: color-mix(in oklab, var(--text-primary) 30%, transparent);
	}
	.ui-switch[aria-checked='true'] {
		border-color: transparent;
		background: var(--accent);
	}
	.ui-switch[aria-checked='true']:hover:not(:disabled) {
		background: var(--accent-hover);
	}
	.ui-switch-thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--text-secondary);
		transition:
			transform var(--duration-fast) var(--ease-standard),
			background-color var(--duration-fast) var(--ease-standard);
	}
	.ui-switch[aria-checked='true'] .ui-switch-thumb {
		transform: translateX(16px);
		background: var(--on-accent);
	}
	.ui-switch:disabled {
		cursor: not-allowed;
		opacity: 0.45;
	}
</style>
