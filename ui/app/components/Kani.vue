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

// Changed data for DataTable.
const dataChanged = ref<VecMergeHashKaniList>([]);
function valueChange(v: VecMergeHashKaniList) {
  dataChanged.value = v;
}

// stats
type Counts = { total: number, selected_total: number, standard: number, contract: number };
const counts = computed<Counts>(() => ({
  total: raw.value.length,
  selected_total: dataChanged.value.length,
  standard: dataChanged.value.filter(ele => ele.proof_kind === ProofKind.Standard).length,
  contract: dataChanged.value.filter(ele => ele.proof_kind === ProofKind.Contract).length
}));

// fitler rows
const filters = ref(FILTERS.filters);

// module names
type ModName = { name: string, n: number };
const selectedMods = ref<string[]>([]);
const mod_names = computed<ModName[]>(() => {
  return Object.entries(
    raw.value.reduce((acc, { harness }) => {
      const prefix = harness.split("::")[0] ?? harness;
      acc[prefix] = (acc[prefix] || 0) + 1;
      return acc;
    }, {} as { [key: string]: number })
  )
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, n]) => ({ name, n }));
});

// Real data for DataTable: outer sorting.
const data = ref<VecMergeHashKaniList>([]);
const selectedProofKind = ref<string[]>([]);
watch([selectedMods, selectedProofKind], ([mods, proofs]) => {
  const empty_mod = mods.length === 0;
  const empty_proof = proofs.length === 0;

  if (empty_mod && empty_proof) { data.value = raw.value; return; }

  const set_proof = new Set(proofs);
  let v: VecMergeHashKaniList = [];

  for (const val of raw.value) {
    // consider proof kind
    let push = empty_proof;
    push = push || ((val.proof_kind && !empty_proof && set_proof.has(val.proof_kind)) ?? false);
    if (!push) continue;

    // consider func mod
    push = empty_mod;
    for (const name of mods) {
      if (val.harness.startsWith(`${name}::`)) { push = true; break; }
    }
    if (push) v.push(val);
  }
  data.value = v;
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
        <div class="flex justify-between items-center gap-2">
          <span class="font-bold">Module:</span>
          <MultiSelect v-model="selectedMods" :options="mod_names" :maxSelectedLabels="3" placeholder="select modules"
            optionLabel="name" optionValue="name" filter>
            <template #option="{ option }">
              <span class="inline-block w-10 text-right rounded-lg px-1 my-2" :style="{ background: color.green }">
                {{ option.n }}
              </span>
              {{ option.name }}
            </template>
          </MultiSelect>

          <span class="font-bold">Proof Kind:</span>
          <SelectButton v-model="selectedProofKind" :options="optionsProofKind" :option-label="x => x" multiple :pt="{
            pcToggleButton: {
              content: (opts: SelectButtonPassThroughMethodOptions) => ({
                style: {
                  background: opts.context.active ? color.green : 'transparent',
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

:deep(.p-togglebutton:hover) {
  background: var(--p-emerald-300) !important;
}
</style>
