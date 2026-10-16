<!--
	Confirmation dialog built on <Modal>. Same props as before, so existing callers
	keep working. Destructive confirmations use a red button only (§8) and cannot be
	dismissed by clicking the backdrop.
-->
<script lang="ts">
	import Modal from './ui/Modal.svelte';
	import Button from './ui/Button.svelte';

	type ConfirmVariant = 'default' | 'danger';

	interface Props {
		open?: boolean;
		title?: string;
		message?: string;
		confirmText?: string;
		cancelText?: string;
		variant?: ConfirmVariant;
		showCancel?: boolean;
		loading?: boolean;
		onConfirm?: () => void;
		onCancel?: () => void;
	}

	let {
		open = false,
		title = '',
		message = '',
		confirmText = 'OK',
		cancelText = 'Cancel',
		variant = 'default',
		showCancel = true,
		loading = false,
		onConfirm = () => {},
		onCancel = () => {}
	}: Props = $props();
</script>

<Modal
	{open}
	{title}
	description={message}
	size="sm"
	dismissible={variant !== 'danger'}
	onClose={onCancel}
>
	{#snippet footer()}
		{#if showCancel}
			<Button
				variant="secondary"
				onclick={onCancel}
				data-autofocus={variant === 'danger' || undefined}
			>
				{cancelText}
			</Button>
		{/if}
		<Button
			variant={variant === 'danger' ? 'danger' : 'primary'}
			{loading}
			onclick={onConfirm}
			data-autofocus={variant !== 'danger' || undefined}
		>
			{confirmText}
		</Button>
	{/snippet}
</Modal>
