import type { DataTableSortMeta } from "primevue";
import { FilterMatchMode } from '@primevue/core/api';

export const URL_MERGE_DIFF = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/merge_results-core.json";

export type VecMergeHashKaniList = MergeHashKaniList[];

export interface MergeHashKaniList {
  crate: string,
  file: string,
  harness: string,
  proof_kind?: ProofKind,
  hash?: string,
  time?: number,
  props: number,
  func: {
    name: string,
    safe: string,
    file: String,
  },
  ok?: boolean,
  n_fialed_properties?: number,
}

export enum ProofKind {
  Standard = "Standard",
  Contract = "Contract",
  AutoStandard = "AutoStandard",
  AutoContract = "AutoContract",
  // Placeholder for null proof_kind
  Unknown = "Unknown",
}

export interface Column {
  field: string,
  header: string,
  width: string,
  sortable?: boolean,
  pt?: any
}

export interface MergeKaniColumn {
  key: string,
  col: Column,
}

/** Make header and column content center aligned. */
export const center = {
  columnHeaderContent: { style: { "justify-content": "center" } },
  bodyCell: { style: { "text-align": "center" } }
};
export const right = {
  columnHeaderContent: { style: { "justify-content": "flex-end" } },
  bodyCell: { style: { "text-align": "right" } }
};

export const MergeKaniColumns: MergeKaniColumn[] = [
  {
    key: "crate",
    col: { field: "crate", header: "Crate", width: "5%", sortable: true },
  },
  {
    key: "file",
    col: { field: "file", header: "File Path", width: "15%", sortable: true },
  },
  {
    key: "harness",
    col: { field: "harness", header: "Harness", width: "25%", sortable: true },
  },
  {
    key: "hash",
    col: { field: "hash", header: "Hash", width: "25%", },
  },
  {
    key: "proof_kind",
    col: { field: "proof_kind", header: "Proof Kind", width: "12%", sortable: true, pt: center },
  },
  {
    key: "time",
    col: { field: "time", header: "Time (ms)", width: "5%", sortable: true, pt: right },
  },
];

export const multiSort: DataTableSortMeta[] = [
  // { field: "proof_kind", order: 1 },
  // { field: "func", order: 1 },
];

export const FILTERS = {
  filters: {
    global: { value: null, matchMode: FilterMatchMode.CONTAINS },
  },
  fields: ["file", "harness"]
};

