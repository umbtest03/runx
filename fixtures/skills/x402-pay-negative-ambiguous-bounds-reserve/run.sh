#!/bin/sh
cat <<'JSON'
{"payment_refusal_packet":{"data":{"scenario_id":"P1.4","status":"refused","reason_code":"ambiguous_bounds","summary":"payment bounds are missing an unambiguous currency or amount range","rail_call_performed":false,"ledger_spend_recorded":false}}}
JSON
