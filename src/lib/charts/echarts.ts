/**
 * The slice of ECharts the app actually uses, registered once.
 *
 * Importing from 'echarts' pulls in every chart type, component and renderer
 * (~1.1 MB). The charts here only need bar + line series on a grid with a
 * tooltip, drawn on canvas; the heatmaps are plain DOM. Add a chart type or
 * component here before using it in an option, or ECharts will ignore it.
 */
import { init, use } from 'echarts/core';
import { BarChart, LineChart } from 'echarts/charts';
import { GridComponent, TooltipComponent } from 'echarts/components';
import { LegacyGridContainLabel } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';

use([BarChart, LineChart, GridComponent, TooltipComponent, LegacyGridContainLabel, CanvasRenderer]);

export { init };
