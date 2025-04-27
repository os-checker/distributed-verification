#!/bin/bash

set -x

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
gh run download "$LATEST_RUN_ID" -R "$REPOSITORY"
