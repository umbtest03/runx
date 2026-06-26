#!/bin/bash
export RUNX_RECEIPT_SIGN_KID=runx-publish-harness-local
export RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64=QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=
export RUNX_RECEIPT_SIGN_ISSUER_TYPE=ci

runx skill umbtest03/dunning-ladder@sha-5a9176c1fd75 \
  --registry https://api.runx.ai \
  --json \
  -i invoice_status=overdue \
  -i aging_days=15 \
  --input-json cadence_policy='{"steps":[{"max_days":7,"channel":"email"},{"max_days":21,"channel":"email"},{"max_days":45,"channel":"letter"}],"cap":3}'
