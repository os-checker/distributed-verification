<script setup lang="ts">
import { ofetch } from "ofetch";
import type { SelectButtonPassThroughMethodOptions } from "primevue";
import { URL_MERGE_DIFF, MergeKaniColumns, multiSort, type VecMergeHashKaniList, FILTERS, ProofKind, optionsProofKind } from "~/shared/utils/kani";
import { useDarkStore, useStyleStore } from "~/stores/style";

// Compute absolute scrollHeight for DataTable.
const { color, viewportHeight, viewportWidth } = storeToRefs(useStyleStore());
const { fontColor } = storeToRefs(useDarkStore());

const raw = ref<VecMergeHashKaniList>([]);
// Download JSON
ofetch<VecMergeHashKaniList>(
  URL_MERGE_DIFF,
  { parseResponse: JSON.parse }
).then(val => {
  raw.value = val;
  data.value = val;
});

// Real data for DataTable.
const data = ref<VecMergeHashKaniList>([]);
// Changed data for DataTable.
const dataChanged = ref<VecMergeHashKaniList>([]);
function valueChange(v: VecMergeHashKaniList) {
  dataChanged.value = v;
}

// fitler rows
const filters = ref(FILTERS.filters);
// stats
type Counts = { total: number, selected_total: number, standard: number, contract: number };
const counts = computed<Counts>(() => ({
  total: raw.value.length,
  selected_total: dataChanged.value.length,
  standard: raw.value.filter(ele => ele.proof_kind === ProofKind.Standard).length,
  contract:
    raw.value.filter(ele => ele.proof_kind === ProofKind.Contract).length,
}));

const selectedProofKind = ref<string[]>([]);
watch(selectedProofKind, val => {
  data.value = (val.length === 0) ? raw.value :
    raw.value.filter(ele => val.find(k => k === ele.proof_kind));
});

// Set title
useHead({ title: "Verify Rust Std - Kani" });
</script>

<template>

  <DataTable :value="data" paginator :rows="5" :rowsPerPageOptions="[5, 10, 20, 50]" sortMode="multiple" removableSort
    v-model:multi-sort-meta="multiSort" stripedRows :tableStyle="{ width: `${Math.round(viewportWidth - 10)}px` }"
    tableClass="p-1" :scrollHeight="`${Math.round(viewportHeight * 0.78)}px`" v-model:filters="filters"
    :globalFilterFields="FILTERS.fields" @value-change="valueChange">

    <template #header>
      <div class="flex justify-between items-center">
        <div class="flex justify-between items-center gap-1">
          Proof Kind:
          <SelectButton v-model="selectedProofKind" :options="optionsProofKind" :option-label="x => x" multiple :pt="{
            pcToggleButton: {
              content: (opt: SelectButtonPassThroughMethodOptions) => ({
                style: {
                  background: opt.context.active ? color.green : 'transparent',
                  color: fontColor
                }
              })
            }
          }" />
        </div>
        <div>
          <IconField>
            <InputIcon>
              <i class="pi pi-search" />
            </InputIcon>
            <InputText v-model="filters.global.value" placeholder="filepath or function" />
          </IconField>
        </div>
      </div>
    </template>

    <!-- Passing ratio to maxWidth seems not working. Pass bodyStyle to wrap long paths while bodyClass not working. -->
    <Column v-for="col of MergeKaniColumns" :key="col.key" :field="col.col.field" :header="col.col.header"
      :style="{ width: col.col.width }" bodyStyle="white-space: normal; word-break: break-word"
      :sortable="col.col.sortable" :pt="col.col.pt">
    </Column>

    <template #paginatorstart>
      <div class="grid grid-cols-2 grid-rows-2 justify-items-end counts">
        <span>Total:</span>
        <span class="mx-auto">{{ counts.total }}</span>
        <span>Filtered:</span>
        <span class="mx-auto">{{ counts.selected_total }}</span>
      </div>
    </template>
    <template #paginatorend>
      <div class="grid grid-cols-2 grid-rows-2 justify-items-end counts mr-3">
        <span>Standard:</span>
        <span class="mx-auto">{{ counts.standard }}</span>
        <span>Contract:</span>
        <span class="mx-auto">{{ counts.contract }}</span>
      </div>
    </template>
  </DataTable>
</template>

<style lang="css" scoped>
.counts {
  color: #aaaaaa
}
</style>
