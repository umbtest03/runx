#!/bin/sh
printf '%s\n' '{"payment_refusal_packet":{"data":{"scenario_id":"P1.2","status":"refused","reason_code":"malformed_challenge","summary":"x402 challenge is missing required bounded payment fields","rail_call_performed":false,"ledger_spend_recorded":false}}}'
