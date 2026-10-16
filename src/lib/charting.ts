type EChartsLib = typeof import('./charts/echarts');

let echartsPromise: Promise<EChartsLib> | null = null;

/** Loads the trimmed ECharts build (see charts/echarts.ts) on first use. */
export function loadEcharts() {
	if (!echartsPromise) {
		echartsPromise = import('./charts/echarts');
	}

	return echartsPromise;
}

export function createDebouncedCallback(callback: () => void, delayMs = 120) {
	let timerId: number | null = null;

	const debounced = () => {
		if (timerId !== null) {
			window.clearTimeout(timerId);
		}

		timerId = window.setTimeout(() => {
			timerId = null;
			callback();
		}, delayMs);
	};

	debounced.cancel = () => {
		if (timerId !== null) {
			window.clearTimeout(timerId);
			timerId = null;
		}
	};

	return debounced;
}

export function listenToChartPreferenceChanges(callback: () => void) {
	window.addEventListener('pt-theme-change', callback);
	window.addEventListener('pt-animations-change', callback);

	return () => {
		window.removeEventListener('pt-theme-change', callback);
		window.removeEventListener('pt-animations-change', callback);
	};
}
