/**
 * Non-intrusive toasts (DOIT.md §28).
 *
 * Toasts are for confirmations ("Sync complete") and soft notices. Critical
 * errors that need the user to act (e.g. an invalid Steam API key) must ALSO be
 * shown inline next to the relevant control — never rely on a toast alone.
 */

export type ToastTone = 'success' | 'info' | 'error';

export type ToastAction = { label: string; onClick: () => void };

export type ToastItem = {
	id: number;
	tone: ToastTone;
	title: string;
	description?: string;
	action?: ToastAction;
	duration: number;
};

type ToastInput = Omit<ToastItem, 'id' | 'tone' | 'duration'> & { duration?: number };

const MAX_VISIBLE = 3;
// Timer handles are bookkeeping, not UI state.
// eslint-disable-next-line svelte/prefer-svelte-reactivity
const timers = new Map<number, number>();
let nextId = 1;

export const toasts = $state<ToastItem[]>([]);

export function dismissToast(id: number) {
	const index = toasts.findIndex((item) => item.id === id);
	if (index >= 0) {
		toasts.splice(index, 1);
	}
	const timer = timers.get(id);
	if (timer !== undefined) {
		window.clearTimeout(timer);
		timers.delete(id);
	}
}

function push(tone: ToastTone, input: ToastInput | string) {
	const data = typeof input === 'string' ? { title: input } : input;
	const item: ToastItem = {
		id: nextId++,
		tone,
		duration: data.duration ?? (tone === 'error' ? 6000 : 4000),
		...data
	};

	toasts.push(item);
	while (toasts.length > MAX_VISIBLE) {
		dismissToast(toasts[0].id);
	}

	if (item.duration > 0) {
		timers.set(
			item.id,
			window.setTimeout(() => dismissToast(item.id), item.duration)
		);
	}
	return item.id;
}

export const toast = {
	success: (input: ToastInput | string) => push('success', input),
	info: (input: ToastInput | string) => push('info', input),
	error: (input: ToastInput | string) => push('error', input),
	dismiss: dismissToast
};
