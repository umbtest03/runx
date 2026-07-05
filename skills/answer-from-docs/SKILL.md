---
name: answer-from-docs
description: >
  Answer a question strictly from a bounded corpus. Returns grounded answers
  with citations or refuses when the corpus lacks coverage.
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
  question:
    type: string
    required: true
    description: Natural-language question to answer from the corpus.
  corpus:
    type: array
    required: true
    description: Bounded set of source documents, each with id and text.
    items:
      type: object
      properties:
        id:
          type: string
        text:
          type: string
outputs:
  answer:
    type: object
    description: Answer packet with text and citations.
    properties:
      text:
        type: string
      citations:
        type: array
        items:
          type: object
          properties:
            source_id:
              type: string
            excerpt:
              type: string
  kb_gaps:
    type: array
    description: Topics present in the question but absent from the corpus.
    items:
      type: string
  grounded:
    type: boolean
    description: True when every claim in the answer is supported by a citation.
runx:
  input_resolution:
    required:
      - question
      - corpus
  artifacts:
    named_emits:
      answer: answer
      kb_gaps: kb_gaps
      grounded: grounded
---

Answer a question using only the supplied corpus. Every sentence in the answer
must be traceable to a corpus entry via citation. When the corpus lacks
sufficient coverage the skill returns grounded: false with kb_gaps describing
what is missing.
