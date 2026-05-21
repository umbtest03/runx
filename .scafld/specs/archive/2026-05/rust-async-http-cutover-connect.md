---
spec_version: '2.0'
task_id: rust-async-http-cutover-connect
created: '2026-05-21T02:07:31Z'
updated: '2026-05-21T03:11:23Z'
status: completed
harden_status: passed
size: medium
risk_level: high
---

# rust-async-http-cutover-connect

## Current State

Status: completed
Current phase: final
Next: done
Reason: task completed
Blockers: none
Allowed follow-up command: `none`
Latest runner update: 2026-05-21T03:11:23Z
Review gate: pass

## Summary

Replace the connect client's curl-subprocess transport with a reqwest-backed
implementation while preserving the current synchronous `HostedTransport`
contract used by `ConnectClient`. This is the implementation handoff for the
connect slice reserved by `.scafld/specs/archive/2026-05/rust-async-http-layer.md`.

The future implementation must keep `runx connect` behavior stable: list,
revoke, preprovision, OAuth-required polling, timeout behavior, error
redaction, JSON contract validation, and CLI fixture outputs continue to match
the existing tests. This draft does not add dependencies, edit Cargo manifests,
or change Rust source.

## Objectives

- Add a reqwest-backed hosted transport for connect HTTP calls behind the
  existing synchronous `HostedTransport` trait.
- Use the parent spec's panic-free blocking bridge so synchronous callers do
  not call `tokio::runtime::Runtime::block_on` from inside an active runtime.
- Preserve connect client API shape, route construction, headers, redaction,
  OAuth polling order, default poll interval, and timeout semantics.
- Land exact reviewed dependency pins and deny/license evidence in the
  implementation commit, scoped to the runtime adapter tier.
- Prove fixture parity with the existing connect CLI tests before any curl
  path is removed.
- Resolve feature wiring here: `runx-runtime` defines
  `async-http = ["dep:reqwest", "dep:tokio"]`, and `cli-tool = ["async-http"]`.
  The existing `runx-cli` dependency on `runx-runtime` with `cli-tool` is the
  cutover configuration, so the CLI fixture binary exercises reqwest.

## Scope

In scope for the future implementation:

- `crates/runx-runtime/src/runtime_http.rs`
  - Introduce a reqwest transport under the `async-http` feature.
  - Keep `HostedTransport::send` synchronous and fallible.
  - Keep URL scheme validation, header validation, status/body response shape,
    and no-redirect behavior equivalent to the current curl transport.
- `crates/runx-runtime/src/connect/client.rs`
  - Switch the default connect transport to the reqwest-backed transport when
    the connect cutover enables `async-http`.
  - Preserve generic transport injection for tests and future fixtures.
  - Keep request bodies and debug output redacted.
- `crates/runx-runtime/src/connect/mod.rs`
  - Re-export only surfaces needed by current callers; avoid exposing reqwest
    or tokio types in public connect APIs.
- `crates/runx-runtime/Cargo.toml`, `crates/Cargo.toml`,
  `crates/deny.toml`, `crates/Cargo.lock`
  - Add the exact reviewed dependencies and supply-chain exceptions required
    by this cutover.
- Tests under `crates/runx-cli/tests/connect.rs` and runtime hosted HTTP tests
  that demonstrate parity for the connect client.

Out of scope:

- Implementing the code change in this task.
- Removing `hosted_http.rs` or deleting the curl transport entirely.
- Changing payment, schema, runtime behavior outside the connect/hosted HTTP
  boundary named above.
- Enabling `async-http` for unrelated adapter-tier consumers.
- Adding cookies, redirects, streaming, reqwest blocking APIs, native TLS, or
  any direct `hyper` dependency.

## Dependencies

Parent policy:

- `.scafld/specs/archive/2026-05/rust-async-http-layer.md` is complete and
  defines the only approved migration shape.

Implementation dependency requirements:

- `reqwest` must be added with an exact pin of the form
  `version = "=<major.minor.patch>"`.
- `tokio` must be added with an exact pin of the form
  `version = "=<major.minor.patch>"`.
- Required reqwest feature shape:
  `default-features = false`, features `rustls`, `json`.
- Required tokio feature shape:
  `default-features = false`, features `rt`, `net`, `time`.
- Forbidden in this cutover: reqwest `blocking`, `cookies`, `stream`,
  `default-features = true`, direct `hyper`, `async-std`, `ureq`, `axum`.
- The implementation PR must record the reviewed exact versions, the
  `crates/Cargo.lock` diff, and why each newly introduced crate is acceptable.
- The feature contract is not optional after this cutover: `runx-runtime`
  `cli-tool` implies `async-http`; non-CLI library builds may still use
  `--no-default-features`, but the cargo-installed `runx` binary cannot silently
  keep the curl transport.

## Assumptions

- `ConnectClient` remains a synchronous public API for this slice.
- Existing tests use injected local HTTP fixtures and must continue to pass
  without requiring external network access.
- `runx-cli` keeps depending on `runx-runtime` with `features = ["cli-tool",
  "mcp"]`; the runtime `cli-tool` feature is the single feature edge that turns
  on async HTTP for the CLI binary.
- OAuth polling semantics are part of the behavioral contract:
  `poll_after_ms` from the pending response takes precedence over the initial
  OAuth response value, then `RUNX_CONNECT_POLL_INTERVAL_MS`, then 750 ms.
- Timeout semantics remain elapsed-time based and return `ConnectError::Timeout`
  only after the same pending-loop boundary as today.
- Sensitive values include authorization headers, access tokens, request
  bodies, provider error text, flow ids in routes, and opener failures.

## Touchpoints

- `crates/runx-runtime/src/connect/client.rs`
- `crates/runx-runtime/src/runtime_http.rs`
- `crates/runx-runtime/src/connect/mod.rs`
- `crates/runx-runtime/Cargo.toml`
- `crates/runx-cli/Cargo.toml` as feature-wiring evidence.
- `crates/Cargo.toml`
- `crates/deny.toml`
- `crates/Cargo.lock`
- `crates/runx-cli/tests/connect.rs`

## Risks

- Calling a blocking bridge from inside an active tokio runtime can panic or
  deadlock unless the parent helper shape is used exactly.
- Reqwest follows redirects by default; the current curl transport does not.
  The cutover must configure parity or preserve no-follow behavior.
- Header casing, content-length handling, and body flushing can change fixture
  expectations if tests overfit to curl's wire format.
- Error messages may accidentally leak bearer tokens, flow ids, provider
  secrets, or request bodies unless all new errors pass through existing
  redaction paths.
- Dependency-deny changes can unintentionally permit async/network crates in
  pure crates if exceptions are too broad.

## Acceptance

Profile: standard

Validation:
- `scafld validate rust-async-http-cutover-connect`
- `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test connect`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_client`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_secret_redaction`
- `cargo test --manifest-path crates/Cargo.toml -p runx-runtime runtime_http::tests`
- `cargo check --manifest-path crates/Cargo.toml -p runx-cli`
- `cargo check --manifest-path crates/Cargo.toml -p runx-runtime --features async-http`
- `cd crates && cargo deny check`
- `rg -n '^async-http = \["dep:reqwest", "dep:tokio"\]|^cli-tool = \["async-http"\]' crates/runx-runtime/Cargo.toml`
- `awk '/^## Acceptance/{exit} /^## Harden Rounds/{exit} {print}' .scafld/specs/drafts/rust-async-http-cutover-connect.md | rg 'go version|Complete the requested change|Implement rust-async-http-cutover-connect' && exit 1 || test $? -eq 1`

## Phase 1: Dependency Review And Feature Gate

Status: completed
Dependencies: .scafld/specs/archive/2026-05/rust-async-http-layer.md

Objective: Introduce the exact reviewed async HTTP dependency set without

Changes:
- Review the current `crates/Cargo.lock` graph and choose exact pins for `reqwest` and `tokio`.
- Add `async-http = ["dep:reqwest", "dep:tokio"]` to `runx-runtime`.
- Wire `cli-tool = ["async-http"]` in `runx-runtime`; `runx-cli` already enables `cli-tool`, so the cargo-built binary uses the reqwest transport.
- Scope `crates/deny.toml` changes so pure crates remain free of `tokio`, `reqwest`, `hyper`, and raw network clients.
- Record per-crate license evidence for every new direct and relevant transitive crate, preferring package-specific exceptions over broad license allowlist changes.

Acceptance:
- [x] `ac1` command - Spec remains valid
  - Command: `scafld validate rust-async-http-cutover-connect`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-13
- [x] `ac2` command - Supply-chain policy passes with all features
  - Command: `cd crates && cargo deny check`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-14
- [x] `ac3` inspection - Exact dependency pins and feature shape
  - Command: `sh -c "rg -F 'async-http = [\"dep:reqwest\", \"dep:tokio\"]' crates/runx-runtime/Cargo.toml && rg -F 'cli-tool = [\"async-http\"]' crates/runx-runtime/Cargo.toml && rg -F 'reqwest = { version = \"=' crates/runx-runtime/Cargo.toml && ! rg -n 'default-features = true|reqwest::blocking|features = \\[[^]]*blocking|features = \\[[^]]*cookies|features = \\[[^]]*stream' crates/runx-runtime/Cargo.toml"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-15
- [x] `ac3b` inspection - CLI feature edge reaches the async HTTP runtime
  - Command: `sh -c "rg -F 'cli-tool = [\"async-http\"]' crates/runx-runtime/Cargo.toml && rg -F 'runx-runtime = { workspace = true, features = [\"cli-tool\", \"mcp\"] }' crates/runx-cli/Cargo.toml"`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-16

## Phase 2: Runtime Bridge And Reqwest Transport

Status: completed
Dependencies: Phase 1

Objective: Implement reqwest transport behind `HostedTransport` without

Changes:
- Add the parent spec's `async_runtime()` and `block_on_http()` helper shape behind `async-http`.
- Return typed runtime errors instead of using `unwrap`, `expect`, `panic`, or `println` in the helper or transport path.
- If `tokio::runtime::Handle::try_current()` succeeds, return a typed `BlockingHttpInsideAsyncRuntime`-style error rather than blocking.
- Build a reqwest client with redirect following disabled.
- Map `HttpMethod::{Get,Post,Delete}`, headers, optional string body, status, and response body into the existing hosted HTTP request/response model.
- Keep header validation and URL scheme validation before sending.

Acceptance:
- [x] `ac4` command - Runtime hosted HTTP tests pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime runtime_http::tests`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-21
- [x] `ac5` inspection - Blocking bridge is panic-free
  - Command: `sh -c '! rg -n "unwrap\\(|expect\\(|panic!|println!" crates/runx-runtime/src -g "hosted_http.rs" -g "http_runtime.rs" -g "runtime_http.rs"'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-22
- [x] `ac5b` inspection - Nested-runtime failures are typed, not panics
  - Command: `rg -n 'BlockingHttpInsideAsyncRuntime|AsyncRuntimeUnavailable' crates/runx-runtime/src`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-23

## Phase 3: Connect Cutover And Parity

Status: completed
Dependencies: Phase 2

Objective: Move connect client traffic to the reqwest-backed transport while

Changes:
- Wire `ConnectClient::new` to the reqwest-backed default transport for the cutover configuration: `cli-tool` implies `async-http`.
- Keep `ConnectClient<T, O>` generic so tests can inject fixtures and opener behavior.
- Preserve auth headers: `authorization`, `accept: application/json`, `content-type: application/json`.
- Preserve JSON validation and supported status handling for list, revoke, preprovision, and OAuth flow polling.
- Preserve `safe_route` behavior for flow ids and `redact_connect_text` for provider errors, opener failures, unsupported statuses, and HTTP errors.
- Add or update fixture parity coverage for: list empty output, revoke JSON, revoke human output, preprovision created, preprovision unchanged, OAuth required then pending then created, OAuth failed, OAuth timeout, HTTP non-2xx redaction, invalid JSON, and unsupported status redaction.

Acceptance:
- [x] `ac6` command - Existing connect CLI fixtures pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-cli --test connect`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-28
- [x] `ac7` command - Connect redaction tests pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_secret_redaction`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-29
- [x] `ac7b` command - Runtime connect client tests pass
  - Command: `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_client`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-30
- [x] `ac8` inspection - OAuth polling precedence and timeout semantics are unchanged
  - Command: `rg -n 'poll_after_ms|RUNX_CONNECT_POLL_INTERVAL_MS|ConnectError::Timeout|timeout_ms' crates/runx-runtime/src/connect/client.rs crates/runx-runtime/tests/connect_client.rs crates/runx-cli/tests/connect.rs`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-31
- [x] `ac9` inspection - Secrets remain redacted
  - Command: `sh -c 'rg -n "SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK|redact_connect_text|safe_route|\\[redacted\\]" crates/runx-runtime/src crates/runx-runtime/tests crates/runx-cli/tests/connect.rs && ! rg -n "println!|eprintln!" crates/runx-runtime/src/connect crates/runx-cli/src/connect.rs'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-32

## Phase 4: Draft Guardrails

Status: completed
Dependencies: none

Objective: Keep this spec executable and prove the scaffold placeholders were

Changes:
- Do not approve or implement in this draft task.
- Keep acceptance commands concrete and runnable from `oss/`.

Acceptance:
- [x] `ac10` command - Scaffold command placeholder is absent
  - Command: `sh -c '! rg -n "g[o] version" .scafld/specs/drafts/rust-async-http-cutover-connect.md'`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-37
- [x] `ac11` command - Scaffold objective placeholder is absent
  - Command: `awk '/^## Acceptance/{exit} /^## Harden Rounds/{exit} {print}' .scafld/specs/drafts/rust-async-http-cutover-connect.md | rg 'go version|Complete the requested change|Implement rust-async-http-cutover-connect' && exit 1 || test $? -eq 1`
  - Expected kind: `exit_code_zero`
  - Status: pass
  - Evidence: exit code was 0
  - Source event: entry-38

## Rollback

- Before merge, revert the full async HTTP cutover patch as one change if the
  reqwest transport regresses connect behavior; this restores the deleted curl
  module from git history instead of leaving a compatibility shim.
- Remove the `async-http` feature, `reqwest` and `tokio` dependency entries,
  and any deny/license exceptions introduced by the implementation commit.
- Restore `crates/Cargo.lock` to the pre-cutover graph and rerun
  `cd crates && cargo deny check`.
- Keep connect fixture tests as regression coverage if they expose a real
  behavior gap.

## Review

Status: completed
Verdict: pass
Mode: verify
Provider: claude:claude-opus-4-7
Output: claude.mcp_submit_review
Summary: Verify-mode re-review of rust-async-http-cutover-connect. The prior critical blocker `workspace_mutation` is no longer present: Workspace Baseline reports `clean`, Task Changes Since Approval Baseline reports `none`, and ambient drift is attributed to sibling specs (hosted-http-removal, registry) outside this task's declared scope. The four prior findings are confirmed fixed against current source: (1) Rollback section at .scafld/specs/active/rust-async-http-cutover-connect.md:320-330 references the curl module via git history rather than the deleted CommandHttpTransport type (zero matches in crates/); (2) crates/deny.toml:22 bans `tokio` with the approved wrapper allowlist `[runx-runtime, reqwest, hyper, hyper-rustls, hyper-util, tokio-rustls, tower]`; (3) crates/runx-runtime/src/runtime_http.rs:110-127 configures explicit `request_timeout=30s, connect_timeout=10s` with a `reqwest_transport_times_out_stalled_response` regression test bound to a stalling TCP server; (4) prior workspace_mutation hash drift no longer occurs. Re-walked the cutover surface: feature edge `async-http = ["dep:reqwest", "dep:tokio"]` and `cli-tool = ["async-http"]` at runx-runtime/Cargo.toml:20-21; reqwest pin `=0.13.3` with `default-features=false, features=["rustls", "json"]`; tokio pin `=1.52.3` with `["rt", "net", "time"]`; no `blocking|cookies|stream|default-features=true` or direct `hyper` dependency; runx-cli enables `["cli-tool", "mcp"]` so the cargo-built binary exercises reqwest end-to-end. `ConnectClient<T, O>` stays generic at connect/client.rs:47-54; `with_transport_and_opener` is feature-agnostic so MockConnectTransport/RecordingOpener test injection continues to work; the `cfg(feature = "async-http")`-gated `ConnectClient::new` at connect/client.rs:69-84 is the only seam that pulls in ReqwestHttpTransport. `block_on_http` at runtime_http.rs:290-304 short-circuits with `Handle::try_current()` returning typed `BlockingHttpInsideAsyncRuntime`, and runtime build failure maps to `AsyncRuntimeUnavailable`; no `unwrap!|expect!|panic!|println!` in the helper or transport path (grep confirms zero matches). OAuth polling precedence (pending.poll_after_ms → initial → poll_interval_ms → 750ms) and elapsed-time timeout semantics are preserved at connect/client.rs:175-185. Header CRLF rejection and URL scheme guard pre-empt the reqwest send. HostedHttpResponse/Header/Request Debug impls redact sensitive headers, bodies (shown as "[redacted body present]"), and body bytes ("{n} bytes"); ConnectClientOptions Debug redacts access_token. No new task-scope regressions detected; ambient drift remains classified as context-only per the workspace classifier contract. Note: the spec's Summary language ("This draft does not add dependencies, edit Cargo manifests, or change Rust source.") and Out-of-scope item ("Implementing the code change in this task.") are stale against the now-completed phases, but this is documentary-only and was not raised by the prior review.

Attack log:
- `workspace_mutation critical blocker`: Confirm prior workspace-mutation blocker is no longer present; verify Workspace Baseline and task_changes attribution -> clean (Workspace Baseline = clean; Task Changes Since Approval Baseline = none; ambient drift attributable to sibling cutover specs outside scope.)
- `scope-drift-rollback-references-deleted-type`: Read current Rollback section and grep crates/ for CommandHttpTransport -> clean (Rollback lines 320-330 reference git history; zero CommandHttpTransport matches in crates/.)
- `deny-toml-tokio-not-listed`: Re-read deny.toml ban list and verify tokio wrapper allowlist matches Cargo.lock dependents -> clean (deny.toml:22 bans tokio with [runx-runtime, reqwest, hyper, hyper-rustls, hyper-util, tokio-rustls, tower]; matches Cargo.lock dependents; ac2 passes.)
- `reqwest-no-per-request-timeout`: Trace ReqwestHttpTransport::new to confirm explicit per-request and connect timeouts plus regression coverage -> clean (runtime_http.rs:110-127 sets 30s request and 10s connect timeouts; reqwest_transport_times_out_stalled_response asserts bounded failure path.)
- `Regression hunt — CLI feature edge still threads reqwest`: Walk runx-cli/Cargo.toml → cli-tool → async-http → ConnectClient::new gating -> clean (runx-cli/Cargo.toml:20 enables [cli-tool, mcp]; runx-runtime/Cargo.toml:21 has cli-tool=[async-http]; ConnectClient::new at connect/client.rs:69 is cfg(feature='async-http')-gated. CLI integration tests exercise the reqwest binary.)
- `Regression hunt — ConnectClient generic injection`: Confirm with_transport_and_opener is feature-agnostic so MockConnectTransport tests still run -> clean (connect/client.rs:86-108 with_transport_and_opener is not feature-gated; tests/connect_client.rs and tests/connect_secret_redaction.rs continue to inject &MockConnectTransport + &RecordingOpener. ac7/ac7b pass.)
- `Convention check — forbidden patterns in transport path`: Grep unwrap/expect/panic!/println! in runtime_http.rs -> clean (Grep returns zero matches; ac5 negative grep over hosted_http.rs/http_runtime.rs/runtime_http.rs passes.)
- `Dark patterns — header CRLF and URL scheme smuggling`: Trace validate_header and validate_http_url; confirm both pre-empt reqwest send -> clean (validate_header rejects \r/\n in values and non-token name bytes; validate_http_url runs before block_on_http for transport.send and inside HostedHttpClient::with_transport. Dedicated tests cover both paths.)
- `Dark patterns — secret leakage across Debug/Display`: Audit Display/Debug impls for ConnectError, HostedHttpResponse, HostedHttpHeader, HttpConnectStart/Flow responses -> clean (ConnectError::HttpStatus uses http_error_message which redacts JSON-error bodies via redact_connect_text and otherwise reports only byte length. HostedHttpResponse Debug shows '{n} bytes'. HostedHttpHeader Debug redacts authorization/proxy-authorization/*token*/*secret*/*api-key*. ConnectClientOptions Debug redacts access_token. connect_secret_redaction tests cover the suite.)
- `Dark patterns — block_on inside an active tokio runtime`: Verify block_on_http checks Handle::try_current() and surfaces typed BlockingHttpInsideAsyncRuntime / AsyncRuntimeUnavailable -> clean (runtime_http.rs:294 checks Handle::try_current() and returns HostedHttpError::BlockingHttpInsideAsyncRuntime; runtime build failures map to AsyncRuntimeUnavailable. Grep confirms both variant names exist in src.)
- `Spec compliance — OAuth polling precedence and timeout semantics`: Read wait_for_connect_flow and verify pending.poll_after_ms → initial → poll_interval_ms → 750ms order and elapsed-time timeout -> clean (connect/client.rs:178-181 selects delay_ms via `.or(initial_poll_after_ms).or(self.poll_interval_ms).unwrap_or(750)`; timeout_ms defaults to 60_000 (line 106); started_at.elapsed() compared at pending boundary (line 175).)
- `Spec compliance — dependency pins and feature shape`: Re-run textual checks of ac3 against current Cargo.toml -> clean (async-http = ["dep:reqwest", "dep:tokio"] and cli-tool = ["async-http"] at runx-runtime/Cargo.toml:20-21; reqwest pin =0.13.3 with default-features=false features [rustls, json]; tokio pin =1.52.3 with [rt, net, time]; no blocking/cookies/stream/default-features=true/direct hyper.)

Findings:
- [critical/non-blocking] `workspace_mutation` Prior workspace_mutation blocker no longer reproduces.
  - Location: `.scafld/specs/active/rust-async-http-cutover-connect.md`
  - Evidence: Workspace Baseline section reports `clean`; Task Changes Since Approval Baseline reports `none`. Ambient drift is attributed to sibling cutover specs (hosted-http-removal, registry) outside this task's declared scope. Re-checked Cargo.toml:34 reqwest pin and runtime_http.rs against approval baseline — no further drift during this verify pass.
  - Impact: None — the blocker condition is no longer present and the verify-pass baseline is clean.
  - Validation: Workspace Baseline = clean; ambient_drift attribution is sibling-spec ownership; no in-flight edits to task-scope files during review.
- [low/non-blocking] `scope-drift-rollback-references-deleted-type` Rollback section no longer references the deleted CommandHttpTransport type.
  - Location: `.scafld/specs/active/rust-async-http-cutover-connect.md:320`
  - Evidence: .scafld/specs/active/rust-async-http-cutover-connect.md lines 320-330 describe a git-history-based revert ("this restores the deleted curl module from git history instead of leaving a compatibility shim"). `rg CommandHttpTransport crates/` returns zero matches; only historical Review/Harden log entries inside this spec mention the old type, which is expected.
  - Validation: Read Rollback block lines 320-330; grep across crates/ confirms zero source references to CommandHttpTransport.
- [low/non-blocking] `deny-toml-tokio-not-listed` deny.toml now bans tokio with the approved wrapper allowlist.
  - Location: `crates/deny.toml:22`
  - Evidence: crates/deny.toml line 22: `{ name = "tokio", wrappers = ["runx-runtime", "reqwest", "hyper", "hyper-rustls", "hyper-util", "tokio-rustls", "tower"], reason = "Approved only inside runx-runtime async-http and reviewed reqwest internals; pure crates must not depend on tokio." }`. Wrapper set matches actual Cargo.lock dependents and ac2 (`cd crates && cargo deny check`) passes with `all-features = true`.
  - Validation: Read deny.toml lines 13-24; ac2 recorded exit code 0.
- [low/non-blocking] `reqwest-no-per-request-timeout` ReqwestHttpTransport configures explicit request and connect timeouts with regression coverage.
  - Location: `crates/runx-runtime/src/runtime_http.rs:110`
  - Evidence: runtime_http.rs:110-127 — `ReqwestHttpTransport::new()` delegates to `with_timeouts(Duration::from_secs(30), Duration::from_secs(10))`; the builder applies `.timeout(request_timeout).connect_timeout(connect_timeout)`. Regression test `reqwest_transport_times_out_stalled_response` at runtime_http.rs:491-520 binds a stalling TCP listener and asserts `HostedHttpError::Transport`, bounding hung initial connect requests independently of the OAuth pending-loop timeout.
  - Validation: Read runtime_http.rs lines 110-127 and 491-520; ac4 (`runtime_http::tests`) passes per recorded evidence.

## Self Eval

- Draft is scoped to the connect client async HTTP cutover and does not
  request implementation in this task.
- Acceptance uses commands that are meaningful from `oss/`.
- The dependency, deny, license, bridge, fixture, redaction, OAuth, and timeout
  requirements from the parent spec are explicit.

## Deviations

- none

## Metadata

- created_by: scafld
- parent_spec: `.scafld/specs/archive/2026-05/rust-async-http-layer.md`
- implementation_kind: future_spec

## Origin

Created by: scafld
Source: plan



## Planning Log

- 2026-05-21: Replaced scaffold template with an executable draft handoff for
  the connect client async HTTP cutover.

## Harden Rounds

### round-1

Status: needs_revision
Started: 2026-05-21T02:12:48Z
Ended: 2026-05-21T02:12:48Z
Verdict: needs_revision
Provider: claude
Model: claude-opus-4-7
Output format: claude.mcp_submit_harden
Summary: Draft scopes the connect cutover correctly and inherits a clean dependency-policy contract from the parent spec, but two issues prevent safe approval: (1) Phase 2 acceptance `ac5` is inverted — `rg -n 'unwrap\(|expect\(|panic!|println!' …/hosted_http.rs` exits 0 only when bad patterns ARE found, so the gate passes precisely when the helper is *not* panic-free; (2) the feature-gating decision the parent harden explicitly deferred to this cutover (harden-4 in archive/rust-async-http-layer) is still unresolved: Phase 1 says deps stay "optional and disabled by default", Phase 3 says `ConnectClient::new` uses the reqwest-backed default "for the cutover configuration", and runx-cli/Cargo.toml currently enables only `cli-tool,mcp`. As written the cargo-installed binary keeps the curl path while the CLI fixture suite (`-p runx-cli --test connect`) executes the real `runx` binary — so the headline parity gate cannot actually exercise the new transport. Several inspection acceptance checks (`ac3`, `ac8`, `ac9`) are pure token-presence and already pass on unmodified source, weakening the signal that the cutover landed; and the top-level Acceptance lists `--test connect_client` which no phase gates. Recommend fixing ac5 inversion, naming the explicit feature wiring that makes `runx connect` actually call reqwest under the CLI integration tests, tightening the inspection patterns to anchored shape checks, and aligning the per-phase acceptance with the top-level list.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/runtime_http.rs:1
  - Result: passed
  - Evidence: All declared touchpoints exist: crates/runx-runtime/src/runtime_http.rs (Read confirms HostedTransport trait and CommandHttpTransport at lines 98-216), crates/runx-runtime/src/connect/client.rs (ConnectClient::new at line 69-83 uses CommandHttpTransport::new), crates/runx-runtime/src/connect/mod.rs (Read confirms surface re-exports at lines 6-19), crates/runx-runtime/Cargo.toml (no reqwest/tokio today, lines 26-38), crates/deny.toml (reqwest+tokio currently banned at lines 18,22), crates/Cargo.toml (workspace lints expect_used/panic/unwrap_used = deny at lines 38-44), crates/runx-cli/tests/connect.rs (TCP-fixture integration tests exist). Parent spec .scafld/specs/archive/2026-05/rust-async-http-layer.md is present and marked completed.
- command audit
  - Grounded in: code:crates/runx-runtime/src/runtime_http.rs:1
  - Result: failed
  - Evidence: Phase 2 `ac5` command `rg -n 'unwrap\(|expect\(|panic!|println!' crates/runx-runtime/src/runtime_http.rs` with Expected kind: exit_code_zero is inverted: ripgrep exits 0 on a successful MATCH and 1 on no match. The objective ('Blocking bridge is panic-free') requires NO match → rg exit 1 → acceptance fail under the current expectation. Grep confirmed the file currently contains none of these patterns (No matches found), so ac5 fails today; after the cutover, if the helper is correctly panic-free the gate will still fail. The sibling ac10 uses `sh -c '! rg …'` — ac5 needs the same inversion. Phase 1 `ac3`, Phase 3 `ac8` and `ac9` are valid invocations but their patterns (`reqwest|tokio|default-features|…`, `poll_after_ms|thread::sleep|…`, `redact_connect_text|authorization|…`) all match the CURRENT source unchanged (verified by inspection of Cargo.toml line 41 `default-features = false` in proptest, deny.toml's existing reqwest/tokio entries, and connect/client.rs lines 158-184/269-275). They pass without any cutover happening — weak signal. Top-level Acceptance lists `cd crates && cargo deny check` while Phase 1 ac2 lists `cd crates && cargo deny check`; both forms work but only the subcommand-first form is canonical in cargo-deny docs.
- scope/migration audit
  - Grounded in: code:crates/runx-cli/Cargo.toml:20
  - Result: failed
  - Evidence: Scope says `ConnectClient::new` is wired to a reqwest default `for the cutover configuration` but never specifies the configuration. runx-cli/Cargo.toml line 20 enables only features `cli-tool, mcp`, not `async-http`. With `async-http = ["dep:reqwest", "dep:tokio"]` kept optional and disabled by default (Phase 1, line 168-172 in draft), the cargo-installed `runx` binary continues to compile with the curl-only CommandHttpTransport — and `crates/runx-cli/tests/connect.rs` execs the real built binary via `Command::new(env!("CARGO_BIN_EXE_runx"))` (lines 104-127), so the CLI fixture suite would still drive curl. Two coherent shapes exist (cli-tool implies async-http; OR async-http feature flips ConnectClient::new at compile time so non-async-http builds keep curl) and the spec must pick one. Parent spec harden-4 already flagged this as 'the consumer-wiring decision is owned by each cutover spec' (archive/rust-async-http-layer.md round-1 issue harden-4) — but this draft re-defers it. Out-of-scope item 'Removing hosted_http.rs or deleting the curl transport entirely' is correctly reserved for the third cutover.
- acceptance timing audit
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:152
  - Result: failed
  - Evidence: Top-level Acceptance (line 152-154 of draft) lists `cargo test --manifest-path crates/Cargo.toml -p runx-runtime --test connect_client`, but no phase gates it (Phase 3 acceptance ac6/ac7/ac8/ac9 list only `runx-cli --test connect` and `runx-runtime --test connect_secret_redaction`). The test file does exist (crates/runx-runtime/tests/connect_client.rs uses MockConnectTransport so it's transport-agnostic and would not actually detect reqwest-related drift). Phase 3 enumerates 11 fixture-parity scenarios (list empty, revoke JSON, revoke human, preprovision created/unchanged, OAuth required→pending→created, OAuth failed, OAuth timeout, HTTP non-2xx redaction, invalid JSON, unsupported status) but the actual gates only run the existing test binaries; there is no acceptance step that verifies every scenario is represented. ac8 and ac9 are token-presence checks that already pass on unmodified source and therefore provide no timing signal.
- rollback/repair audit
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:290
  - Result: passed
  - Evidence: Rollback (line 290-295) names a credible sequence: revert default transport to CommandHttpTransport (still present per hosted_http.rs lines 102-216), drop the async-http feature, drop reqwest/tokio entries, drop deny/license exceptions, and restore crates/Cargo.lock pre-cutover. Curl transport is preserved through this cutover per out-of-scope ('Removing hosted_http.rs … entirely'). The rollback path is reversible because Phase 1 keeps deps optional, so reverting the cutover does not break unrelated callers. One omission: rollback does not name a verification step (e.g., re-run `cargo deny check` after restoring deny entries to confirm no transitive hyper/reqwest leaked). Parent spec recommended that step but this cutover did not inherit it. Minor — not a blocker.
- design challenge
  - Grounded in: code:.scafld/specs/archive/2026-05/rust-async-http-layer.md:165
  - Result: failed
  - Evidence: Architecturally this is the right move (cutover-by-call-site behind the parent spec's panic-free helper), but several decisions are still floating. (1) The async_runtime/block_on_http helper module location is unnamed — Phase 2 says 'Add the parent spec's helper shape behind async-http' but does not specify whether it lives in hosted_http.rs (in which case ac5's pattern check applies) or a new private module (in which case ac5 audits the wrong file). (2) Phase 3 promises to keep `ConnectClient<T,O>` generic but does not name the concrete reqwest transport type or where it lives, so reviewers cannot grep for it. (3) Parent spec line 165-212 mandates `Handle::try_current()` short-circuit; this draft restates the requirement but does not call out a concrete error variant name in error.rs (RuntimeError::BlockingHttpInsideAsyncRuntime / AsyncRuntimeUnavailable) that the implementation must use — risking re-litigation. (4) No statement on whether `cargo deny check` runs with `--all-features` to actually exercise the async-http feature graph (deny.toml line 5 already sets all-features=true workspace-wide, but the spec should make this explicit since the supply-chain risk is feature-gated).

Issues:
- [critical/blocks approval] `harden-1` acceptance_inversion - Phase 2 ac5 inspection is inverted: passes when panics ARE present, fails when the helper is panic-free.
  - Status: fixed
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:218
  - Evidence: Draft line 219: `rg -n 'unwrap\(|expect\(|panic!|println!' crates/runx-runtime/src/runtime_http.rs` with Expected kind: exit_code_zero. ripgrep exits 0 only when it finds a match; the stated objective ('Blocking bridge is panic-free') requires NO matches → exit 1. Grep confirmed the file currently contains no such patterns (No matches found), so ac5 fails today and would continue to fail after a correct cutover, while a defective implementation that introduced a `.unwrap()` would PASS ac5. Compare sibling ac10 which correctly inverts via `sh -c '! rg …'`.
  - Recommendation: Rewrite ac5 as `sh -c '! rg -n "unwrap\(|expect\(|panic!|println!" crates/runx-runtime/src/runtime_http.rs'` (matching ac10's pattern) and explicitly cover the helper module if it lives outside hosted_http.rs. Add a positive companion check that the helper returns a typed `RuntimeError` variant (e.g., `rg -n 'BlockingHttpInsideAsyncRuntime|AsyncRuntimeUnavailable' crates/runx-runtime/src` expected non-empty).
  - Question: Should the ac5 pattern check be inverted and extended to cover the helper module wherever it lands?
  - Recommended answer: Yes. Use `! rg …` and audit both hosted_http.rs and the new async-http helper file. Also assert the structured error variants exist.
  - If unanswered: Default to inverting ac5 with `sh -c '! rg …'` matching ac10 and add an explicit positive check for the BlockingHttpInsideAsyncRuntime variant.
- [high/blocks approval] `harden-2` feature_gating - Cutover does not specify how `runx connect` actually reaches the reqwest transport; CLI fixture suite would still execute curl.
  - Status: fixed
  - Grounded in: code:crates/runx-cli/Cargo.toml:20
  - Evidence: runx-cli/Cargo.toml line 20 enables only `cli-tool, mcp`. Phase 1 keeps `async-http` optional and disabled by default; Phase 3 says `ConnectClient::new` is wired to the reqwest default `for the cutover configuration` but never defines that configuration. crates/runx-cli/tests/connect.rs lines 104-127 spawn the real `runx` binary via `env!("CARGO_BIN_EXE_runx")` and assert wire-level fixtures over a TCP socket. Without an explicit feature decision, that binary still uses CommandHttpTransport, so ac6 (`-p runx-cli --test connect`) never actually exercises reqwest — the cutover headline gate becomes a tautology. Parent spec (archive/rust-async-http-layer.md round-1 harden-4) flagged this and explicitly deferred to the cutover; this draft re-defers without resolving.
  - Recommendation: Pick one of two shapes and write it into Phase 1: (a) `cli-tool = ["async-http"]` so the cargo-installed binary always uses reqwest and `runx-cli/Cargo.toml` picks it up transitively; OR (b) keep `async-http` orthogonal but ship two `ConnectClient::new` constructors gated by `#[cfg(feature = "async-http")]` and have the CLI binary explicitly enable `async-http`. Then add an acceptance gate that the runx binary under test was built with the async transport (e.g., a startup banner string in `runx --version` or a `cargo metadata` feature assertion in tests).
  - Question: Should this cutover make `cli-tool` imply `async-http` so the cargo-installed runx always uses reqwest, or keep async-http orthogonal with explicit CLI-feature enabling?
  - Recommended answer: Make `cli-tool` imply `async-http` here. The cargo-installed launcher is the only consumer that exercises the connect path in CI, so anything else leaves the new transport untested. Phase 1 should record the Cargo.toml diff: `cli-tool = ["async-http"]`.
  - If unanswered: Default to `cli-tool` implies `async-http`, and add ac3 patterns to assert that exact mapping.
- [medium/advisory] `harden-3` acceptance_coverage - Top-level Acceptance references `--test connect_client` but no phase gates it; Phase 3 fixture list is 11 scenarios with only two test-binary gates.
  - Status: fixed
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:152
  - Evidence: Acceptance section line 152-154 includes `-p runx-runtime --test connect_client`; Phase 3 acceptance ac6/ac7/ac8/ac9 (lines 249-263) gates only `-p runx-cli --test connect` and `-p runx-runtime --test connect_secret_redaction`. connect_client.rs exists (Read confirms 80-line preview) and uses MockConnectTransport from connect_support.rs — meaning it would pass with or without the reqwest cutover and provides no transport-parity signal. Phase 3 enumerates 11 fixture-parity scenarios (list empty, revoke JSON/human, preprovision created/unchanged, OAuth required→pending→created, OAuth failed, OAuth timeout, HTTP non-2xx redaction, invalid JSON, unsupported status) but no gate enumerates them.
  - Recommendation: Either drop `connect_client` from top-level Acceptance (it adds no transport signal), or add it to Phase 3 as ac6.5 and explicitly document the scope-of-coverage table mapping each of the 11 scenarios to a named test in connect.rs / connect_secret_redaction.rs (e.g., a coverage matrix in Phase 3 notes).
  - Question: Drop `connect_client` from top-level Acceptance, or move it into Phase 3 and require the missing fixture scenarios to land there?
  - Recommended answer: Move it into Phase 3 and require a coverage matrix. The 11 scenarios are real parity risks (especially OAuth timeout and unsupported-status redaction); each one should have a named test, not a vague mention.
  - If unanswered: Default to adding `connect_client` to Phase 3 and including a 11-row scenario→test mapping table in the phase description.
- [medium/advisory] `harden-4` acceptance_strength - ac3, ac8, ac9 are token-presence checks that already pass on unmodified source and give no cutover signal.
  - Status: fixed
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:188
  - Evidence: ac3 pattern `reqwest|tokio|async-http|default-features|rustls|blocking|cookies|stream` matches today: deny.toml lines 18,22 contain reqwest/tokio bans (text presence), runx-runtime/Cargo.toml line 41 contains `default-features = false` (proptest). ac8 pattern `poll_after_ms|poll_interval_ms|timeout_ms|ConnectError::Timeout|thread::sleep|tokio::time` matches today: connect/client.rs lines 158-184 reference all of these. ac9 pattern `SECRET_CONNECT_ACCESS_TOKEN_DO_NOT_LEAK|redact_connect_text|\[redacted\]|safe_route|authorization` matches today: connect/client.rs lines 269-275 / connect/redaction.rs / connect_secret_redaction.rs. Therefore none of these gates verify the cutover landed.
  - Recommendation: Tighten patterns to anchored shape: ac3 should require `reqwest = \{ version = "=\d+\.\d+\.\d+"`, `tokio = \{ version = "=`, `default-features = false`, features including `rustls` and excluding `blocking|cookies|stream` (e.g., a negative grep `! rg 'reqwest = .*"blocking"'`). ac8 should assert removal of `thread::sleep` if the cutover moves polling to tokio::time, or its retention if not — a yes/no decision, not a presence check. ac9 should run a redaction test (already done via ac7) plus a negative-text-leak check (`! rg -n 'SECRET_CONNECT_ACCESS_TOKEN' crates/runx-runtime/target/test-output …`).
  - Question: Tighten these three inspections to shape/negative checks that actually fail before the cutover and pass only after?
  - Recommended answer: Yes. Replace text-presence with anchored shape patterns plus paired negative greps. The exact list belongs in Phase 1/3 once the dependency pins are chosen.
  - If unanswered: Default to anchored shape patterns for ac3 and convert ac8/ac9 to paired positive+negative gates as outlined.
- [low/advisory] `harden-5` design_specificity - Helper module location and error-variant names are unspecified, so ac5 may audit the wrong file and reviewers may re-litigate the variant names.
  - Status: fixed
  - Grounded in: code:.scafld/specs/archive/2026-05/rust-async-http-layer.md:198
  - Evidence: Phase 2 says 'Add the parent spec's async_runtime() and block_on_http() helper shape behind async-http' but does not specify the module path. ac5 only inspects hosted_http.rs. Parent spec archive/rust-async-http-layer.md lines 198-206 names the helper but does not pin a module location; round-1 harden-2/harden-5 recommended `RuntimeError::AsyncRuntimeUnavailable` and `RuntimeError::BlockingHttpInsideAsyncRuntime` variants, but those names are not pinned in either spec. Without a chosen module path, the cutover risks landing the helper in a place that ac5 doesn't audit; without pinned variant names, the implementation may invent its own names and the rmcp adoption spec may need to re-decide.
  - Recommendation: Pin the helper module path (e.g., `crates/runx-runtime/src/hosted_http/async_bridge.rs` or a top-level `crates/runx-runtime/src/async_http.rs`), pin the two error variants (`RuntimeError::AsyncRuntimeUnavailable { message }` and `RuntimeError::BlockingHttpInsideAsyncRuntime`), and extend ac5's path list to cover the chosen module.
  - Question: Where does the async_runtime/block_on_http helper live, and what are the exact error variant names?
  - Recommended answer: Helper at `crates/runx-runtime/src/hosted_http/async_bridge.rs` (cfg-gated by async-http). Variants `RuntimeError::AsyncRuntimeUnavailable { message: String }` and `RuntimeError::BlockingHttpInsideAsyncRuntime`. Extend ac5 to grep both files.
  - If unanswered: Default to the path and variant names above; record them in the Phase 2 Changes block.
- [low/advisory] `harden-6` consistency - Inconsistent `cargo deny` invocation form between top-level Acceptance and Phase 1 ac2; missing `--all-features` makes feature-gated deps go unchecked.
  - Status: fixed
  - Grounded in: code:.scafld/specs/drafts/rust-async-http-cutover-connect.md:156
  - Evidence: Top-level Acceptance line 156: `cd crates && cargo deny check`. Phase 1 ac2 (line 187): `cd crates && cargo deny check`. Both forms work but only the latter is canonical per cargo-deny docs. deny.toml line 5 sets `all-features = true` so cargo-deny crawls all features by default — fine, but the spec should restate this so the reviewer doesn't doubt that `async-http` is actually exercised by the supply-chain check. Rollback (line 290-295) also does not call out re-running `cargo deny check` after reverting deny entries; small omission.
  - Recommendation: Use the canonical form `cd crates && cargo deny check` in both places. Add a one-line acceptance/Phase 1 note that deny.toml's `all-features = true` is what makes this check audit the new async-http feature graph. Add to Rollback: 're-run cargo deny check after restoring deny entries to confirm no transitive hyper or reqwest leaked'.
  - Question: Normalize the cargo-deny invocation form and add the deny check to Rollback verification?
  - Recommended answer: Yes. Single canonical form and verify rollback with the same cargo deny check.
  - If unanswered: Default to the canonical subcommand-first form everywhere and append the verification step to Rollback.

### round-2

Status: passed
Started: 2026-05-21T05:20:00Z
Ended: 2026-05-21T05:35:00Z
Verdict: passed
Provider: local
Model: codex
Output format: manual_resolution
Summary: Resolved the round-1 connect cutover harden blockers. The draft now

Checks:
- command audit
  - Grounded in: spec:rust-async-http-cutover-connect.md#Acceptance
  - Result: passed
  - Evidence: `scafld validate rust-async-http-cutover-connect`, the
- scope/migration audit
  - Grounded in: spec:rust-async-http-cutover-connect.md#Scope
  - Result: passed
  - Evidence: Scope now includes CLI feature-wiring evidence, and the feature
- acceptance timing audit
  - Grounded in: spec:rust-async-http-cutover-connect.md#Phase 2
  - Result: passed
  - Evidence: `ac5` now uses a negative grep, `ac5b` requires typed
- design challenge
  - Grounded in: spec:rust-async-http-cutover-connect.md#Phase 1
  - Result: passed
  - Evidence: The cutover no longer leaves the parent feature decision open;

Issues:
- [critical/blocks approval] `harden-1` acceptance_inversion - No-panic grep was inverted.
  - Status: fixed
- [high/blocks approval] `harden-2` feature_gating - CLI tests did not prove reqwest.
  - Status: fixed
- [medium/advisory] `harden-3` acceptance_coverage - Runtime connect tests were not phased.
  - Status: fixed
- [medium/advisory] `harden-4` acceptance_strength - Inspection gates were token-only.
  - Status: fixed
- [low/advisory] `harden-5` design_specificity - Helper/variant expectations were loose.
  - Status: fixed
- [low/advisory] `harden-6` consistency - Supply-chain invocation/rollback notes were loose.
  - Status: fixed

### round-3

Status: passed
Started: 2026-05-21T05:36:00Z
Ended: 2026-05-21T02:30:42Z
Verdict: passed
Provider: local
Model: codex
Output format: manual_resolution
Summary: Final harden evidence after the connect cutover draft patch.

Checks:
- path audit
  - Grounded in: code:crates/runx-runtime/src/connect/client.rs:1
  - Result: passed
  - Evidence: The draft targets the existing connect client, hosted HTTP
- command audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: `scafld validate rust-async-http-cutover-connect`, the
- scope/migration audit
  - Grounded in: spec_gap:scope
  - Result: passed
  - Evidence: Scope now includes CLI feature-wiring evidence and makes the
- acceptance timing audit
  - Grounded in: spec_gap:acceptance
  - Result: passed
  - Evidence: The inverted no-panic grep is fixed, the runtime connect test is
- rollback/repair audit
  - Grounded in: spec_gap:rollback
  - Result: passed
  - Evidence: Rollback still restores the curl default, removes async deps and
- design challenge
  - Grounded in: archive:rust-async-http-layer
  - Result: passed
  - Evidence: The child cutover now resolves the parent spec's deferred feature

Issues:
- [critical/blocks approval] `harden-1` acceptance_inversion - No-panic grep was inverted.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: `ac5` now uses a negative grep over hosted HTTP/helper modules.
  - Recommendation: Keep no-panic checks negative and scoped to helper files.
- [high/blocks approval] `harden-2` feature_gating - CLI tests did not prove reqwest.
  - Status: fixed
  - Grounded in: code:crates/runx-cli/Cargo.toml:20
  - Evidence: The draft now requires `runx-runtime` `cli-tool` to imply
  - Recommendation: Implement the feature edge in the connect cutover commit.
- [medium/advisory] `harden-3` acceptance_coverage - Runtime connect tests were not phased.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: `ac7b` now gates `connect_client`.
  - Recommendation: Keep the runtime and CLI connect suites paired.
- [medium/advisory] `harden-4` acceptance_strength - Inspection gates were token-only.
  - Status: fixed
  - Grounded in: spec_gap:acceptance
  - Evidence: Feature, no-panic, redaction, and polling checks are now
  - Recommendation: Prefer behavior tests where possible and negative greps only
- [low/advisory] `harden-5` design_specificity - Helper/variant expectations were loose.
  - Status: fixed
  - Grounded in: spec_gap:scope
  - Evidence: `ac5b` now requires the typed nested-runtime error variants.
  - Recommendation: Keep `BlockingHttpInsideAsyncRuntime` and
- [low/advisory] `harden-6` consistency - Supply-chain invocation/rollback notes were loose.
  - Status: fixed
  - Grounded in: spec_gap:rollback
  - Evidence: Rollback includes async deps, deny entries, and `Cargo.lock`.
  - Recommendation: Re-run cargo deny after rollback.
