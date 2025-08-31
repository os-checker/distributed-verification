<script setup lang="ts">
import type { SelectButtonPassThroughMethodOptions } from "primevue";
import { download } from "~/shared/utils";
import { URL_MERGE_DIFF, MergeKaniColumns, multiSort, type VecMergeHashKaniList, type MergeHashKaniList, FILTERS, ProofKind, } from "~/shared/utils/kani";
import { get_split_json, src, URL_HASH_JSON, type HashJson, type DbFunction } from "~/shared/utils/kani-split";
import { useDarkStore, useStyleStore } from "~/stores/style";

// Set title
useHead({ title: "Verify Rust Std - Kani" });

// Compute absolute scrollHeight for DataTable.
const { color, viewportHeight, viewportWidth } = storeToRefs(useStyleStore());
const { fontColor } = storeToRefs(useDarkStore());

const raw = ref<VecMergeHashKaniList>([]);
// Download JSON
download<VecMergeHashKaniList>(URL_MERGE_DIFF)
  .then(val => {
    raw.value = val;
    data.value = val;
  });

// Changed data for DataTable.
const dataChanged = ref<VecMergeHashKaniList>([]);
function valueChange(v: VecMergeHashKaniList) {
  dataChanged.value = v;
}

// stats
type CountsProofKind = { kind: string, count: number };
type Counts = {
  total: number, total_proof: CountsProofKind[],
  selected_total: number, selected_proof: CountsProofKind[]
};
const counts = computed<Counts>(() => {
  function counts_proof_kind(v: VecMergeHashKaniList): CountsProofKind[] {
    let kinds: CountsProofKind[] = [];
    const standard = v.filter(ele => ele.proof_kind === ProofKind.Standard).length;
    if (standard !== 0) kinds.push({ kind: "Standard", count: standard })
    const contract = v.filter(ele => ele.proof_kind === ProofKind.Contract).length;
    if (contract !== 0) kinds.push({ kind: "Contract", count: contract })
    const unknown = v.filter(ele => !ele.proof_kind).length;
    if (unknown !== 0) kinds.push({ kind: "Unknown", count: unknown })
    kinds.sort((a, b) => b.count - a.count);
    return kinds
  }

  return {
    total: raw.value.length, total_proof: counts_proof_kind(raw.value),
    selected_total: dataChanged.value.length, selected_proof: counts_proof_kind(dataChanged.value)
  }
});

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
const selectedProofKind = ref<CountsProofKind[]>([]);
watch([selectedMods, selectedProofKind], ([mods, proofs]) => {
  const empty_mod = mods.length === 0;
  const push_due_to_empty_or_full_proof = proofs.length === 0 || proofs.length === 3;

  if (empty_mod && push_due_to_empty_or_full_proof) { data.value = raw.value; return; }

  const set_proof = new Set(proofs.map(p => p.kind ?? "Unknown"));
  let v: VecMergeHashKaniList = [];

  for (const val of raw.value) {
    // consider proof kind
    let push = push_due_to_empty_or_full_proof;
    push = push || set_proof.has(val.proof_kind ?? "Unknown");
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
function selectedProofKindLabel(x: CountsProofKind): string {
  const sel_count = counts.value.selected_proof.find(sel => sel.kind === x.kind)?.count ?? 0;
  return `${x.kind} (${sel_count} / ${x.count})`
}

const v_hash = ref<HashJson[]>([]);
download<HashJson[]>(URL_HASH_JSON).then(v => v_hash.value = v);

const funcHarness = ref<DbFunction>();
function funcHarnessReset() { funcHarness.value = undefined }
const funcTarget = ref<DbFunction>();
function funcTargetReset() { funcTarget.value = undefined }

const visible = ref(false);
watch(visible, val => { if (!val) { funcHarnessReset(); funcTargetReset(); } });

const selectedHarnessTag = ref("success");
const selectedHarness = ref<MergeHashKaniList | null>(null);
watch(selectedHarness, val => {
  if (val === null) {
    visible.value = false;
    funcTargetReset();
    funcTargetReset();
    selectedHarnessTag.value = "success";
    return;
  }

  visible.value = true;
  if (val.ok === false) { selectedHarnessTag.value = "danger" } else { selectedHarnessTag.value = "success" }

  const url_harness = get_split_json(v_hash.value, val.harness);
  if (url_harness) download<DbFunction>(url_harness).then(f => funcHarness.value = f).catch(funcHarnessReset);
  else funcHarnessReset();

  const url_target = get_split_json(v_hash.value, val.func.name);
  if (url_target) download<DbFunction>(url_target).then(f => funcTarget.value = f).catch(funcTargetReset);
  else funcTargetReset();
});

</script>

<template>

  <DataTable :value="data" paginator :rows="5" :rowsPerPageOptions="[5, 10, 20, 50]" sortMode="multiple" removableSort
    v-model:multi-sort-meta="multiSort" stripedRows :tableStyle="{ width: `${Math.round(viewportWidth - 10)}px` }"
    tableClass="p-1" :scrollHeight="`${Math.round(viewportHeight * 0.78)}px`" v-model:filters="filters"
    :globalFilterFields="FILTERS.fields" @value-change="valueChange" selectionMode="single"
    v-model:selection="selectedHarness">

    <template #header>
      <div class="flex justify-between items-center">
        <div class="flex justify-between items-center gap-2">
          <span class="font-bold">Module:</span>
          <MultiSelect v-model="selectedMods" :options="mod_names" :maxSelectedLabels="3" placeholder="select modules"
            optionLabel="name" optionValue="name" filter>
            <template #option="{ option }">
              <span class="inline-block w-10 text-right rounded-lg px-1 my-2"
                :style="{ background: color.green, color: 'black' }">
                {{ option.n }}
              </span>
              {{ option.name }}
            </template>
          </MultiSelect>

          <span class="font-bold">Proof Kind:</span>
          <SelectButton v-model="selectedProofKind" :options="counts.total_proof" :option-label="selectedProofKindLabel"
            multiple :pt="{
              pcToggleButton: {
                content: (opts: SelectButtonPassThroughMethodOptions) => ({
                  style: {
                    background: opts.context.active ? color.green : '#efefef',
                    color: 'black'
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
      <div> </div>
    </template>
  </DataTable>


  <Dialog v-model:visible="visible" modal header="Kani Harness" :style="{ width: '80%' }">
    <div class="space-y-4 break-all">
      <Card class="border border-green-300 card">
        <template #content>
          <div> Harness Name:
            <Tag :severity="selectedHarnessTag" :value="selectedHarness?.harness" />
          </div>
          <div> Harness File: {{ selectedHarness?.file }}</div>
          <div> Harness Hash: {{ selectedHarness?.hash }}</div>
          <div class="flex items-center gap-4">
            <div> Proof Kind:
              <Tag :severity="selectedHarnessTag"> {{ selectedHarness?.proof_kind }}</Tag>
            </div>
            <div> Total Properties: {{ selectedHarness?.props }}</div>
            <div> Execution Time:
              <Tag :severity="selectedHarnessTag"> {{ selectedHarness?.time }}ms</Tag>
            </div>
          </div>
          <CodeBlock v-if="funcHarness?.src" :code="src(funcHarness)" />
        </template>
      </Card>

      <Card class="border border-sky-300 card">
        <template #content>
          <div> Verified Function:
            <Tag severity="info" :value="selectedHarness?.func.name" />
          </div>
          <div> Function File: {{ selectedHarness?.func.file }}</div>
          <CodeBlock v-if="funcTarget?.src" :code="src(funcTarget)" />
        </template>
      </Card>
    </div>
  </Dialog>
</template>

<style lang="css" scoped>
.counts {
  color: #aaaaaa
}

:deep(.p-togglebutton:hover) {
  background: var(--p-emerald-300) !important;
}

.card {
  --p-card-body-padding: 10px;
}
</style>
