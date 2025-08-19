#!/usr/bin/bash

set -eou pipefail

# Usage: sqlite.sh db-path sql-path
db=$1
sql=$2
base=$3

echo "db=$db, sql=$sql, base=$base"

PREFIX=/home/runner/work/verify-rust-std/verify-rust-std/library/

sqlite3 -batch "$db" -separator $'\t' <"$sql" |
  while IFS=$'\t' read -r filename json; do
    json_folder="$base/${filename#"$PREFIX"}"
    echo "$json_folder"
    mkdir -p "$json_folder"
    jq 'map(del(.[] | select(. == [])))' <<<"$json" >"$json_folder/DbFunctions.json"
  done
