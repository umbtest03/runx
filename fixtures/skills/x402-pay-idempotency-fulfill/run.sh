#!/bin/sh
count_path=${RUNX_PAYMENT_RAIL_COUNT_PATH:-}
if [ -n "$count_path" ]; then
  count=0
  if [ -f "$count_path" ]; then
    count=$(cat "$count_path")
  fi
  count=$((count + 1))
  printf '%s\n' "$count" > "$count_path"
fi

key=${RUNX_X402_IDEMPOTENCY_KEY:-payment:paid-echo-001}
mode=${RUNX_X402_RAIL_MODE:-sealed}

if [ "$mode" = "partial" ]; then
  printf '%s\n' '{"payment_rail_packet":{"data":{"rail_result":{"status":"partial","rail":"mock","amount_minor":125,"currency":"USD","counterparty":"merchant:paid-echo"},"recovery_hint":{"status":"partial","idempotency_key":"'"${key}"'","next_action":"recover_by_idempotency_key"}}}}'
  printf '%s\n' "partial rail mutation for ${key}" >&2
  exit 1
fi

printf '%s\n' '{"payment_rail_packet":{"data":{"rail_result":{"status":"fulfilled","rail":"mock","amount_minor":125,"currency":"USD","counterparty":"merchant:paid-echo"},"credential_envelope":{"form":"paid_tool_credential","credential_ref":"credential:mock:paid-echo-001"},"redactions":["rail_session_material"],"recovery_hint":{"status":"sealed"},"rail_proof":{"proof_ref":"receipt-proof:mock:paid-echo-001","idempotency_key":"'"${key}"'","rail_session_material_ref":"rail-session-material:mock:paid-echo-001"}}}}'
