/**
 * Shared chart styling (DOIT.md §22–§24).
 *
 *   Title 14/600 · description 12 (DataPanel) · axis labels 11–12 · near-invisible grid
 *   Primary series = accent · secondary series = neutral gray · tooltip = elevated surface
 *   One accent, varied by opacity. No per-game colors, no gradients, no pies.
 *
 * Every value is read from CSS tokens at build time, so charts follow the theme.
 */
import type { EChartsOption } from 'echarts';
import { formatAxisDuration, formatPlaytime } from '$lib/format';

export type ChartTokens = {
	accent: string;
	neutral: string;
	grid: string;
	axisLine: string;
	textPrimary: string;
	textSecondary: string;
	textTertiary: string;
	surface: string;
	elevated: string;
	border: string;
	fontFamily: string;
	heat: string[];
	animate: boolean;
	dark: boolean;
};

function cssVar(styles: CSSStyleDeclaration, name: string) {
	return styles.getPropertyValue(name).trim();
}

export function readChartTokens(): ChartTokens {
	const root = document.documentElement;
	const styles = getComputedStyle(root);
	const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	return {
		accent: cssVar(styles, '--chart-accent'),
		neutral: cssVar(styles, '--chart-neutral'),
		grid: cssVar(styles, '--chart-grid'),
		axisLine: cssVar(styles, '--border-normal'),
		textPrimary: cssVar(styles, '--text-primary'),
		textSecondary: cssVar(styles, '--text-secondary'),
		textTertiary: cssVar(styles, '--text-tertiary'),
		surface: cssVar(styles, '--bg-surface'),
		elevated: cssVar(styles, '--bg-elevated'),
		border: cssVar(styles, '--border-normal'),
		fontFamily: getComputedStyle(document.body).fontFamily,
		heat: [0, 1, 2, 3, 4, 5].map((level) => cssVar(styles, `--heat-${level}`)),
		animate: !reduced && !root.classList.contains('animations-disabled'),
		dark: root.classList.contains('dark')
	};
}

/** Accent at a given opacity, for hover states and secondary emphasis. */
export function withAlpha(color: string, alpha: number) {
	// Canvas (ECharts) cannot parse color-mix(), so convert hex → rgba.
	const hex = color.replace('#', '');
	if (!/^[0-9a-f]{6}$/i.test(hex)) return color;
	const [r, g, b] = [0, 2, 4].map((i) => parseInt(hex.slice(i, i + 2), 16));
	return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/** Tooltip shell — elevated surface, hairline border, 8px radius. */
export function tooltipBase(tk: ChartTokens) {
	return {
		backgroundColor: tk.elevated,
		borderColor: tk.border,
		borderWidth: 1,
		padding: [6, 10],
		textStyle: { color: tk.textPrimary, fontSize: 12, fontFamily: tk.fontFamily },
		extraCssText: `border-radius:8px;backdrop-filter:blur(16px);box-shadow:${
			tk.dark ? '0 8px 24px rgba(0,0,0,.45)' : '0 6px 20px rgba(0,0,0,.08)'
		};`
	};
}

/** Tooltip row: "<label>  <b>value</b>" with tabular numbers. */
export function tooltipRow(label: string, value: string, swatch?: string) {
	const dot = swatch
		? `<span style="display:inline-block;width:8px;height:8px;border-radius:2px;margin-right:6px;background:${swatch}"></span>`
		: '';
	return `<div style="display:flex;justify-content:space-between;gap:16px;align-items:center"><span>${dot}${label}</span><b style="font-weight:600;font-variant-numeric:tabular-nums">${value}</b></div>`;
}

export function tooltipTitle(text: string) {
	return `<div style="opacity:.7;margin-bottom:2px">${text}</div>`;
}

/** Base option every chart starts from. */
export function baseOption(tk: ChartTokens): EChartsOption {
	return {
		animation: tk.animate,
		animationDuration: tk.animate ? 240 : 0,
		animationDurationUpdate: tk.animate ? 180 : 0,
		animationEasing: 'cubicOut',
		textStyle: { fontFamily: tk.fontFamily, fontSize: 12, color: tk.textSecondary },
		grid: { left: 4, right: 4, top: 12, bottom: 4, containLabel: true },
		tooltip: { ...tooltipBase(tk), trigger: 'item', confine: true }
	};
}

const axisLabel = (tk: ChartTokens) => ({
	color: tk.textTertiary,
	fontSize: 11,
	fontFamily: tk.fontFamily
});

/** Category axis (dates, weekdays, hours). */
export function categoryAxis(tk: ChartTokens, data: string[], extra: Record<string, unknown> = {}) {
	return {
		type: 'category' as const,
		data,
		axisLine: { lineStyle: { color: tk.axisLine } },
		axisTick: { show: false },
		axisLabel: { ...axisLabel(tk), hideOverlap: true },
		...extra
	};
}

/** Duration value axis (seconds), recessive grid, short labels ("2h"). */
export function durationAxis(tk: ChartTokens, extra: Record<string, unknown> = {}) {
	return {
		type: 'value' as const,
		min: 0,
		splitNumber: 4,
		axisLine: { show: false },
		axisTick: { show: false },
		splitLine: { lineStyle: { color: tk.grid } },
		axisLabel: { ...axisLabel(tk), formatter: (value: number) => formatAxisDuration(value) },
		...extra
	};
}

const DURATION_STEPS = [60, 300, 600, 900, 1800, 3600, 7200, 10800, 14400, 21600, 43200, 86400];

/**
 * Round axis scale for durations: ticks land on 15m / 30m / 1h / 2h … instead of
 * "1.4h, 2.8h". Returns { max, interval } for an ECharts value axis.
 */
export function niceDurationScale(maxValue: number, ticks = 4) {
	if (!(maxValue > 0)) return { max: 3600, interval: 900 };
	const raw = maxValue / ticks;
	const interval = DURATION_STEPS.find((step) => step >= raw) ?? Math.ceil(raw / 86400) * 86400;
	return { max: Math.ceil(maxValue / interval) * interval, interval };
}

/** Bar mark spec: thin, 4px rounded data-end, accent fill. */
export function barSeries(tk: ChartTokens, horizontal = false, count = 30) {
	return {
		type: 'bar' as const,
		// Thin marks, but not hairlines when there are only a few bars.
		barMaxWidth: horizontal ? 14 : count <= 14 ? 28 : 18,
		barCategoryGap: '35%',
		itemStyle: {
			color: tk.accent,
			borderRadius: horizontal ? [0, 4, 4, 0] : [4, 4, 0, 0]
		},
		emphasis: { itemStyle: { color: tk.accent, opacity: 0.85 } }
	};
}

/** 2px line, markers only on hover. */
export function lineSeries(color: string) {
	return {
		type: 'line' as const,
		smooth: 0.25,
		showSymbol: false,
		symbol: 'circle',
		symbolSize: 8,
		lineStyle: { width: 2, color },
		itemStyle: { color },
		emphasis: { focus: 'none' as const }
	};
}

/** Map a value to a 0–5 heat level (0 = none). */
export function heatLevel(value: number, max: number) {
	if (value <= 0 || max <= 0) return 0;
	const ratio = value / max;
	if (ratio <= 0.15) return 1;
	if (ratio <= 0.35) return 2;
	if (ratio <= 0.6) return 3;
	if (ratio <= 0.85) return 4;
	return 5;
}

/** Trailing moving average (window over the available points). */
export function movingAverage(values: number[], window = 7) {
	return values.map((_, index) => {
		const start = Math.max(0, index - window + 1);
		const slice = values.slice(start, index + 1);
		return slice.reduce((sum, value) => sum + value, 0) / slice.length;
	});
}

export const formatDurationValue = (seconds: number) => formatPlaytime(seconds);
