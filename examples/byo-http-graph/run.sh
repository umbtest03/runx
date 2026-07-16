#!/usr/bin/env sh
# BYO HTTP portfolio demo: a non-GitHub provider read over the governed HTTP
# front using one-run local credential delivery.
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"
OSS="$(cd "$HERE/../.." && pwd)"
RUNX="${RUNX_BIN:-$OSS/crates/target/debug/runx}"
[ -x "$RUNX" ] || RUNX="$(command -v runx || true)"
[ -n "$RUNX" ] || { echo "runx binary not found; set RUNX_BIN." >&2; exit 1; }

# A demo-only receipt-signing identity (runx mandates signed receipts).
export RUNX_RECEIPT_SIGN_KID="${RUNX_RECEIPT_SIGN_KID:-runx-demo-key}"
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64="${RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64:-QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=}"
export RUNX_RECEIPT_SIGN_ISSUER_TYPE="${RUNX_RECEIPT_SIGN_ISSUER_TYPE:-hosted}"
export RUNX_HTTP_ALLOW_PRIVATE_NETWORK=1

# Isolate the demo profile from the operator's real Runx home. The value crosses
# the CLI boundary only on stdin and is removed from the child environment.
export RUNX_HOME="$(mktemp -d 2>/dev/null || echo /tmp/runx-byo-http-home)"
CRM_TOKEN="${EXAMPLE_CRM_TOKEN:-crm_demo_secret}"
unset EXAMPLE_CRM_TOKEN
printf '%s' "$CRM_TOKEN" | "$RUNX" credential set example-crm \
  --profile demo \
  --auth-mode api_key \
  --from-stdin >/dev/null

EXAMPLE_CRM_TOKEN="$CRM_TOKEN" node "$HERE/server.mjs" &
SERVER=$!
unset CRM_TOKEN
cleanup() {
  kill "$SERVER" 2>/dev/null || true
  wait "$SERVER" 2>/dev/null || true
  rm -rf "$RUNX_HOME"
}
trap cleanup EXIT
sleep 1
kill -0 "$SERVER" 2>/dev/null || { echo "BYO HTTP fixture server did not start." >&2; exit 1; }

RDIR="$(mktemp -d 2>/dev/null || echo /tmp/runx-byo-http-demo)"
OUT="$(mktemp 2>/dev/null || echo /tmp/runx-byo-http-output)"
"$RUNX" skill "$OSS/examples/byo-http-graph" \
  --account-id acct-42 \
  --profile demo \
  --skip-operator-context \
  --receipt-dir "$RDIR" \
  --json > "$OUT"

node - "$RDIR" <<'NODE'
const fs = require("node:fs");
const crypto = require("node:crypto");
const path = require("node:path");

const root = process.argv[2];
const expectedCredentialRef = `runx:credential:local:${crypto
  .createHash("sha256")
  .update("local:example-crm:demo")
  .digest("hex")}`;
const statesRoot = path.join(root, "runs");
const stateFiles = fs.existsSync(statesRoot)
  ? fs.readdirSync(statesRoot).filter((name) => name.endsWith(".graph-state.json"))
  : [];

for (const name of stateFiles) {
  const state = JSON.parse(fs.readFileSync(path.join(statesRoot, name), "utf8"));
  const steps = state?.checkpoint?.steps ?? [];
  const read = steps.find((step) => step.step_id === "read_account");
  const output = read?.output;
  const stdout = typeof output?.stdout === "string" ? output.stdout : "";
  const parsed = stdout ? JSON.parse(stdout) : undefined;
  const observations = output?.metadata?.credential_delivery_observations;
  if (
    output?.status === "Success" &&
    output?.metadata?.http_status === "200" &&
    parsed?.id === "acct-42" &&
    parsed?.plan === "portfolio" &&
    Array.isArray(observations) &&
    observations.length === 1 &&
    JSON.stringify(output).includes(expectedCredentialRef) &&
    !JSON.stringify(output).includes("crm_demo_secret")
  ) {
    console.log(
      JSON.stringify(
        {
          http_status: output.metadata.http_status,
          account: parsed,
          credential_ref: observations[0].credential_refs?.[0]?.uri,
        },
        null,
        2,
      ),
    );
    process.exit(0);
  }
}

console.error("BYO HTTP graph did not seal the expected credentialed provider read");
process.exit(1);
NODE

echo "------------------------------------------------------------"
echo "the BYO HTTP provider read executed with a local credential and sealed:"
echo "receipts: $RDIR"
