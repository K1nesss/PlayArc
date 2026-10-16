import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	build: {
		// The largest chunk is ECharts (~510 kB), already trimmed to the parts we use
		// (src/lib/charts/echarts.ts) and loaded only when a chart first renders. It's
		// read from disk inside the app, not downloaded, so Vite's 500 kB web default
		// doesn't apply. Anything else growing past 600 kB should still warn.
		chunkSizeWarningLimit: 600
	}
});
