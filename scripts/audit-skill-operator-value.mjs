#!/usr/bin/env node

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  auditOfficialSkills,
  auditSummary,
  reviewDocument,
  reviewSummary,
} from "./lib/skill-operator-value.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = path.join(root, "docs", "core-skill-review.md");
const decisionsPath = path.join(root, "docs", "core-skill-review-decisions.json");
const trialsPath = path.join(root, "docs", "core-skill-trial-results.json");
const write = process.argv.includes("--write");
const check = process.argv.includes("--check");
const json = process.argv.includes("--json");

if (write && check) {
  throw new Error("choose either --write or --check");
}

const skills = auditOfficialSkills(root);
if (skills.length === 0) throw new Error("operator-value audit found no top-level skills");
if (new Set(skills.map((skill) => skill.skill)).size !== skills.length) {
  throw new Error("operator-value audit found duplicate top-level skill names");
}
const decisions = JSON.parse(readFileSync(decisionsPath, "utf8"));
const trials = JSON.parse(readFileSync(trialsPath, "utf8"));
validateReviewInputs(skills, decisions, trials);
const document = reviewDocument(skills, decisions, trials);

if (write) writeFileSync(reportPath, document, "utf8");
if (check) {
  let current;
  try {
    current = readFileSync(reportPath, "utf8");
  } catch (error) {
    if (error?.code === "ENOENT") {
      throw new Error(`missing generated audit report: ${path.relative(root, reportPath)}`);
    }
    throw error;
  }
  if (current !== document) {
    throw new Error("core skill review is stale; run with --write");
  }
}

if (json) {
  console.log(JSON.stringify({
    audit: auditSummary(skills),
    review: reviewSummary(skills, decisions, trials),
    skills,
  }, null, 2));
} else {
  const summary = reviewSummary(skills, decisions, trials);
  console.log(
    `review covers ${summary.skill_count} baseline skills; current core=${summary.current_skill_count} (${Object.entries(summary.recommendations)
      .map(([name, count]) => `${name}=${count}`)
      .join(", ")})`,
  );
}

function validateReviewInputs(skills, decisions, trials) {
  if (decisions?.schema !== "runx.core_skill_review_decisions.v1"
    || !["proposed", "implemented"].includes(decisions.status)) {
    throw new Error("core skill review decisions must be a proposed or implemented v1 packet");
  }
  if (trials?.schema !== "runx.core_skill_trials.v1" || !Array.isArray(trials.skills)) {
    throw new Error("core skill trial results must be a v1 packet");
  }

  const allowedActions = new Set([
    "keep",
    "improve",
    "consolidate_review",
    "internal_fixture",
    "internal_runtime",
  ]);
  const allowedArchetypes = new Set([
    "operation",
    "workflow",
    "artifact",
    "builder",
    "context",
    "runtime",
  ]);
  const skillNames = new Set(skills.map((skill) => skill.skill));
  const decisionNames = Object.keys(decisions.recommendations ?? {}).sort();
  for (const [skill, decision] of Object.entries(decisions.recommendations)) {
    if (!allowedActions.has(decision?.action)
      || !allowedArchetypes.has(decision?.archetype)
      || typeof decision?.reason !== "string"
      || !decision.reason.trim()) {
      throw new Error(`invalid core skill review decision for ${skill}`);
    }
    if (!skillNames.has(skill)) {
      throw new Error(`reviewed core skill is missing: ${skill}`);
    }
    if (decision.action !== "keep"
      && decision.action !== "internal_fixture"
      && decision.action !== "internal_runtime"
      && (typeof decision.improvement !== "string" || !decision.improvement.trim())) {
      throw new Error(`core skill review decision for ${skill} needs an improvement or consolidation target`);
    }
  }
  for (const skill of skillNames) {
    if (!decisionNames.includes(skill)) {
      throw new Error(`current core skill has no review decision: ${skill}`);
    }
  }

  const publicNames = skills
    .filter((skill) => skill.visibility === "public")
    .map((skill) => skill.skill)
    .sort();
  const trialNames = trials.skills.map((skill) => skill.skill).sort();
  if (new Set(trialNames).size !== trialNames.length
    || JSON.stringify(publicNames) !== JSON.stringify(trialNames)) {
    throw new Error("core skill trials must cover every public skill exactly once");
  }
  if (trials.summary.failed !== 0) {
    throw new Error("core skill trials contain a failing public skill");
  }
}
