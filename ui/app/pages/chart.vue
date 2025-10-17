<script setup lang="ts">
import * as d3 from "d3";
import { download } from "~/shared/utils";

const { viewportWidth } = storeToRefs(useStyleStore());

type Cnt = { not_proof: number, standard: number, contract: number };
type Datum = { mod: string, cnt: Cnt, avg?: number, time?: number[] };
const URL = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/chart/merged.json";

const data = ref<Datum[]>([]);
download<Datum[]>(URL).then(v => {
  data.value = v.sort((a, b) => {
    // proofs first
    var cmp_cnt = b.cnt.standard + b.cnt.contract - a.cnt.standard - a.cnt.contract;
    if (cmp_cnt !== 0) return cmp_cnt;

    // avg time second: the one with time or higher time is prior
    if (a.avg === undefined && b.avg !== undefined) return 1;
    if (b.avg === undefined && a.avg !== undefined) return -1;
    const cmp_time = (b.avg ?? 0) - (a.avg ?? 0);
    if (cmp_time !== 0) return cmp_time;

    // total count last
    cmp_cnt += b.cnt.not_proof - a.cnt.not_proof;
    return cmp_cnt;
  });
});

const module_names = computed<string[]>(() => data.value.map(d => d.mod));

type ViolinDatum = { mod: string, time: number };
const violinData = computed<ViolinDatum[]>(() => {
  let v: ViolinDatum[] = [];
  for (const d of data.value) {
    if (d.time)
      for (const t of d.time ?? []) {
        v.push({ mod: d.mod, time: t });
      }
    else v.push({ mod: d.mod, time: 0 });
  }
  return v;
});

type StackedDatum = { mod: string } & Cnt;
const proof_kinds = ["not_proof", "standard", "contract"];
const stackedBarData = computed<StackedDatum[]>(() => {
  return data.value.map(d => ({
    mod: d.mod, not_proof: d.cnt.not_proof, standard: d.cnt.standard, contract: d.cnt.contract
  }));
});

function plot() {
  // Do nothing if data are nor ready.
  if (data.value.length === 0) return;
  const elemId = "#chart-container";

  // Styling
  const margin = { top: 30, right: 40, bottom: 30, left: 10 };
  const width = viewportWidth.value - margin.left - margin.right;
  const height = module_names.value.length * 40 - margin.top - margin.bottom;
  const yAxisWidth = 150;
  const widthLeftRatio = 0.25;
  const subplotWidthLeft = (width - yAxisWidth) * widthLeftRatio;
  const subplotStartRight = subplotWidthLeft + yAxisWidth;
  const subplotWidthRight = width - subplotStartRight;

  // Clear old SVG when view size changes.
  d3.select(elemId).selectAll("*").remove();

  // Set up SVG container.
  const svg = d3.select(elemId)
    .append("svg")
    .attr("width", width + margin.left + margin.right)
    .attr("height", height + margin.top + margin.bottom)
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  // Share y-axis.
  const y = d3.scaleBand()
    .domain(module_names.value)
    .range([0, height])
    .padding(0.15);

  const yAxis = svg.append("g")
    .attr("class", "y-axis-label")
    .attr("transform", `translate(${subplotWidthLeft}, 0)`)
    .call(d3.axisRight(y).tickSize(0));

  yAxis.select(".domain").remove();
  yAxis.selectAll("text").attr("x", yAxisWidth * 0.1);

  // Left side: violin plot
  const leftSvg = svg.append("g");

  const xViolinLeft = d3.scaleLinear()
    // Deternmin the range of x-axis.
    .domain([0, d3.max(violinData.value, d => d.time)! * 1.05])
    // NOTE: invert range, i.e. stretching to left with x-axis increasing
    .range([subplotWidthLeft, 0]);

  leftSvg.append("g")
    .call(d3.axisTop(xViolinLeft).ticks(3));

  // Compute density.
  const histogram = d3.bin<ViolinDatum, number>()
    .domain(xViolinLeft.domain() as [number, number])
    .thresholds(xViolinLeft.ticks(10))
    .value((d: ViolinDatum) => d.time);

  const sumstat = d3.group(violinData.value, d => d.mod);

  // Determine the maximum of violin height.
  let maxNum = 0;
  for (const mod of module_names.value) {
    const currentNum = d3.max(histogram(sumstat.get(mod)!), d => d.length)!;
    if (currentNum > maxNum) { maxNum = currentNum; }
  }

  const yViolin = d3.scaleLinear()
    .range([0, y.bandwidth() / 2])
    .domain([0, maxNum]);

  // Plot violin.
  leftSvg.selectAll("g.violin")
    .data(module_names.value)
    .join("g")
    .attr("class", "violin")
    .attr("transform", d => `translate(0, ${y(d)! + y.bandwidth() / 2})`)
    .append("path")
    .datum(d => histogram(sumstat.get(d)!))
    .style("stroke", "none")
    .style("fill", "#66c2a5")
    //@ts-ignore: attr's type is restricted for Area 
    .attr("d", d3.area()
      //@ts-ignore: d is of type [...ViolinDatum[], x0, x1] here
      .x(d => xViolinLeft((d.x0 + d.x1) / 2)) // <-- 使用新的 xViolinLeft 比例尺
      .y0(d => -yViolin(d.length))
      .y1(d => yViolin(d.length))
      .curve(d3.curveCatmullRom)
    );

  // Right side: stacked bar plot
  const rightSvg = svg.append("g")
    .attr("transform", `translate(${subplotStartRight}, 0)`);

  const stack = d3.stack<StackedDatum>().keys(proof_kinds);
  const stackedData = stack(stackedBarData.value);

  const xBarRight = d3.scaleLinear()
    .domain([0, d3.max(stackedData, layer => d3.max(layer, d => d[1]))! * 1.1])
    .range([0, subplotWidthRight]); // Normal range: left-to-right

  rightSvg.append("g")
    .call(d3.axisTop(xBarRight).ticks(5));

  const color = d3.scaleOrdinal()
    .domain(proof_kinds)
    .range(["#8da0cb", "#fc8d62", "#FFC300"]);

  const barGroups = rightSvg.append("g")
    .selectAll("g")
    .data(stackedData)
    .join("g")
    .attr("fill", d => color(d.key) as any);

  // Plot rectangle.
  barGroups.selectAll("rect")
    .data(d => d)
    .join("rect")
    .attr("y", d => y(d.data.mod)!)
    .attr("x", d => xBarRight(d[0])) // <-- x 從 d[0] 開始
    .attr("width", d => xBarRight(d[1]) - xBarRight(d[0])) // <-- 寬度是 d[1] 和 d[0] 的差值
    .attr("height", y.bandwidth());

  // Add value annotation inside the rectangle.
  barGroups.selectAll("text")
    .data(d => d)
    .join("text")
    .attr("x", d => xBarRight(d[1]) - 4) // <-- x 位置基於 d[0]
    .attr("y", d => y(d.data.mod)! + y.bandwidth() / 2)
    .attr("text-anchor", "end")
    .attr("fill", "white")
    .attr("font-size", "11px")
    .attr("font-weight", "bold")
    .attr("dominant-baseline", "middle")
    .text(d => {
      const value = d[1] - d[0];
      const segmentWidth = xBarRight(d[1]) - xBarRight(d[0]); // <-- 使用新的比例尺計算寬度
      if (value > 0 && segmentWidth > 30) {
        return value;
      }
      return "";
    });

  svg.selectAll('text').attr('font-size', 16);
}

watch(data, plot);
watch(viewportWidth, plot);
onMounted(plot);
</script>

<template>
  <div id="chart-container"></div>
</template>

<style lang="css" scoped>
#chart-container {
  /* background-color: #fff; */
  padding: 10px;
  overflow: auto;
  height: 100vh;
}

/* D3.js 座標軸樣式 */
.tick line {
  stroke: #e0e0e0;
  stroke-dasharray: 2, 2;
}

.tick text {
  fill: #555;
  font-size: 12px;
}

.domain {
  /* 移除座標軸的主線條，讓風格更簡潔 */
  stroke: none;
}

.y-axis-label text {
  text-anchor: middle;
  font-weight: bold;
  fill: #444;
}
</style>
