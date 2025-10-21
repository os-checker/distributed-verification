<script setup lang="ts">
import * as d3 from "d3";
import { download } from "~/shared/utils";

const { viewportWidth } = storeToRefs(useStyleStore());

type Cnt = { NotProof?: number, Standard?: number, Contract?: number, AutoStandard?: number, AutoContract?: number };
type Time = { avg: number, time: number[] }
type Datum = { mod: string, total: number, kind?: Cnt, time?: Time };
const URL = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/chart/merged.json";

const data = ref<Datum[]>([]);
download<Datum[]>(URL).then(v => {
  data.value = v
    .map(d => {
      const k = d.kind;
      // Compute NotProof
      if (k) k.NotProof =
        d.total - (k.Standard ?? 0) - (k.Contract ?? 0) - (k.AutoStandard ?? 0) - (k.AutoContract ?? 0)
      else d.kind = { NotProof: d.total };
      return d;
    })
    .sort((a, b) => {
      // proofs first
      if (a.kind === undefined && b.kind !== undefined) return 1;
      if (b.kind === undefined && a.kind !== undefined) return -1;
      // More proofs prior
      var cmp_cnt = (b.total - (b.kind?.NotProof ?? 0)) - (a.total - (a.kind?.NotProof ?? 0));
      if (cmp_cnt !== 0) return cmp_cnt;

      // avg time second: the one with time or higher time is prior
      if (a.time === undefined && b.time !== undefined) return 1;
      if (b.time === undefined && a.time !== undefined) return -1;
      const cmp_time = (b.time?.avg ?? 0) - (a.time?.avg ?? 0);
      if (cmp_time !== 0) return cmp_time;

      // total count last
      cmp_cnt += b.total - a.total;
      return cmp_cnt;
    });
});

const module_names = computed<string[]>(() => data.value.map(d => d.mod));

type ViolinDatum = { mod: string, time: number };
const violinData = computed<ViolinDatum[]>(() => {
  let v: ViolinDatum[] = [];
  for (const d of data.value) {
    if (d.time)
      for (const t of d.time?.time ?? []) {
        v.push({ mod: d.mod, time: t });
      }
    else v.push({ mod: d.mod, time: 0 });
  }
  return v;
});

type AvgTime = { mod: string, avg: number };
const avgTime = computed<AvgTime[]>(() => {
  return data.value.map(d => ({ mod: d.mod, avg: d.time?.avg ?? 0 })).filter(d => d.avg)
});

type StackedDatum = { mod: string } & Cnt;
const proof_kinds = ["NotProof", "Standard", "Contract", "AutoStandard", "AutoContract"];
const stackedBarData = computed<StackedDatum[]>(() => data.value.map(d => ({
  mod: d.mod, NotProof: d.kind?.NotProof ?? 0,
  Standard: d.kind?.Standard ?? 0, Contract: d.kind?.Contract ?? 0,
  AutoStandard: d.kind?.AutoStandard ?? 0, AutoContract: d.kind?.AutoContract ?? 0
})));

type TotalCountLabel = { mod: string, cnt: number };
const totalCount = computed<TotalCountLabel[]>(() => {
  return data.value.map(d => ({ mod: d.mod, cnt: d3.sum(Object.values(d.kind ?? {})) }))
});

function plot() {
  // Do nothing if data are nor ready.
  if (data.value.length === 0) return;
  const elemId = "#chart-container";

  // Styling
  const margin = { top: 80, right: 40, bottom: 30, left: 10 };
  const width = viewportWidth.value - 20 - margin.left - margin.right;
  const height = module_names.value.length * 40 - margin.top - margin.bottom;
  const yAxisWidth = 150;
  const widthLeftRatio = 0.25;
  const subplotWidthLeft = (width - yAxisWidth) * widthLeftRatio;
  const subplotStartRight = subplotWidthLeft + yAxisWidth;
  const subplotWidthRight = width - subplotStartRight;

  // Clear old SVG when view size changes.
  d3.select(elemId).selectAll("*").remove();

  // Set up SVG container.
  var svg = d3.select(elemId)
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
    if (currentNum > maxNum) maxNum = currentNum
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

  leftSvg.append("g").selectAll("text")
    .data(avgTime.value)
    .join("text")
    .attr("x", 0)
    .attr("y", d => y(d.mod)! + y.bandwidth() * 0.6)
    .text(d => `${(d.avg / 1000).toFixed(1)}s`)
    // Inherent font color sensitive to the theme.
    .attr("fill", "currentColor");

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

  const color = d3.scaleOrdinal(["#8DA0CB", "#FC8D62", "#FFC300", "#D7B0E3", "#750699"]).domain(proof_kinds);

  const barGroups = rightSvg.append("g")
    .selectAll("g")
    .data(stackedData)
    .join("g")
    .attr("fill", d => color(d.key));

  // Plot rectangle.
  barGroups.selectAll("rect")
    .data(d => d)
    .join("rect")
    .attr("y", d => y(d.data.mod)!)
    .attr("x", d => xBarRight(d[0])) // <-- x 從 d[0] 開始
    .attr("width", d => xBarRight(d[1]) - xBarRight(d[0])) // <-- 寬度是 d[1] 和 d[0] 的差值
    .attr("height", y.bandwidth());

  // Add value annotation inside the rectangle.
  const yLabelAdjust = 0.64;
  barGroups.selectAll("text")
    .data(d => d)
    .join("text")
    .attr("x", d => xBarRight(d[1]) - 4) // <-- x 位置基於 d[0]
    .attr("y", d => y(d.data.mod)! + y.bandwidth() * yLabelAdjust)
    .attr("text-anchor", "end")
    .attr("fill", "white")
    .attr("font-size", "11px")
    .attr("font-weight", "bold")
    // .attr("dominant-baseline", "middle")
    .text(d => {
      const value = d[1] - d[0];
      const segmentWidth = xBarRight(d[1]) - xBarRight(d[0]); // <-- 使用新的比例尺計算寬度
      if (value > 0 && segmentWidth > 30) {
        return value;
      }
      return "";
    });

  // Add total count value next to the rectangle.
  rightSvg.append("g").selectAll("text")
    .data(totalCount.value)
    .join("text")
    .attr("x", d => xBarRight(d.cnt) + 4)
    .attr("y", d => y(d.mod)! + y.bandwidth() * yLabelAdjust)
    .text(d => `(${d.cnt.toString()})`)
    // Inherent font color sensitive to the theme.
    .attr("fill", "currentColor");

  svg = d3.select("svg");

  // Set font-size.
  svg.selectAll("text").attr("font-size", 16);

  // Styles for legends & titles.
  const legendY = 10; // Move legend down by 10px.
  const legendYText = 15; // Move legend text down by 15px.
  const titleY = legendY + legendYText; // Move title text down.

  // Right plot title.
  svg.append("text")
    .text("Average Verification Time and Distribution (unit: ms)")
    .style("font-weight", "bold")
    .attr("fill", "currentColor")
    .attr("x", 5)
    .attr("y", titleY);

  // Right plot title.
  svg.append("text")
    .text("Count of Kani Harnesses over Proof Kinds")
    .style("font-weight", "bold")
    .attr("fill", "currentColor")
    .attr("x", subplotStartRight + 5)
    .attr("y", titleY);

  // Right plot legend.
  const legendRectWidth = 20;
  const legendStartX = width - 550;
  const legend = svg.append("g")
    .attr('transform', `translate(${legendStartX},${legendY})`)
    .selectAll("g")
    .data(proof_kinds)
    .enter().append('g');
  legend.append('rect')
    .attr('width', legendRectWidth)
    .attr('height', legendRectWidth)
    .attr('fill', d => color(d));
  legend.append("text")
    .text(d => d)
    .attr("x", legendRectWidth + 4)
    .attr("y", legendY + legendRectWidth * 0.35)
    .attr("fill", "currentColor");

  const gap = 5;
  var offsetX = 0;
  legend.each(function () {
    const g = d3.select(this)!;
    const bbox = g.node()!.getBBox(); // 整组实际占宽
    g.attr("transform", `translate(${offsetX}, 0)`);
    offsetX += bbox.width + gap;     // 下一项起点
  })
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
