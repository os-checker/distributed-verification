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
OUT_DIR=partition
mkdir $OUT_DIR -p

# Split the array to $N batches, with length as much close as possible.
# Consecutive harnesses are split into the same group. Say [1,2,3,4] is
# partitioned into `[1,2]` and `[3,4]` with N=2.
jq -c --argjson N $N '
  . as $in
  | length as $L
  | ($L / $N | floor) as $base_size
  | ($L % $N) as $remainder
  | ([range($remainder) | $base_size + 1] + [range($N - $remainder) | $base_size]) as $sizes
  | reduce $sizes[] as $size (
      {slices: [], offset: 0};
      .slices += [$in[.offset : .offset + $size]] |
      .offset += $size
    )
  | .slices
  | .[]
' kani-list_arr.json | while read -r line; do
  echo "$line" >"$OUT_DIR/${i}.json"
  i=$((i + 1))
done
