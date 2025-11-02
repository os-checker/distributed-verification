#!/bin/bash

set -eoux pipefail

# Generate assets/json
./.github/gen_core.json.sh

# Merge non-auto proofs.
jq --slurp '
.[0] + .[1]
| map(select(.proof_kind) | .name)
| unique
| sort
| map(select(endswith("kani::internal::automatic_harness") | not))
' assets/json/core.json assets/json/alloc.json >assets/kani-list_arr.json
