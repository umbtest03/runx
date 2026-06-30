import crypto from "node:crypto"
import fs from "node:fs"

const inputs = loadInputs()
const { invoice_status, aging_days, cadence_policy } = inputs

if (invoice_status !== "overdue" || aging_days <= 0) {
  console.error("Record is not overdue")
  process.exit(1)
}

if (!cadence_policy?.steps?.length || cadence_policy.cap == null) {
  console.error("Invalid cadence policy")
  process.exit(1)
}

let step = 0
for (let i = 0; i < cadence_policy.steps.length; i++) {
  if (aging_days <= cadence_policy.steps[i].max_days) {
    step = i
    break
  }
  step = i + 1
}

if (step >= cadence_policy.cap) {
  const output = {
    decision: { step, action: "escalate" },
    reminder_proposal: null,
    escalation: {
      reason: "cadence cap reached",
      recommended_action: "escalate to collections",
    },
  }
  console.log(JSON.stringify(output))
  process.exit(0)
}

const currentStep = cadence_policy.steps[step]
const output = {
  decision: { step, action: "remind" },
  reminder_proposal: {
    channel: currentStep.channel,
    content_digest: `sha256:${crypto.randomBytes(32).toString("hex")}`,
  },
  escalation: null,
}
console.log(JSON.stringify(output))
process.exit(0)

function loadInputs() {
  if (process.env.RUNX_INPUTS_JSON) {
    return JSON.parse(process.env.RUNX_INPUTS_JSON)
  }
  if (process.env.RUNX_INPUTS_PATH) {
    return JSON.parse(fs.readFileSync(process.env.RUNX_INPUTS_PATH, "utf8"))
  }
  return {}
}
