#!/usr/bin/bash

set -eou pipefail

# Usage: sqlite.sh db-path sql-path
# e.g. ./split.sh ../artifacts/artifact-libcore/core.sqlite3 split.sql ../tmp/json
db=$1
sql=$2
base=$3

echo "db=$db, sql=$sql, base=$base"

# Let parallel use bash.
export PARALLEL_SHELL=$(type -p bash)

run() {
  PREFIX=/home/runner/work/verify-rust-std/verify-rust-std/library/

  # Read all arguments and JSON from stdin through parallel pipe.
  while IFS=$'\t' read -r filename hash json; do
    # $base is unavaible in parallel run, so pass it as env var.
    json_folder="$BASE/${filename#"$PREFIX"}"
    json_file="$json_folder/$hash.json"
    echo "$json_file"
    mkdir -p "$json_folder"
    jq 'del(.[] | select(. == []))' <<<"$json" >"$json_file"
  done
}

# Let parallel call the function.
export -f run

sqlite3 -batch "$db" -separator $'\t' <"$sql" |
  parallel --colsep '\t' --pipe --progress --tagstring '[Job {#}]' \
    "BASE=$base run"
