import type { DataTableSortMeta } from "primevue";

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
    col: { field: "file", header: "File Path", width: "25%", sortable: true },
  },
  {
    key: "func",
    col: { field: "func", header: "Function", width: "25%", sortable: true },
  },
  {
    key: "hash",
    col: { field: "hash", header: "Hash", width: "25%" },
  },
  {
    key: "proof_kind",
    col: { field: "proof_kind", header: "Proof Kind", width: "10%", sortable: true },
  },
];

export const multiSort: DataTableSortMeta[] = [
  // { field: "proof_kind", order: 1 },
  // { field: "func", order: 1 },
];
