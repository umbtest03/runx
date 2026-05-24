export interface ReceiptViewNode {
  readonly id: string;
  readonly label: string;
  readonly kind: "receipt" | "act" | "decision" | "sync" | "child-ref";
  readonly status?: string;
  readonly detail?: Readonly<Record<string, unknown>>;
}

export interface ReceiptViewEdge {
  readonly from: string;
  readonly to: string;
  readonly label?: string;
}

export interface ReceiptViewModel {
  readonly title: string;
  readonly nodes: readonly ReceiptViewNode[];
  readonly edges: readonly ReceiptViewEdge[];
}

/**
 * Build a tree view of a flat `runx.receipt.v1`: the seal at the root, then the
 * acts that ran, the governance decisions behind them, fan-out sync points, and
 * child-receipt lineage.
 */
export function buildReceiptViewModel(receipt: unknown): ReceiptViewModel {
  if (!isRecord(receipt)) {
    return { title: "Invalid receipt", nodes: [], edges: [] };
  }

  const id = stringValue(receipt.id) ?? "receipt";
  const schema = stringValue(receipt.schema) ?? "receipt";
  const seal = recordValue(receipt.seal);
  const nodes: ReceiptViewNode[] = [
    {
      id,
      label: `${schema} ${id}`,
      kind: "receipt",
      status: stringValue(seal?.disposition),
      detail: sealDetail(receipt, seal),
    },
  ];
  const edges: ReceiptViewEdge[] = [];

  for (const act of arrayValue(receipt.acts)) {
    if (!isRecord(act)) {
      continue;
    }
    const actId = stringValue(act.id) ?? `act-${nodes.length}`;
    const nodeId = `${id}:act:${actId}`;
    const closure = recordValue(act.closure);
    const provenance = recordValue(act.by);
    nodes.push({
      id: nodeId,
      label: `${stringValue(act.form) ?? "act"} ${actId}`,
      kind: "act",
      status: stringValue(closure?.disposition),
      detail: {
        form: act.form,
        summary: act.summary,
        purpose: isRecord(act.intent) ? act.intent.purpose : undefined,
        reason_code: closure?.reason_code,
        criteria: criterionStatuses(act.criterion_bindings),
        provider: provenance?.provider,
        model: provenance?.model,
        context_ref: referenceValue(act.context_ref)?.uri,
      },
    });
    edges.push({ from: id, to: nodeId, label: stringValue(act.form) ?? "act" });
  }

  for (const decision of arrayValue(receipt.decisions)) {
    if (!isRecord(decision)) {
      continue;
    }
    const decisionId = stringValue(decision.decision_id) ?? `decision-${nodes.length}`;
    const nodeId = `${id}:decision:${decisionId}`;
    const closure = recordValue(decision.closure);
    nodes.push({
      id: nodeId,
      label: `decision ${stringValue(decision.choice) ?? ""}`.trim(),
      kind: "decision",
      status: stringValue(decision.choice),
      detail: {
        choice: decision.choice,
        selected_act_id: decision.selected_act_id,
        reason_code: closure?.reason_code,
        summary: closure?.summary,
      },
    });
    edges.push({ from: id, to: nodeId, label: "decision" });
  }

  const lineage = recordValue(receipt.lineage);
  for (const syncPoint of arrayValue(lineage?.sync)) {
    if (!isRecord(syncPoint)) {
      continue;
    }
    const syncId = `${id}:sync:${stringValue(syncPoint.group_id) ?? nodes.length.toString()}`;
    nodes.push({
      id: syncId,
      label: `sync ${String(syncPoint.group_id ?? "")}`,
      kind: "sync",
      status: stringValue(syncPoint.decision),
      detail: {
        strategy: syncPoint.strategy,
        rule_fired: syncPoint.rule_fired,
        branch_count: syncPoint.branch_count,
        success_count: syncPoint.success_count,
        failure_count: syncPoint.failure_count,
        required_successes: syncPoint.required_successes,
      },
    });
    edges.push({ from: id, to: syncId, label: "sync" });
  }

  for (const [index, childRef] of arrayValue(lineage?.children).entries()) {
    const ref = referenceValue(childRef);
    if (!ref) {
      continue;
    }
    const refId = `${id}:child:${index}`;
    nodes.push({
      id: refId,
      label: `child ${ref.uri}`,
      kind: "child-ref",
      detail: ref,
    });
    edges.push({ from: id, to: refId, label: "child" });
  }

  return {
    title: receiptTitle(receipt, id),
    nodes,
    edges,
  };
}

function sealDetail(
  receipt: Readonly<Record<string, unknown>>,
  seal: Readonly<Record<string, unknown>> | undefined,
): Readonly<Record<string, unknown>> {
  const detail: Record<string, unknown> = {
    digest: receipt.digest,
    reason_code: seal?.reason_code,
    summary: seal?.summary,
  };
  const subject = recordValue(receipt.subject);
  for (const commitment of arrayValue(subject?.commitments)) {
    if (!isRecord(commitment)) {
      continue;
    }
    const scope = stringValue(commitment.scope);
    if (scope) {
      detail[`${scope}_hash`] = commitment.value;
    }
  }
  return detail;
}

function criterionStatuses(value: unknown): readonly string[] {
  const statuses: string[] = [];
  for (const binding of arrayValue(value)) {
    if (!isRecord(binding)) {
      continue;
    }
    const id = stringValue(binding.criterion_id);
    const status = stringValue(binding.status);
    if (id && status) {
      statuses.push(`${id}:${status}`);
    }
  }
  return statuses;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function recordValue(value: unknown): Readonly<Record<string, unknown>> | undefined {
  return isRecord(value) ? value : undefined;
}

function arrayValue(value: unknown): readonly unknown[] {
  return Array.isArray(value) ? value : [];
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function referenceValue(value: unknown): Readonly<{ type?: string; uri: string; label?: string }> | undefined {
  if (!isRecord(value)) {
    return undefined;
  }
  const uri = stringValue(value.uri);
  if (!uri) {
    return undefined;
  }
  return {
    type: stringValue(value.type),
    uri,
    label: stringValue(value.label),
  };
}

function receiptTitle(receipt: Readonly<Record<string, unknown>>, fallback: string): string {
  const subject = recordValue(receipt.subject);
  const subjectRef = recordValue(subject?.ref);
  return stringValue(subjectRef?.label)
    ?? stringValue(subjectRef?.uri)
    ?? fallback;
}
