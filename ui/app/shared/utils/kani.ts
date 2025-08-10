import type { DataTableSortMeta } from "primevue";
import { FilterMatchMode } from '@primevue/core/api';

export const URL_MERGE_DIFF = "https://raw.githubusercontent.com/os-checker/verify-rust-std_data/refs/heads/main/merge_diff-proofs-only.json";

export type VecMergeHashKaniList = MergeHashKaniList[];

export interface MergeHashKaniList {
  file: string;
  func: string;
  hash?: string;
  proof_kind?: ProofKind;
}

export enum ProofKind {
  Standard = "Standard",
  Contract = "Contract"
}

export interface Column {
  field: string,
  header: string,
  width: string,
  sortable?: boolean,
}

export interface MergeKaniColumn {
  key: string,
  col: Column,
}

export const MergeKaniColumns: MergeKaniColumn[] = [
  {
    key: "file",
    col: { field: "file", header: "File Path", width: "15%", sortable: true },
  },
  {
    key: "func",
    col: { field: "func", header: "Function", width: "25%", sortable: true },
  },
  {
    key: "hash",
    col: { field: "hash", header: "Hash", width: "25%", },
  },
  {
    key: "proof_kind",
    col: { field: "proof_kind", header: "Proof Kind", width: "12%", sortable: true },
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
  fields: ["file", "func"]
};

export const optionsProofKind: string[] = [ProofKind.Standard, ProofKind.Contract];
