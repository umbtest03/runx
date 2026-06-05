#!/usr/bin/env sh
# Public-provider HTTP/OpenAPI proof: National Weather Service points endpoint.
#
# This executes a real api.weather.gov call through Runx's governed HTTP front
# and validates stable response shape in the graph checkpoint. It avoids
# assertions on forecast prose, which is intentionally live and volatile.
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
OSS="$(cd "$HERE/../.." && pwd)"
RUNX="${RUNX_BIN:-$OSS/crates/target/debug/runx}"
[ -x "$RUNX" ] || RUNX="$(command -v runx || true)"
[ -n "$RUNX" ] || { echo "runx binary not found; set RUNX_BIN." >&2; exit 1; }

export RUNX_RECEIPT_SIGN_KID="${RUNX_RECEIPT_SIGN_KID:-runx-demo-key}"
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64="${RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64:-QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=}"
export RUNX_RECEIPT_SIGN_ISSUER_TYPE="${RUNX_RECEIPT_SIGN_ISSUER_TYPE:-hosted}"

RDIR="$(mktemp -d 2>/dev/null || echo /tmp/runx-nws-weather-demo)"
"$RUNX" harness "$OSS/examples/nws-weather-openapi" --receipt-dir "$RDIR" --json

node - "$RDIR" <<'NODE'
const fs = require("node:fs");
const path = require("node:path");

const root = process.argv[2];
const runs = path.join(root, "runs");
const states = fs.existsSync(runs)
  ? fs.readdirSync(runs).filter((name) => name.endsWith(".graph-state.json"))
  : [];
const receipts = fs
  .readdirSync(root)
  .filter((name) => name.endsWith(".json") && name !== "index.json")
  .map((name) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8")))
  .filter((receipt) => receipt?.schema === "runx.receipt.v1" && typeof receipt?.id === "string");

if (receipts.length === 0) {
  console.error("NWS weather graph did not write a signed runx.receipt.v1 receipt");
  process.exit(1);
}

for (const name of states) {
  const state = JSON.parse(fs.readFileSync(path.join(runs, name), "utf8"));
  const steps = state?.checkpoint?.steps ?? [];
  const points = steps.find((step) => step.step_id === "points");
  const output = points?.outputs;
  const claim = output?.skill_claim;
  const properties = claim?.properties;
  if (
    output?.status === "success" &&
    claim?.type === "Feature" &&
    typeof properties?.forecast === "string" &&
    properties.forecast.startsWith("https://api.weather.gov/gridpoints/") &&
    typeof properties?.forecastHourly === "string" &&
    properties.forecastHourly.startsWith("https://api.weather.gov/gridpoints/") &&
    typeof properties?.gridId === "string" &&
    Number.isFinite(properties?.gridX) &&
    Number.isFinite(properties?.gridY)
  ) {
    console.log(
      JSON.stringify(
        {
          provider: "national-weather-service",
          front: "http",
          openapi_described: true,
          grid: {
            id: properties.gridId,
            x: properties.gridX,
            y: properties.gridY,
          },
          forecast: properties.forecast,
          forecastHourly: properties.forecastHourly,
          receipts: receipts.map((receipt) => receipt.id),
        },
        null,
        2,
      ),
    );
    process.exit(0);
  }
}

console.error("NWS points response did not include stable forecast metadata");
process.exit(1);
NODE

echo "------------------------------------------------------------"
echo "the governed HTTP call executed against api.weather.gov and sealed:"
echo "receipts: $RDIR"
