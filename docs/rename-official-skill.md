# Runbook: rename an official skill

Renaming an official `runx/*` skill is a governed operator procedure, not an
ad-hoc `git mv`. A skill's identity is its directory name, wired into generated
artifacts, the hosted registry row, and the runx.ai site. Done wrong, a rename
resets the registry row's `created_at`, drops its version history, or 404s site
links. Done with the machinery below, it preserves everything and the only
manual thinking is the name itself.

This runbook pairs with [reference.md](reference.md) (crates/registry) and the
cloud deploy procedure. The registry clean-rename primitive already exists; you
do not rebuild it.

## Naming principle

Rename to make an agent instinctively reach for the right skill:

- Name the **job**, not the mechanism. Drop `-auditor`, `-generator`,
  `-analyst`, `-pipeline` suffixes.
- One distinct **verb per pipeline layer** so sibling skills never blur. The
  security trio is the model: `cve-audit` (detect, deterministic) ->
  `vuln-triage` (assess, agent) -> `vuln-disclosure` (publish, governed).
- Keep provider rails and integrations descriptive (`stripe-pay`, `web-fetch`).
- Retrieval beats cleverness: the searchable term wins, and keywords the name
  drops go in the `description`.
- Packet ids (`runx.*.v1`) are versioned wire contracts. They are **never**
  renamed with the skill.

## How the registry preserves a rename

The hosted API bootstraps official skills from `oss/skills` on connect boot and
reconciles the `runx/*` catalog to match. A rename map, `RUNX_OFFICIAL_REGISTRY_RENAMES`
(a `old:new,old:new` string in `cloud/deploy/docker-compose.connect.yml`), tells
the bootstrap to **move** each retired row wholesale to its new id before it
reconciles: every version, its `created_at`, digests, attestations, and metadata
carry across untouched and re-sign for the new id; only `skill_id` and `name`
change. It is idempotent (a no-op once the old id is gone) and chains multi-hop
in order (`a:b,b:c` renames `a` all the way to `c`). A skill with no successor in
`oss/skills` is pruned, which is the real delete path. So renames preserve and
only true retirements delete.

The public read model surfaces a row's **earliest** version `created_at` as its
`created_at` (birth) and the resolved version's time as `updated_at`, so the
birth date survives even though the renamed content publishes as the newest
version.

## Procedure

**1. oss repo**
- `git mv skills/<old> skills/<new>`.
- Token-replace `<old>` -> `<new>` across `skills/ docs/ tests/ README.md`.
  Do not touch `dist/`, `official.lock.json`, or packet ids (they use
  underscores/dots, so the hyphenated skill name will not match them anyway).
  Watch for external URLs that merely contain the token.
- Sharpen the renamed skill's `SKILL.md` and bump the version of any *other*
  skill whose `SKILL.md` cross-reference changed.
- Regenerate: `pnpm exec tsx scripts/generate-packet-schemas.ts` and
  `node scripts/generate-official-lock.mjs`.
- Gate: `pnpm verify:fast`, `runx harness skills/<new>`, and the catalog sweep.
- Commit, push, bump the workspace submodule pointer.

**2. cloud repo**
- Token-replace `<old>` -> `<new>` in the site: `apps/web/src/lib/runx-api.ts`,
  `apps/web/src/pages/x/index.astro`, `apps/web/src/components/home/CatalogSection.astro`,
  `apps/web/public/vendor/runx-run-globe.js`, `packages/api/src/public-featured-skills.ts`.
- Append `<old>:<new>` to `RUNX_OFFICIAL_REGISTRY_RENAMES` in
  `deploy/docker-compose.connect.yml` (keep prior entries; they are no-ops).
- `pnpm astro check` (web) and commit, push, bump the submodule.

**3. deploy**
- Back up the registry store first:
  `ssh root@<droplet> 'cd /opt/runx-live/cloud/deploy && cp -a data/runx-registry data/runx-registry.bak.$(date +%s)'`.
- Sync the renamed source: `rsync -az --delete oss/skills/ root@<droplet>:/opt/runx-live/oss/skills/`.
- **Build the api dist locally: `pnpm build:server`.** The connect image expects
  `packages/api/dist` pre-built and synced; `deploy:site` with
  `RUNX_DEPLOY_INCLUDE_DOCS=0` does NOT build it. Skip this and the image ships
  stale code, the rename never runs, and the row gets delete+recreated with a
  reset `created_at`.
- `RUNX_DEPLOY_INCLUDE_DOCS=0 pnpm deploy:site` (syncs cloud + the fresh dist +
  the compose map).
- Rebuild and recreate connect (build-before-recreate = no downtime on failure):
  `docker compose -p runx-connect-live -f docker-compose.connect.yml build connect`
  then `... up -d --no-deps --force-recreate --no-build connect`.
- Edge-deploy the site so runx.ai reflects the new catalog.

Do NOT use `pnpm deploy:skills` when the api code changed: it recreates connect
without a rebuild, so it runs the old image.

**4. verify**
- `curl -s https://api.runx.ai/v1/skills/runx/<new>` shows the preserved
  `created_at`; the old id returns 404; the total count did not grow (no doubling).
- `curl -sI https://runx.ai/x/runx/<new>` resolves (302 -> `/x/@runx/<new>` -> 200).
