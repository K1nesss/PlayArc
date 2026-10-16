import { convertFileSrc } from '@tauri-apps/api/core';

/** Local artwork path → URL the webview can load. Passes through data:/http(s) URLs. */
export function assetSrc(path: string | null | undefined): string | null {
	if (!path) return null;
	if (/^(data:|https?:|asset:|blob:)/.test(path)) return path;
	try {
		return convertFileSrc(path);
	} catch {
		return null;
	}
}
