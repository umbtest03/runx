#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { auditOfficialSkills } from "./lib/skill-operator-value.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const binary = path.resolve(
  process.env.RUNX_BIN ?? path.join(root, "crates", "target", "debug", executable("runx")),
);
const outputPath = path.join(root, "docs", "core-skill-trial-results.json");
const providerEvidencePath = path.join(root, "docs", "core-skill-provider-trials.json");
const providerEvidence = JSON.parse(readFileSync(providerEvidencePath, "utf8"));
const decisions = JSON.parse(readFileSync(path.join(root, "docs", "core-skill-review-decisions.json"), "utf8"));
validateProviderEvidence(providerEvidence);
const timeoutMs = integerFlag("--timeout-ms", 120_000);
const write = process.argv.includes("--write");
const check = process.argv.includes("--check");
const json = process.argv.includes("--json");
const strict = process.argv.includes("--strict");

if (process.argv.includes("--managed-agent")) {
  throw new Error("core-skill trials forbid managed-agent execution");
}
if (write && check) throw new Error("choose either --write or --check");
if (!existsSync(binary)) {
  throw new Error(`runx binary is missing at ${binary}; build runx-cli first or set RUNX_BIN`);
}

const skills = auditOfficialSkills(root).filter((skill) => skill.visibility === "public");
const results = skills.map(trialSkill);
const packet = {
  schema: "runx.core_skill_trials.v1",
  execution: {
    managed_agent: false,
    credential_source: "isolated_none",
    cwd: "isolated_temp_directory",
    receipt_signer: "ephemeral_test_key",
  },
  summary: {
    public_skills: results.length,
    locally_proven: results.filter((result) => result.local_trial === "passed").length,
    failed: results.filter((result) => result.local_trial === "failed").length,
    unproven: results.filter((result) => result.local_trial === "unproven").length,
    meets_full_bar: results.filter((result) => result.meets_full_bar).length,
  },
  skills: results,
};
const serialized = `${JSON.stringify(packet, null, 2)}\n`;

if (write) writeFileSync(outputPath, serialized, "utf8");
if (check) {
  if (!existsSync(outputPath) || readFileSync(outputPath, "utf8") !== serialized) {
    throw new Error("core-skill trial results are stale; run with --write");
  }
}
if (json) process.stdout.write(serialized);
else {
  process.stdout.write(
    `trialled ${packet.summary.public_skills} public skills: `
      + `${packet.summary.locally_proven} locally proven, ${packet.summary.failed} failed, `
      + `${packet.summary.unproven} unproven, ${packet.summary.meets_full_bar} meet the full bar\n`,
  );
}
if (packet.summary.failed > 0 || (strict
  && (packet.summary.unproven > 0
    || packet.summary.meets_full_bar !== packet.summary.public_skills))) {
  process.exitCode = 1;
}

function trialSkill(skill) {
  const cases = uniqueProofCases(skill.proof.cases);
  const caseResults = cases.map((entry) => trialFixture(skill.skill, entry));
  const localTrial = caseResults.length === 0
    ? "unproven"
    : caseResults.every((entry) => entry.status === "passed")
      ? "passed"
      : "failed";
  const decision = decisions.recommendations?.[skill.skill];
  const archetype = decision?.archetype ?? "unreviewed";
  const providerReadbackRequired = skill.completion === "provider_readback";
  const providerTrial = providerEvidence.skills?.[skill.skill] ?? null;
  const providerReadback = providerReadbackRequired
    ? providerTrial?.status === "passed"
      ? "passed"
      : caseResults.some(
        (entry) => entry.status === "passed" && entry.provider_readback === "live-keyless-read",
      )
        ? "passed_by_live_keyless_fixture"
      : skill.capability_boundaries.includes("http") && localTrial === "passed"
        ? "passed_by_live_http_fixture"
      : "not_proven_by_isolated_fixture"
    : "not_required";
  const operationProofRequired = archetype === "operation" && skill.execution !== "plan";
  const operationProven = skill.proof.operation_cases > 0
    && caseResults.some((entry) => entry.status === "passed" && entry.proof_type === "operation");
  return {
    skill: skill.skill,
    path: skill.path,
    archetype,
    decision_status: "pending_review",
    preliminary_route: skill.disposition,
    managed_agent_acts: skill.managed_agent_acts,
    capabilities: skill.capabilities,
    static_findings: skill.issues,
    improvement_findings: skill.improvements,
    local_trial: localTrial,
    operation_proof: operationProven ? "passed" : operationProofRequired ? "missing" : "not_required",
    provider_readback: providerReadback,
    provider_trial: providerTrial,
    meets_full_bar: decision?.action === "keep"
      && skill.issues.length === 0
      && skill.improvements.length === 0
      && localTrial === "passed"
      && (!operationProofRequired || operationProven)
      && !providerReadback.startsWith("not_proven"),
    cases: caseResults,
  };
}

function uniqueProofCases(cases) {
  const seen = new Set();
  return cases.filter((entry) => {
    const key = entry.kind === "inline" ? `inline:${entry.path}` : `fixture:${entry.path}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function trialFixture(skill, fixture) {
  const rootDir = mkdtempSync(path.join(os.tmpdir(), "runx-core-skill-trial-"));
  const receiptDir = path.join(rootDir, "receipts");
  const fixturePath = path.join(root, fixture.path);
  try {
    let result = runHarness(fixturePath, receiptDir, rootDir, rootDir);
    if (result.status !== 0 && String(result.stderr).includes("writable path(s) outside workspace")) {
      const sourceSkillDir = path.join(root, "skills", skill);
      const copiedSkillDir = path.join(rootDir, "workspace", skill);
      mkdirSync(path.dirname(copiedSkillDir), { recursive: true });
      cpSync(sourceSkillDir, copiedSkillDir, { recursive: true });
      const copiedFixture = path.join(copiedSkillDir, path.relative(sourceSkillDir, fixturePath));
      result = runHarness(copiedFixture, receiptDir, copiedSkillDir, rootDir);
    }
    if (result.error) {
      return failedCase(fixture, `process error: ${result.error.message}`);
    }
    if (result.status !== 0) {
      return failedCase(fixture, boundedFailure(result.stderr || result.stdout, rootDir));
    }
    let report;
    try {
      report = JSON.parse(result.stdout);
    } catch (error) {
      return failedCase(fixture, `invalid JSON harness result: ${error.message}`);
    }
    if (report.schema === "runx.receipt.v1" && report.id && report.seal?.disposition) {
      return {
        name: fixture.name,
        path: fixture.path,
        runner: fixture.runner,
        proof_type: fixture.proof_type,
        ...(fixture.provider_readback ? { provider_readback: fixture.provider_readback } : {}),
        status: "passed",
        receipt: {
          schema: report.schema,
          id: report.id,
          disposition: report.seal.disposition,
          reason_code: report.seal.reason_code,
        },
      };
    }
    if (report.status === "passed" && Number.isInteger(report.case_count)) {
      return {
        name: fixture.name,
        path: fixture.path,
        runner: fixture.runner,
        proof_type: fixture.proof_type,
        ...(fixture.provider_readback ? { provider_readback: fixture.provider_readback } : {}),
        status: "passed",
        harness: { status: report.status, case_count: report.case_count },
      };
    }
    return failedCase(fixture, "harness did not return a sealed receipt or passing report");
  } finally {
    rmSync(rootDir, { recursive: true, force: true });
  }
}

function runHarness(fixturePath, receiptDir, cwd, rootDir) {
  return spawnSync(
    binary,
    ["harness", fixturePath, "--receipt-dir", receiptDir, "--json"],
    {
      cwd,
      env: isolatedEnv(rootDir),
      encoding: "utf8",
      timeout: timeoutMs,
      maxBuffer: 8 * 1024 * 1024,
    },
  );
}

function failedCase(fixture, reason) {
  return {
    name: fixture.name,
    path: fixture.path,
    runner: fixture.runner,
    proof_type: fixture.proof_type,
    status: "failed",
    reason,
  };
}

function isolatedEnv(rootDir) {
  const binaryDir = path.dirname(binary);
  const inheritedPath = process.env.PATH ?? "";
  return Object.fromEntries(
    [
      ["PATH", [binaryDir, inheritedPath].filter(Boolean).join(path.delimiter)],
      ["TMPDIR", process.env.TMPDIR ?? os.tmpdir()],
      ["SSL_CERT_FILE", process.env.SSL_CERT_FILE],
      ["SSL_CERT_DIR", process.env.SSL_CERT_DIR],
      ["HOME", rootDir],
      ["RUNX_HOME", path.join(rootDir, "runx-home")],
      ["RUNX_RECEIPT_SIGN_KID", "runx-core-skill-trial-key"],
      [
        "RUNX_RECEIPT_SIGN_ED25519_SEED_BASE64",
        "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=",
      ],
      ["RUNX_RECEIPT_SIGN_ISSUER_TYPE", "hosted"],
      ["NO_COLOR", "1"],
    ].filter(([, value]) => typeof value === "string" && value.length > 0),
  );
}

function boundedFailure(value, rootDir) {
  return String(value || "trial failed without output")
    .replaceAll(rootDir, "<trial-dir>")
    .replaceAll(root, "<repo>")
    .trim()
    .slice(0, 2_000);
}

function integerFlag(name, fallback) {
  const prefix = `${name}=`;
  const value = process.argv.find((entry) => entry.startsWith(prefix));
  if (!value) return fallback;
  const parsed = Number(value.slice(prefix.length));
  if (!Number.isInteger(parsed) || parsed < 1) throw new Error(`${name} expects a positive integer`);
  return parsed;
}

function executable(name) {
  return process.platform === "win32" ? `${name}.exe` : name;
}

function validateProviderEvidence(value) {
  if (value?.schema !== "runx.core_skill_provider_trials.v1" || !value.skills) {
    throw new Error("invalid core-skill provider trial evidence");
  }
  const serialized = JSON.stringify(value);
  if (/api[_-]?key|authorization|bearer|credential_material_ref|secret|token/i.test(serialized)) {
    throw new Error("provider trial evidence contains a forbidden credential field");
  }
  for (const [skill, evidence] of Object.entries(value.skills)) {
    if (evidence.status !== "passed" || evidence.managed_agent !== false || evidence.mutation !== false) {
      throw new Error(`provider evidence for ${skill} is not a passed, no-agent, read-only trial`);
    }
    if (!/^sha256:[a-f0-9]{64}$/u.test(evidence.receipt_id ?? "")) {
      throw new Error(`provider evidence for ${skill} has no sealed receipt id`);
    }
  }
}
