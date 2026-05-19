#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."
pnpm exec tsx scripts/generate-runtime-catalog-adapter-oracles.ts --check
