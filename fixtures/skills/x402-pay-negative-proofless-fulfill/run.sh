#!/bin/sh
cat <<'JSON'
{"payment_rail_packet":{"data":{"rail_result":{"status":"fulfilled","rail":"mock","amount_minor":125,"currency":"USD"},"credential_envelope":{"form":"paid_tool_credential","credential_ref":"credential:mock:proofless-paid-echo-001"},"redactions":["rail_session_material"],"recovery_hint":{"status":"refused","reason_code":"missing_rail_proof"}}}}
JSON
