#!/usr/bin/bash

# FIXME: currently there is no old hash.json.
verify_rust_std diff --old hash-original.json --new hash.json | tee diff_raw.json

jq --slurp '
{ disable: .[0] | map({(.): true}) | add, diff: .[1] | map(.func) } as $root
| $root.diff | map(select($root.disable[.] != true))
' .github/disable/harness.json diff_raw.json >diff.json

cat diff.json
