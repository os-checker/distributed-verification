#!/usr/bin/bash

set -eou pipefail

# Usage: sqlite.sh db-path sql-path
# e.g. ./split.sh ../artifacts/artifact-libcore/core.sqlite3 split.sql ../tmp/json
db=$1
sql=$2
base=$3

echo "db=$db, sql=$sql, base=$base"

PREFIX=/home/runner/work/verify-rust-std/verify-rust-std/library/

sqlite3 -batch "$db" -separator $'\t' <"$sql" |
  while IFS=$'\t' read -r filename hash json; do
    json_folder="$base/${filename#"$PREFIX"}"
    json_file="$json_folder/$hash.json"
    echo "$json_file"
    mkdir -p "$json_folder"
    jq 'del(.[] | select(. == []))' <<<"$json" >"$json_file"
  done
