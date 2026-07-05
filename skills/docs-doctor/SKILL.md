---
name: docs-doctor
description: >
  Find stale product documentation by comparing docs against the actual product
  surface. Emits grounded findings with severity, evidence, and fix proposals.
  Never rewrites docs without a proposal.
source:
  type: cli-tool
  command: node
  args:
    - run.mjs
  timeout_seconds: 15
  sandbox:
    profile: readonly
    cwd_policy: skill-directory
inputs:
  docs_corpus:
    type: array
    required: true
    description: Current documentation entries, each with id, title, and content.
    items:
      type: object
      properties:
        id:
          type: string
        title:
          type: string
        content:
          type: string
  product_surface:
    type: object
    required: true
    description: Actual product surface with commands, endpoints, and schemas.
    properties:
      commands:
        type: array
        items:
          type: object
          properties:
            name:
              type: string
            description:
              type: string
      endpoints:
        type: array
        items:
          type: object
          properties:
            path:
              type: string
            method:
              type: string
            description:
              type: string
      schemas:
        type: array
        items:
          type: object
          properties:
            name:
              type: string
            fields:
              type: array
              items:
                type: string
  user_task_matrix:
    type: array
    required: true
    description: Common user tasks, each with task name and steps.
    items:
      type: object
      properties:
        task:
          type: string
        steps:
          type: array
          items:
            type: string
  style_policy:
    type: string
    required: true
    description: Documentation style guidelines.
outputs:
  doc_findings:
    type: array
    description: List of documentation issues found.
    items:
      type: object
      properties:
        page:
          type: string
        issue:
          type: string
        severity:
          type: string
        doc_evidence:
          type: string
        product_surface_evidence:
          type: string
        proposed_fix_scope:
          type: string
  coverage_map:
    type: object
    description: Coverage map showing which surface areas have documentation.
    properties:
      total_commands:
        type: number
      documented_commands:
        type: number
      total_endpoints:
        type: number
      documented_endpoints:
        type: number
      total_schemas:
        type: number
      documented_schemas:
        type: number
  patch_plan:
    type: array
    description: Proposed doc patch plan entries.
    items:
      type: object
      properties:
        target:
          type: string
        action:
          type: string
        reason:
          type: string
  docs_pr_proposal:
    type: object
    description: Gated proposal consumed by issue-to-pr. Never edits repo directly.
    properties:
      title:
        type: string
      summary:
        type: string
      files:
        type: array
        items:
          type: object
          properties:
            path:
              type: string
            change:
              type: string
runx:
  input_resolution:
    required:
      - docs_corpus
      - product_surface
      - user_task_matrix
      - style_policy
  artifacts:
    named_emits:
      doc_findings: doc_findings
      coverage_map: coverage_map
      patch_plan: patch_plan
      docs_pr_proposal: docs_pr_proposal
---

Docs Doctor compares a documentation corpus against the actual product surface.
It finds stale, missing, or incorrect documentation and emits structured findings
with severity (critical, major, minor), evidence from both docs and product
surface, and patch proposals. When no issues are found, it returns empty findings
with a coverage map showing full coverage.
