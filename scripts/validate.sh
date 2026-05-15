#!/usr/bin/env bash

# JARVIS OS - Local Validation Script
# This script ensures that the codebase meets all quality standards before pushing.

set -e

echo "Starting JARVIS OS validation..."

# 1. Format Check
echo "Checking formatting..."
cargo fmt -- --check

# 2. Compilation Check
echo "Checking compilation for custom target..."
cargo check -Zbuild-std=core,alloc --target x86_64-jarvis_os.json -Zjson-target-spec

# 3. Linting
echo "Running Clippy..."
cargo clippy -Zbuild-std=core,alloc --target x86_64-jarvis_os.json -Zjson-target-spec -- -D warnings

echo "Validation successful! All checks passed (Build skipped due to local resource constraints)."
