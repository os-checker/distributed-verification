<script setup lang="ts">
import * as d3 from "d3";

onMounted(() => {
  // --- 1. 數據準備 ---
  const categories = ["類別 A", "類別 B", "類別 C", "類別 D", "類別 E"];

  type BarDatum = { category: string, group1: number, group2: number, group3: number };
  const stackedBarData: BarDatum[] = [
    { category: "類別 A", group1: 10, group2: 20, group3: 15 },
    { category: "類別 B", group1: 15, group2: 25, group3: 10 },
    { category: "類別 C", group1: 20, group2: 10, group3: 30 },
    { category: "類別 D", group1: 5, group2: 15, group3: 25 },
    { category: "類別 E", group1: 25, group2: 5, group3: 20 },
  ];

  type ViolinDatum = { category: string, value: number };
  const violinData: ViolinDatum[] = [];
  categories.forEach(cat => {
    const numPoints = 100;
    const mean = Math.random() * 20 + 10;
    const stdDev = Math.random() * 3 + 2;
    for (let i = 0; i < numPoints; i++) {
      let u1 = Math.random();
      let u2 = Math.random();
      let z = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2.0 * Math.PI * u2);
      violinData.push({ category: cat, value: Math.max(0, z * stdDev + mean) });
    }
  });

  // --- 2. 圖表尺寸設定 ---
  const margin = { top: 50, right: 30, bottom: 30, left: 30 };
  const width = 900 - margin.left - margin.right;
  const height = 500 - margin.top - margin.bottom;
  const yAxisWidth = 100;
  const subplotWidth = (width - yAxisWidth) / 2;

  // --- 3. SVG 容器 ---
  const svg = d3.select("#chart-container")
    .append("svg")
    .attr("width", width + margin.left + margin.right)
    .attr("height", height + margin.top + margin.bottom)
    .append("g")

    .attr("transform", `translate(${margin.left},${margin.top})`);

  // --- 4. 共享 Y 軸 ---
  const y = d3.scaleBand()
    .domain(categories)
    .range([0, height])
    .padding(0.15);

  const yAxis = svg.append("g")
    .attr("class", "y-axis-label")
    .attr("transform", `translate(${subplotWidth}, 0)`)
    .call(d3.axisRight(y).tickSize(0));

  yAxis.select(".domain").remove();
  yAxis.selectAll("text").attr("x", yAxisWidth / 2);

  // --- 5. 左側：小提琴圖 ---
  const leftSvg = svg.append("g");

  // 為左側小提琴圖建立 X 軸
  // domain 保持不變，但 range 反轉，使其向左延伸
  const xViolinLeft = d3.scaleLinear()
    .domain([0, d3.max(violinData, d => d.value)! * 1.1])
    .range([subplotWidth, 0]); // <-- 注意 range 是反轉的

  leftSvg.append("g")
    .call(d3.axisTop(xViolinLeft).ticks(5));

  // 計算密度/直方圖的邏輯保持不變
  const histogram = d3.bin<ViolinDatum, number>()
    .domain(xViolinLeft.domain() as [number, number]) // 使用新 X 軸的 domain
    .thresholds(xViolinLeft.ticks(20))
    .value((d: ViolinDatum) => d.value);

  const sumstat = d3.group(violinData, d => d.category);

  let maxNum = 0;
  for (const category of categories) {
    const currentNum = d3.max(histogram(sumstat.get(category)!), d => d.length)!;
    if (currentNum > maxNum) { maxNum = currentNum; }
  }

  const yViolin = d3.scaleLinear()
    .range([0, y.bandwidth() / 2])
    .domain([0, maxNum]);

  // 繪製向左的小提琴圖
  leftSvg.selectAll("g.violin")
    .data(categories)
    .join("g")
    .attr("class", "violin")
    .attr("transform", d => `translate(0, ${y(d)! + y.bandwidth() / 2})`)
    .append("path")
    .datum(d => histogram(sumstat.get(d)!))
    .style("stroke", "none")
    .style("fill", "#FFC300") // 亮黄色
    //@ts-ignore: attr's type is restricted for Area 
    .attr("d", d3.area()
      //@ts-ignore: d is of type [...ViolinDatum[], x0, x1] here
      .x(d => xViolinLeft((d.x0 + d.x1) / 2)) // <-- 使用新的 xViolinLeft 比例尺
      .y0(d => -yViolin(d.length))
      .y1(d => yViolin(d.length))
      .curve(d3.curveCatmullRom)
    );

  // --- 6. 右側：堆疊柱狀圖 ---
  const rightSvg = svg.append("g")
    .attr("transform", `translate(${subplotWidth + yAxisWidth}, 0)`);

  // 為右側柱狀圖建立 X 軸
  const keys = ["group1", "group2", "group3"];
  const stack = d3.stack<BarDatum>().keys(keys);
  const stackedData = stack(stackedBarData);

  // range 是常規的 [0, subplotWidth]，使其向右延伸
  const xBarRight = d3.scaleLinear()
    .domain([0, d3.max(stackedData, layer => d3.max(layer, d => d[1]))! * 1.1])
    .range([0, subplotWidth]); // <-- 注意 range 是常規的

  rightSvg.append("g")
    .call(d3.axisTop(xBarRight).ticks(5));

  const color = d3.scaleOrdinal()
    .domain(keys)
    .range(['#66c2a5', '#fc8d62', '#8da0cb']);

  // 建立包含每個堆疊層的群組
  const barGroups = rightSvg.append("g")
    .selectAll("g")
    .data(stackedData)
    .join("g")
    .attr("fill", d => color(d.key) as any);

  // 繪製向右延伸的矩形
  barGroups.selectAll("rect")
    .data(d => d)
    .join("rect")
    .attr("y", d => y(d.data.category as any)!)
    .attr("x", d => xBarRight(d[0])) // <-- x 從 d[0] 開始
    .attr("width", d => xBarRight(d[1]) - xBarRight(d[0])) // <-- 寬度是 d[1] 和 d[0] 的差值
    .attr("height", y.bandwidth());

  // 添加文字標籤
  barGroups.selectAll("text")
    .data(d => d)
    .join("text")
    .attr("x", d => xBarRight(d[1]) - 5) // <-- x 位置基於 d[0]
    .attr("y", d => y(d.data.category as any)! + y.bandwidth() / 2)
    .attr("text-anchor", "end")

    .attr("fill", "white")
    .attr("font-size", "11px")
    .attr("font-weight", "bold")
    .attr("dominant-baseline", "middle")
    .text(d => {
      const value = d[1] - d[0];
      const segmentWidth = xBarRight(d[1]) - xBarRight(d[0]); // <-- 使用新的比例尺計算寬度
      if (value > 0 && segmentWidth > 20) {
        return value;
      }
      return "";
    });
})


</script>

<template>
  <div id="chart-container"></div>
</template>

<style lang="css" scoped>
#chart-container {
  background-color: #fff;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
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
