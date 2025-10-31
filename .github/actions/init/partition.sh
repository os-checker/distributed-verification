#!/bin/bash

set -eoux pipefail

# Flatten nested harnesses and deduplicate.
jq '
. as $root
| ($root["standard-harnesses"] | to_entries | map(.value) | add) +
  ($root["contract-harnesses"] | to_entries | map(.value) | add) +
  ($root["contracts"] | map(.harnesses) | add)
| unique
| sort
' kani-list.json >kani-list_arr.json

N=8
i=1
OUT_DIR=parition
mkdir $OUT_DIR -p

# Split the array to $N batches, with length as much close as possible.
# The order is not guaranteed: consecutive harnesses are split into
# differenct groups. Say [1,2,3,4] is partitioned into `[1,3]` and
# `[2,4]` with N=2.
jq -c --argjson N $N '
. as $in | [range(length)] | group_by(. % $N) | map(map($in[.])) | .[]
' kani-list_arr.json | while read -r line; do
  echo "$line" >"$OUT_DIR/${i}.json"
  i=$((i + 1))
done
