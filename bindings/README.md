# Upstream Bindings

Bindings connect a verified upstream `SKILL.md` to a runx execution profile.

The upstream repository remains the source of truth for the portable skill
document. This directory stores runx-owned binding data:

- `binding.json`: upstream repo/path/commit provenance, trust tier,
  registry owner, publication state, and proof pointers.
- `X.yaml`: the execution profile artifact containing runner metadata,
  harness cases, policy, scopes, and receipt expectations.

Use `bindings/` only when runx does **not** own the upstream `SKILL.md`.
First-party runx packages belong in `skills/<name>/`. Product-specific operator
wrappers belong in the product repo that owns their policy.

Do not add candidate or placeholder directories here. A binding is eligible only
after the upstream repository has a merged, pinned `SKILL.md` that starts with
YAML frontmatter and a `name` matching `bindings/<owner>/<skill>`.

Publishing materializes a pinned registry package into `dist/` from the
upstream `SKILL.md` plus the local binding artifact. The generated package is
an immutable registry artifact, not the source document.

Example:

```bash
node scripts/materialize-upstream-skill-binding.mjs \
  bindings/nilstate/icey-server-operator/binding.json \
  --output-dir dist/upstream-bindings/nilstate/icey-server-operator
```

Validate the checked-in binding catalog:

```bash
pnpm bindings:check
```

## Candidate Queue

These are good candidates once their upstream repositories own the source
`SKILL.md`; they must not appear as binding directories before that happens.

- `nilstate/scafld-operator` — scafld governs agent work. Add this after
  `nilstate/scafld` owns a merged `SKILL.md` at a pinned commit.
- `sourcey/<project-operator>` — only if Sourcey moves the portable
  instructions into a Sourcey-owned upstream repo. If runx owns the skill,
  keep it under `skills/sourcey`.
- Real OSS project operator skills such as `contentauth/c2pa-rs`,
  `napi-rs`, `hono`, or `drizzle` — only after the project accepts or hosts
  its own `SKILL.md`.
