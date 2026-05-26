import { describe, expect, it } from "vitest";

import {
  buildThreadStoryMessageOutboxEntry,
  readOutboxEntryControl,
} from "../knowledge/index.js";

import {
  buildRunxCommandResponse,
  buildRunxSourceDedupeKey,
  normalizeRunxSourceCommand,
  parseRunxSourceLocator,
  sanitizeRunxCommandText,
} from "./index.js";

describe("runx source command contracts", () => {
  it("parses GitHub issue URLs and preserves the concrete source repo over defaults", () => {
    const command = normalizeRunxSourceCommand({
      action: "issue-intake",
      source: "https://github.com/nitrosend/api/issues/114",
      defaultTargetRepo: "nitrosend/nitrosend",
      title: "Stripe webhook fails",
      body: "Webhook replay returns 500.",
      sourceId: "github-api",
    });

    expect(command.source.provider).toBe("github");
    expect(command.source.kind).toBe("github_issue");
    expect(command.source.threadLocator).toBe("github://nitrosend/api/issues/114");
    expect(command.source.issueUrl).toBe("https://github.com/nitrosend/api/issues/114");
    expect(command.targetRepo).toBe("nitrosend/api");
    expect(command.operationalPolicyRequest).toMatchObject({
      source_id: "github-api",
      target_repo: "nitrosend/api",
      action: "issue-intake",
      source_thread_locator: "github://nitrosend/api/issues/114",
    });
    expect(command.skillInputs.source_event).toMatchObject({
      provider: "github",
      kind: "github_issue",
      source_locator: "github://nitrosend/api/issues/114",
      target_repo: "nitrosend/api",
      requires_hydration: true,
      supported: true,
    });
  });

  it("parses GitHub pull request locators", () => {
    const parsed = parseRunxSourceLocator("github://runxhq/runx/pulls/42");

    expect(parsed.kind).toBe("github_pull_request");
    expect(parsed.targetRepo).toBe("runxhq/runx");
    expect(parsed.threadLocator).toBe("github://runxhq/runx/pulls/42");
    expect(parsed.pullRequestUrl).toBe("https://github.com/runxhq/runx/pull/42");
  });

  it("parses Slack permalinks into canonical thread locators without fetching Slack", () => {
    const parsed = parseRunxSourceLocator(
      "https://nitrosend.slack.com/archives/C0APFMY0V8Q/p1778834840485629?thread_ts=1778834000.123456&cid=C0APFMY0V8Q",
    );

    expect(parsed.provider).toBe("slack");
    expect(parsed.kind).toBe("slack_thread");
    expect(parsed.threadLocator).toBe(
      "slack://team/nitrosend/channel/C0APFMY0V8Q/thread/1778834000.123456",
    );
    expect(parsed.slack).toMatchObject({
      team: "nitrosend",
      channel: "C0APFMY0V8Q",
      messageTs: "1778834840.485629",
      threadTs: "1778834000.123456",
    });
    expect(parsed.requiresHydration).toBe(true);
  });

  it("parses Slack locators already in runx form", () => {
    const parsed = parseRunxSourceLocator("slack://nitrosend/C0B2GQVBAFJ/1779018404.175039");

    expect(parsed.threadLocator).toBe(
      "slack://team/nitrosend/channel/C0B2GQVBAFJ/thread/1779018404.175039",
    );
  });

  it("parses Sentry issue URLs as adapter-hydrated source references", () => {
    const parsed = parseRunxSourceLocator("https://nitrosend.sentry.io/issues/4509873210/?project=123");

    expect(parsed.provider).toBe("sentry");
    expect(parsed.kind).toBe("sentry_issue");
    expect(parsed.threadLocator).toBe("sentry://nitrosend/issues/4509873210");
    expect(parsed.requiresHydration).toBe(true);
    expect(parsed.diagnostics.join(" ")).toContain("hydrated");
  });

  it("returns unsupported diagnostics instead of pretending a source can dispatch", () => {
    const command = normalizeRunxSourceCommand({
      action: "issue-intake",
      source: "please look at the weird production thing",
      defaultTargetRepo: "runxhq/runx",
    });

    expect(command.supported).toBe(false);
    expect(command.targetRepo).toBe("runxhq/runx");
    expect(command.source.kind).toBe("unsupported");
    expect(command.diagnostics.join(" ")).toContain("not dispatchable");
  });

  it("builds stable source dedupe keys from canonical locators", () => {
    const fromUrl = parseRunxSourceLocator("https://github.com/runxhq/runx/issues/17");
    const fromLocator = parseRunxSourceLocator("github://runxhq/runx/issues/17");

    expect(buildRunxSourceDedupeKey({
      action: "issue-intake",
      source: fromUrl,
      targetRepo: "runxhq/runx",
    })).toBe(buildRunxSourceDedupeKey({
      action: "issue-intake",
      source: fromLocator,
      targetRepo: "runxhq/runx",
    }));
  });

  it("sanitizes command responses for chat surfaces", () => {
    const response = buildRunxCommandResponse({
      status: "blocked",
      summary: "RUNX_BIN=/Users/kam/dev/runx/packages/cli/dist/index.js GITHUB_TOKEN=ghp_123456789012345678901234567890123456",
      error: "{\"message\":\"provider failed while reading /private/tmp/sourcey-prs/raw.json with Bearer abcdefghijklmnopqrstuvwxyz123456\"}",
      nextAction: "Fix the source adapter and rerun intake.",
    });

    expect(response).toContain("Runx: blocked");
    expect(response).toContain("[local path]");
    expect(response).toContain("[redacted token]");
    expect(response).not.toContain("/Users/kam");
    expect(response).not.toContain("/private/tmp");
    expect(response).not.toContain("abcdefghijklmnopqrstuvwxyz");
    expect(response).not.toContain("{\"message\"");
  });

  it("redacts direct text without deleting useful reviewer context", () => {
    expect(sanitizeRunxCommandText("failed in /home/kam/dev/app with nskey_live_abcdefabcdefabcdef"))
      .toBe("failed in [local path] with [redacted token]");
  });

  it("feeds safe source command summaries into thread story outbox entries", () => {
    const command = normalizeRunxSourceCommand({
      action: "issue-intake",
      source: "https://nitrosend.slack.com/archives/C0APFMY0V8Q/p1778834840485629",
      defaultTargetRepo: "nitrosend/api",
      title: "Checkout webhook alert",
      body: "Operator asked runx to triage the source thread.",
      sourceId: "slack-support",
    });
    const summary = buildRunxCommandResponse({
      status: "blocked",
      source: command,
      summary: "Hydration failed in /Users/kam/dev/nitrosend with GITHUB_TOKEN=ghp_123456789012345678901234567890123456",
      nextAction: "Hydrate the Slack source thread, then rerun issue intake.",
    });

    const entry = buildThreadStoryMessageOutboxEntry({
      entryId: "message:runx-source-command:test",
      threadLocator: command.threadLocator ?? "local://missing-thread",
      workflow: "issue-intake",
      lane: "triage",
      sourceLocator: command.sourceLocator,
      story: {
        title: "Runx source command",
        sections: [
          {
            section_id: "triage_results",
            summary,
          },
        ],
      },
    });

    expect(entry.thread_locator).toBe(command.threadLocator);
    expect(readOutboxEntryControl(entry)).toMatchObject({
      workflow: "issue-intake",
      lane: "triage",
      source_locator: command.sourceLocator,
    });
    expect(JSON.stringify(entry)).not.toContain("/Users/kam");
    expect(JSON.stringify(entry)).not.toContain("ghp_");
    expect(JSON.stringify(entry)).toContain("[local path]");
  });
});
