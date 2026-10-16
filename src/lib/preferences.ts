/**
 * Apply a preference to the running app immediately (before it is saved).
 * The root layout listens to these events to keep the shell in sync.
 */
import type { AppLanguage, AppSettings, ThemeMode } from '$lib/settings';

export function prefersDark(mode: ThemeMode) {
	if (mode === 'dark') return true;
	if (mode === 'light') return false;
	return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function applyTheme(mode: ThemeMode) {
	const dark = prefersDark(mode);
	document.documentElement.classList.toggle('dark', dark);
	try {
		localStorage.setItem('theme', dark ? 'dark' : 'light');
	} catch {
		// Ignore.
	}
	window.dispatchEvent(new CustomEvent('pt-theme-change', { detail: { darkMode: dark } }));
}

export function applyLanguage(language: AppLanguage) {
	window.dispatchEvent(new CustomEvent('pt-language-change', { detail: { language } }));
}

export function applyAnimations(enabled: boolean) {
	document.documentElement.classList.toggle('animations-disabled', !enabled);
	window.dispatchEvent(
		new CustomEvent('pt-animations-change', { detail: { animationsEnabled: enabled } })
	);
}

/** Tell the shell that settings were saved (sidebar material, Steam status…). */
export function broadcastSettings(settings: AppSettings) {
	window.dispatchEvent(new CustomEvent('pt-settings-change', { detail: settings }));
}
