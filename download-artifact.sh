#!/bin/bash

set -x

DIR=artifacts

# Define the repo and workflow file
REPOSITORY="os-checker/distributed-verification"
WORKFLOW_FILE=".github/workflows/test.yml"

# Get latest Run ID
LATEST_RUN_ID=$(gh run list -R "$REPOSITORY" --workflow "$WORKFLOW_FILE" --limit 1 --json databaseId --jq '.[0].databaseId')
echo "Download from run id $LATEST_RUN_ID"

# Check Run ID
if [ -z "$LATEST_RUN_ID" ]; then
  echo "Failed to get run id."
  exit 1
fi

# List artifacts
gh api -H "Accept: application/vnd.github.v3+json" \
  /repos/$REPOSITORY/actions/runs/$LATEST_RUN_ID/artifacts |
  jq '.artifacts[] | {name: .name, id: .id, size: .size_in_bytes}'

# Download and unzip the artifact
gh run download "$LATEST_RUN_ID" -R "$REPOSITORY" -D $DIR

# For comparison if some proofs are missing.
# jq -c "map({name, file}) | sort_by(.name) | .[]" artifacts/artifact-libcore/core.json > x64.json

function extract_harness() {
  # gh run download 15710656488 -R model-checking/verify-rust-std -D artifacts
  # cd artifacts/ubuntu-latest-results.json/
  local prefix=/home/runner/work/verify-rust-std/verify-rust-std/library/core/
  jq -c "map(
    select((.result.is_autoharness | not) and .result.file_name)
    | select(.result.file_name | startswith(\"$prefix\"))
    | {name: .result.harness, file: .result.file_name | ltrimstr(\"$prefix\")}
  )" results.json >core.json
}
