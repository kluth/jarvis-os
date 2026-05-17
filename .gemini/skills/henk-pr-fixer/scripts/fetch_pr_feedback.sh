#!/usr/bin/env bash

# JARVIS OS - PR Comment Fetcher (Token Efficient)
# This script extracts all review comments and general comments from open/closed PRs.

REPO="kluth/jarvis-os"

if ! [ -x "$(command -v gh)" ]; then
  echo "Error: gh CLI is not installed." >&2
  exit 1
fi

# 1. Fetch all PR numbers (closed and open)
# We limit to last 20 for context safety
PR_NUMBERS=$(gh pr list --state all --limit 20 --json number --jq '.[].number')

echo "{"
echo "  \"prs\": ["

FIRST_PR=true
for pr in $PR_NUMBERS; do
    if [ "$FIRST_PR" = true ]; then
        FIRST_PR=false
    else
        echo ","
    fi
    
    # Fetch PR title, state, and ALL comments
    # We combine review comments and general comments
    PR_DATA=$(gh pr view "$pr" --json number,title,state,comments,reviews \
        --jq '{number: .number, title: .title, state: .state, feedback: ([.comments[].body] + [.reviews[].body] + [.reviews[].comments[].body]) | map(select(. != null and . != ""))}')
    
    echo "    $PR_DATA"
done

echo "  ]"
echo "}"
