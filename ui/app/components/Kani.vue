<script setup lang="ts">
import { ofetch } from "ofetch";
import { URL_MERGE_DIFF, MergeKaniColumns, multiSort, type VecMergeHashKaniList } from "~/shared/utils/merged_list";

const vec = ref<VecMergeHashKaniList>([]);
// Download JSON
ofetch<VecMergeHashKaniList>(
  URL_MERGE_DIFF,
  { parseResponse: JSON.parse }
).then(val => vec.value = val);

// Set title
useHead({ title: "Verify Rust Std - Kani" });
</script>

<template>
  <div>len = {{ vec.length }}</div>

  <DataTable :value="vec" paginator :rows="5" :rowsPerPageOptions="[5, 10, 20, 50]" sortMode="multiple" removableSort
    sortField="proof_kind" :sort-order="1" v-model:multi-sort-meta="multiSort" tableStyle="min-width: 50rem">
    <Column v-for="col of MergeKaniColumns" :key="col.key" :field="col.col.field" :header="col.col.header"
      :style="{ width: col.col.width }" :sortable="col.col.sortable"></Column>
  </DataTable>
</template>
