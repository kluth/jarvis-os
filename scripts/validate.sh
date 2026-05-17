#!/usr/bin/env bash

# JARVIS OS - Local Validation Script
# This script ensures that the codebase meets all quality standards before pushing.

set -e

echo "Starting JARVIS OS validation..."

# 1. Format Check
echo "Checking formatting..."
cargo fmt -- --check

# 2. Compilation Check
echo "Checking compilation for custom target (default features)..."
cargo check

echo "Checking compilation for custom target (all features)..."
cargo check --all-features

# 3. Linting
echo "Running Clippy (default features)..."
cargo clippy -- -D warnings

echo "Running Clippy (all features)..."
cargo clippy --all-features -- -D warnings

echo "Validation successful! All checks passed (Build skipped due to local resource constraints)."
