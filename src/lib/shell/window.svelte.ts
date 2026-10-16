/**
 * Window chrome state (DOIT.md §9, §13).
 *
 * - material: 'mica' on Windows 11 (native backdrop shows behind the sidebar and
 *   title bar), 'solid' everywhere else. Applied as `html.has-mica`.
 * - maximized / focused: drive the caption buttons and inactive-window styling.
 *
 * The material follows the app theme automatically: whenever `html.dark` flips —
 * from the layout, the Settings page, or the system — Mica is re-tinted to match.
 */
import { invoke } from '@tauri-apps/api/core';

export type WindowMaterial = 'mica' | 'solid';

export const windowState = $state({
	material: 'solid' as WindowMaterial,
	maximized: false,
	focused: true,
	isWindows: typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent)
});

let materialEnabled = true;

async function applyMaterial() {
	const dark = document.documentElement.classList.contains('dark');
	let material: WindowMaterial = 'solid';
	try {
		material = await invoke<WindowMaterial>('window_apply_material', {
			dark,
			enabled: materialEnabled
		});
	} catch {
		material = 'solid'; // Not running inside Tauri (e.g. plain browser preview).
	}
	windowState.material = material;
	document.documentElement.classList.toggle('has-mica', material === 'mica');
}

/** Turn the Mica backdrop on/off (Settings → Appearance → Sidebar transparency). */
export function setMaterialEnabled(enabled: boolean) {
	materialEnabled = enabled;
	void applyMaterial();
}

/** Start tracking. Returns a cleanup function. Call once from the root layout. */
export function initWindowState() {
	const root = document.documentElement;
	let lastDark = root.classList.contains('dark');
	void applyMaterial();

	const observer = new MutationObserver(() => {
		const dark = root.classList.contains('dark');
		if (dark !== lastDark) {
			lastDark = dark;
			void applyMaterial();
		}
	});
	observer.observe(root, { attributes: true, attributeFilter: ['class'] });

	const setFocused = (focused: boolean) => {
		windowState.focused = focused;
		root.classList.toggle('window-inactive', !focused);
	};
	const onFocus = () => setFocused(true);
	const onBlur = () => setFocused(false);
	window.addEventListener('focus', onFocus);
	window.addEventListener('blur', onBlur);

	let unlistenResize: (() => void) | null = null;
	let disposed = false;
	void (async () => {
		try {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			const appWindow = getCurrentWindow();
			const refresh = async () => {
				windowState.maximized = await appWindow.isMaximized();
			};
			await refresh();
			const unlisten = await appWindow.onResized(() => void refresh());
			if (disposed) unlisten();
			else unlistenResize = unlisten;
		} catch {
			// Outside Tauri: leave defaults.
		}
	})();

	return () => {
		disposed = true;
		observer.disconnect();
		window.removeEventListener('focus', onFocus);
		window.removeEventListener('blur', onBlur);
		unlistenResize?.();
	};
}
