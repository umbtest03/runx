#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."
pnpm exec tsx scripts/generate-tool-catalog-oracles.ts --check
