import type { AppLanguage } from '$lib/settings';

/**
 * App-wide language state. The root layout keeps this in sync with settings and the
 * `pt-language-change` event, so components can call `t()` without threading a
 * `language` prop through every level.
 */
export const locale = $state<{ current: AppLanguage }>({ current: 'zh-CN' });

export function setLanguage(language: AppLanguage) {
	locale.current = language;
	if (typeof document !== 'undefined') {
		document.documentElement.lang = language;
	}
}

export function isZh(language: AppLanguage = locale.current) {
	return language === 'zh-CN';
}

/** Inline bilingual string. Reactive when read inside a template or $derived. */
export function t(zh: string, en: string, language: AppLanguage = locale.current) {
	return language === 'zh-CN' ? zh : en;
}
