import {
  defineTool,
  firstNonEmptyString,
  isRecord,
  prune,
  recordInput,
  stringInput,
} from "@runxhq/authoring";
import {
  renderIssueToPrReviewerMarkdown,
  sanitizePublicMarkdown,
  summarizePublicHandoffMarkdown,
} from "@runxhq/core/knowledge";
import { admitOperationalPolicyRequest } from "@runxhq/contracts";

export default defineTool({
  name: "outbox.build_pull_request",
  description: "Build a provider-agnostic draft pull-request packet and outbox entry from native scafld surfaces.",
  inputs: {
    task_id: stringInput({ description: "scafld task id that produced the completed engineering state." }),
    thread_title: stringInput({ optional: true, description: "Canonical thread title when the caller already has one." }),
    thread_locator: stringInput({ optional: true, description: "Canonical thread locator for the bounded harness." }),
    thread: recordInput({ optional: true, description: "Optional hydrated thread that may already carry a pull_request outbox entry." }),
    outbox_entry: recordInput({ optional: true, description: "Optional current pull_request outbox entry when refreshing an existing draft." }),
    harness_context: recordInput({ optional: true, description: "Optional captured harness context containing signal and decision state." }),
    operational_policy: recordInput({ optional: true, description: "Optional runx.operational_policy.v1 packet used for request-time admission." }),
    source_id: stringInput({ optional: true, description: "Operational policy source id for request-time admission." }),
    target_repo: stringInput({ optional: true, description: "Intended repository slug when the caller already knows it." }),
    runner_id: stringInput({ optional: true, description: "Operational policy runner id for request-time admission." }),
    policy_action: stringInput({ optional: true, description: "Operational policy action, defaults to issue-to-pr." }),
    source_thread_locator: stringInput({ optional: true, description: "Recoverable source-thread locator for request-time admission." }),
    branch: stringInput({ optional: true, description: "Explicit head branch for provider publication." }),
    fix_bundle: recordInput({ optional: true, description: "Bounded fix bundle used to derive the governed file list for provider publication." }),
    handoff_markdown: stringInput({ description: "Native markdown emitted by `scafld handoff`." }),
    build_result: recordInput({ description: "Native scafld build result payload." }),
    review_result: recordInput({ description: "Native scafld review result payload." }),
    completion_result: recordInput({ description: "Native scafld complete result payload." }),
    status_snapshot: recordInput({ optional: true, description: "Native scafld status result payload." }),
    current_branch: recordInput({ optional: true, description: "Current git branch packet from git.current_branch." }),
    base: stringInput({ optional: true, description: "Base ref for the draft pull request." }),
  },
  output: {
    named_emits: {
      draft_pull_request: "draft_pull_request_packet",
      outbox_entry: "outbox_entry",
    },
    outputs: {
      draft_pull_request: {
        packet: "runx.outbox.draft_pull_request.v1",
      },
      outbox_entry: {
        packet: "runx.outbox.entry.v1",
      },
    },
  },
  scopes: ["runx:repo:package"],
  run: runBuildPullRequest,
});

function runBuildPullRequest({ inputs }) {
  const taskId = inputs.task_id;
  const handoffMarkdown = inputs.handoff_markdown;
  const buildResult = unwrapRecord(inputs.build_result) ?? {};
  const reviewResult = unwrapRecord(inputs.review_result) ?? {};
  const completionResult = unwrapRecord(inputs.completion_result) ?? {};
  const statusSnapshot = unwrapRecord(inputs.status_snapshot);
  const currentBranch = unwrapRecord(inputs.current_branch);
  const thread = optionalRecord(inputs.thread);
  const harnessContext = optionalRecord(inputs.harness_context);
  const explicitOutboxEntry = optionalRecord(inputs.outbox_entry);
  const fixBundle = unwrapRecord(inputs.fix_bundle);

  const threadContext = thread ?? {};
  const changedFiles = normalizeChangedFiles(fixBundle);

  const existingOutboxEntry =
    normalizePullRequestOutbox(explicitOutboxEntry) ??
    latestPullRequestOutbox(thread);

  const threadLocator = firstNonEmptyString(
    inputs.thread_locator,
    existingOutboxEntry?.thread_locator,
    threadContext.thread_locator,
  );

  const title = firstNonEmptyString(
    completionResult.title,
    statusSnapshot?.title,
    inputs.thread_title,
    threadContext.title,
    taskId,
  );

  const targetRepo = firstNonEmptyString(
    inputs.target_repo,
    parseRepoSlug(firstNonEmptyString(threadContext.canonical_uri)),
  );
  const policyAdmission = admitPolicyRequest({
    policy: optionalRecord(inputs.operational_policy),
    sourceId: inputs.source_id,
    targetRepo,
    runnerId: inputs.runner_id,
    policyAction: inputs.policy_action,
    sourceThreadLocator: firstNonEmptyString(inputs.source_thread_locator, threadLocator),
  });

  const action = existingOutboxEntry ? "refresh" : "create";
  const reviewVerdict = firstNonEmptyString(
    reviewResult.verdict,
    optionalRecord(completionResult.review)?.verdict,
    optionalRecord(completionResult.review)?.status,
  );
  const check = buildCheck(buildResult);
  const checkStatus = firstNonEmptyString(check.status);
  const syncStatus = firstNonEmptyString(statusSnapshot?.session_ok === false ? "degraded" : "ok");
  const pushReady =
    firstNonEmptyString(completionResult.status, statusSnapshot?.status) === "completed" &&
    checkStatus === "success" &&
    !isFailingReview(reviewVerdict);
  const branch = firstNonEmptyString(
    inputs.branch,
    currentBranch?.branch,
  );
  const base = firstNonEmptyString(inputs.base);
  const dedupe = buildPullRequestDedupe({
    existingOutboxEntry,
    taskId,
    targetRepo,
    branch,
    threadLocator,
  });

  const draftPullRequest = prune({
    schema_version: "runx.pull-request-draft.v1",
    action,
    push_ready: pushReady,
    task_id: taskId,
    thread: prune({
      thread_locator: threadLocator,
      thread_kind: firstNonEmptyString(threadContext.thread_kind),
      title: firstNonEmptyString(
        inputs.thread_title,
        threadContext.title,
        title,
      ),
      canonical_uri: firstNonEmptyString(threadContext.canonical_uri),
    }),
    target: prune({
      repo: targetRepo,
      branch,
      base,
      remote: "origin",
    }),
    harness_context: summarizeHarnessContext(harnessContext),
    operational_policy: summarizePolicyAdmission(policyAdmission),
    pull_request: {
      title,
      body_markdown: buildReviewerPullRequestBody({
        taskId,
        title,
        threadLocator,
        threadContext,
        handoffMarkdown,
        buildResult,
        reviewResult,
        completionResult,
        statusSnapshot,
        check,
        reviewVerdict,
        branch,
        base,
      }),
      is_draft: true,
    },
    engineering_summary_markdown: summarizePublicHandoffMarkdown(firstNonEmptyText(handoffMarkdown)) ?? "",
    checks: Object.keys(check).length > 0 ? check : undefined,
    governance: prune({
      status: firstNonEmptyString(
        completionResult.status,
        statusSnapshot?.status,
      ),
      review_verdict: reviewVerdict,
      blocking_count: reviewFindingCount(reviewResult, "blocking"),
      non_blocking_count: reviewFindingCount(reviewResult, "non_blocking"),
      sync_status: syncStatus,
      build_passed: numberOrUndefined(buildResult.passed),
      build_failed: numberOrUndefined(buildResult.failed),
      changed_files: changedFiles,
    }),
  });

  const outboxEntry = prune({
    entry_id: firstNonEmptyString(
      existingOutboxEntry?.entry_id,
      `pull_request:${taskId}`,
    ),
    kind: "pull_request",
    locator: firstNonEmptyString(existingOutboxEntry?.locator),
    title,
    status: firstNonEmptyString(
      existingOutboxEntry?.status,
      existingOutboxEntry?.locator ? "draft" : "proposed",
    ),
    thread_locator: threadLocator,
    metadata: prune({
      schema_version: "runx.outbox-entry.pull-request.v1",
      packet_schema_version: draftPullRequest.schema_version,
      action,
      task_id: taskId,
      repo: draftPullRequest.target?.repo,
      branch: draftPullRequest.target?.branch,
      base: draftPullRequest.target?.base,
      harness_context: summarizeHarnessContext(harnessContext),
      operational_policy: summarizePolicyAdmission(policyAdmission),
      title,
      review_verdict: reviewVerdict,
      check_status: checkStatus,
      sync_status: syncStatus,
      push_ready: pushReady,
      changed_files: changedFiles,
      dedupe,
      human_merge_gate: "required",
      post_merge_observation: "provider_state_update",
      story_milestones: [
        "signal",
        "decision",
        "spec",
        "build",
        "review",
        "pull_request",
        "merge_gate",
        "outcome",
      ],
    }),
  });

  return {
    draft_pull_request: draftPullRequest,
    outbox_entry: outboxEntry,
  };
}

function admitPolicyRequest({
  policy,
  sourceId,
  targetRepo,
  runnerId,
  policyAction,
  sourceThreadLocator,
}) {
  if (!policy) {
    return undefined;
  }
  const admission = admitOperationalPolicyRequest(policy, {
    source_id: firstNonEmptyString(sourceId),
    target_repo: targetRepo,
    action: firstNonEmptyString(policyAction) ?? "issue-to-pr",
    runner_id: firstNonEmptyString(runnerId),
    source_thread_locator: firstNonEmptyString(sourceThreadLocator),
  });
  if (admission.status === "deny") {
    const codes = admission.findings.map((finding) => finding.code).join(", ");
    throw new Error(`operational policy denied pull request packaging: ${codes}`);
  }
  return admission;
}

function summarizePolicyAdmission(admission) {
  if (!admission) {
    return undefined;
  }
  return prune({
    policy_id: admission.policy_id,
    source_id: admission.source_id,
    target_repo: admission.target_repo,
    runner_id: admission.runner_id,
    owner_route_id: admission.owner_route_id,
    owner_count: Array.isArray(admission.owners) ? admission.owners.length : undefined,
    dedupe_strategy: admission.dedupe_strategy,
    outcome_close_mode: admission.outcome_close_mode,
    source_thread_required: admission.source_thread_required,
    mutate_target_repo: admission.mutate_target_repo,
    require_human_merge_gate: admission.require_human_merge_gate,
  });
}

function buildPullRequestDedupe({
  existingOutboxEntry,
  taskId,
  targetRepo,
  branch,
  threadLocator,
}) {
  const branchKey = targetRepo && branch ? `${targetRepo}:${branch}` : undefined;
  return prune({
    strategy: branchKey ? "branch" : "source_fingerprint",
    key: firstNonEmptyString(branchKey, threadLocator, `task:${taskId}`),
    result: existingOutboxEntry ? "reused" : "created",
    existing_entry_id: existingOutboxEntry?.entry_id,
    existing_locator: existingOutboxEntry?.locator,
  });
}

function summarizeHarnessContext(harnessContext) {
  if (!harnessContext) {
    return undefined;
  }
  const harness = optionalRecord(harnessContext.harness);
  const signal = optionalRecord(harnessContext.signal);
  const decision = optionalRecord(harnessContext.decision);
  const fingerprint = optionalRecord(signal?.fingerprint);
  return prune({
    harness_id: firstNonEmptyString(harness?.harness_id),
    state: firstNonEmptyString(harness?.state),
    signal: signal
      ? prune({
          signal_id: firstNonEmptyString(signal.signal_id),
          signal_type: firstNonEmptyString(signal.signal_type),
          fingerprint: firstNonEmptyString(fingerprint?.value),
        })
      : undefined,
    decision: decision
      ? prune({
          decision_id: firstNonEmptyString(decision.decision_id),
          choice: firstNonEmptyString(decision.choice),
          selected_act_id: firstNonEmptyString(decision.selected_act_id),
        })
      : undefined,
  });
}

function optionalRecord(value) {
  return isRecord(value) ? value : undefined;
}

function unwrapRecord(value) {
  if (!isRecord(value)) {
    return undefined;
  }
  if (isRecord(value.data)) {
    return value.data;
  }
  return value;
}

function buildCheck(buildResult) {
  const passed = numberOrUndefined(buildResult.passed);
  const failed = numberOrUndefined(buildResult.failed);
  const status = failed !== undefined
    ? failed === 0 ? "success" : "failure"
    : firstNonEmptyString(buildResult.status);
  return prune({
    status,
    summary: status ? `scafld build ${status}` : undefined,
    passed,
    failed,
  });
}

function reviewFindingCount(reviewResult, severity) {
  const explicit = numberOrUndefined(reviewResult[`${severity}_count`]);
  if (explicit !== undefined) {
    return explicit;
  }
  if (!Array.isArray(reviewResult.findings)) {
    return undefined;
  }
  if (severity === "blocking") {
    return reviewResult.findings
      .filter((finding) => isRecord(finding) && finding.blocks_completion === true)
      .length;
  }
  if (severity === "non_blocking") {
    return reviewResult.findings
      .filter((finding) => isRecord(finding) && finding.blocks_completion === false)
      .length;
  }
  return reviewResult.findings
    .filter((finding) => isRecord(finding) && finding.severity === severity)
    .length;
}

function isFailingReview(value) {
  const verdict = firstNonEmptyString(value);
  return verdict === "fail" || verdict === "blocked" || verdict === "failure";
}

function buildReviewerPullRequestBody(options) {
  const {
    taskId,
    title,
    threadLocator,
    threadContext,
    handoffMarkdown,
    buildResult,
    reviewResult,
    completionResult,
    statusSnapshot,
    check,
    reviewVerdict,
    branch,
    base,
  } = options;
  const sourceTitle = sanitizePublicMarkdown(firstNonEmptyString(
    threadContext.title,
    title,
  ));
  const sourceLocator = sanitizePublicMarkdown(firstNonEmptyString(
    threadLocator,
    threadContext.thread_locator,
  ));
  const handoff = firstNonEmptyText(handoffMarkdown);
  const checkStatus = firstNonEmptyString(check.status, buildResult.status);
  const buildPassed = numberOrUndefined(buildResult.passed);
  const buildFailed = numberOrUndefined(buildResult.failed);
  const blockingCount = reviewFindingCount(reviewResult, "blocking");
  const nonBlockingCount = reviewFindingCount(reviewResult, "non_blocking");
  const completedStatus = firstNonEmptyString(completionResult.status, statusSnapshot?.status);
  return renderIssueToPrReviewerMarkdown({
    taskId,
    title,
    sourceTitle,
    sourceLocator,
    branch,
    base,
    governanceStatus: completedStatus,
    checkStatus,
    buildPassed,
    buildFailed,
    reviewVerdict,
    blockingCount,
    nonBlockingCount,
    handoffMarkdown: handoff,
  });
}

function firstNonEmptyText(...values) {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) {
      return value;
    }
  }
  return undefined;
}

function numberOrUndefined(value) {
  return typeof value === "number" && Number.isFinite(value)
    ? value
    : undefined;
}

function stringArray(value) {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const items = value.filter(
    (entry) => typeof entry === "string" && entry.trim().length > 0,
  );
  return items.length > 0 ? items : undefined;
}

function normalizePullRequestOutbox(value) {
  if (!isRecord(value)) {
    return undefined;
  }
  if (value.kind !== "pull_request") {
    return undefined;
  }
  const entryId = firstNonEmptyString(value.entry_id);
  if (!entryId) {
    return undefined;
  }
  return {
    entry_id: entryId,
    kind: "pull_request",
    locator: firstNonEmptyString(value.locator),
    status: firstNonEmptyString(value.status),
    thread_locator: firstNonEmptyString(value.thread_locator),
  };
}

function normalizeChangedFiles(fixBundle) {
  const files = Array.isArray(fixBundle?.files) ? fixBundle.files : [];
  const paths = files
    .filter(isRecord)
    .map((file) => firstNonEmptyString(file.path))
    .filter((filePath) => filePath !== undefined);
  return paths.length > 0 ? [...new Set(paths)] : undefined;
}

function latestPullRequestOutbox(state) {
  const outbox = Array.isArray(state?.outbox) ? state.outbox : [];
  for (let index = outbox.length - 1; index >= 0; index -= 1) {
    const candidate = normalizePullRequestOutbox(outbox[index]);
    if (candidate) {
      return candidate;
    }
  }
  return undefined;
}

function parseRepoSlug(remoteUrl) {
  const value = firstNonEmptyString(remoteUrl);
  if (!value) {
    return undefined;
  }
  const sshMatch = value.match(/[:/]([^/:]+\/[^/]+?)(?:\.git)?$/);
  if (!sshMatch) {
    return undefined;
  }
  return sshMatch[1];
}
