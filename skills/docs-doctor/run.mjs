const docsCorpusRaw = process.env.RUNX_INPUT_DOCS_CORPUS || "[]"
const productSurfaceRaw = process.env.RUNX_INPUT_PRODUCT_SURFACE || "{}"
const userTaskMatrixRaw = process.env.RUNX_INPUT_USER_TASK_MATRIX || "[]"
const stylePolicy = process.env.RUNX_INPUT_STYLE_POLICY || ""

const docs_corpus = JSON.parse(docsCorpusRaw)
const product_surface = JSON.parse(productSurfaceRaw)
const user_task_matrix = JSON.parse(userTaskMatrixRaw)

const { commands = [], endpoints = [], schemas = [] } = product_surface

const docText = docs_corpus.map(d => d.title + " " + d.content).join(" ").toLowerCase()
const docTitles = docs_corpus.map(d => d.title.toLowerCase())

const findings = []

function checkCommandCoverage() {
  for (const cmd of commands) {
    const cmdName = cmd.name.toLowerCase()
    const isDocumented = docTitles.some(t => t.includes(cmdName)) || docText.includes(cmdName)
    if (!isDocumented) {
      findings.push({
        page: `docs/${cmd.name.replace(/\s+/g, "-")}.md`,
        issue: `Command \`${cmd.name}\` is not documented. ${cmd.description}`,
        severity: "critical",
        doc_evidence: "No documentation found for this command.",
        product_surface_evidence: `Command exists: \`${cmd.name}\` — ${cmd.description}`,
        proposed_fix_scope: `Create docs/${cmd.name.replace(/\s+/g, "-")}.md with usage, flags, examples.`
      })
      continue
    }
    const hasExample = docText.includes("example") || docText.includes("`")
    const hasFlags = docText.includes("flag") || docText.includes("--")
    if (cmdName !== "init" && (!hasExample || !hasFlags)) {
      findings.push({
        page: `docs/${cmd.name.replace(/\s+/g, "-")}.md`,
        issue: `Documentation for \`${cmd.name}\` is incomplete — missing code examples or flag details.`,
        severity: "major",
        doc_evidence: `Found mention of ${cmd.name} but no examples or flags documented.`,
        product_surface_evidence: `Command requires: ${cmd.description}`,
        proposed_fix_scope: `Add code examples and flag descriptions to docs/${cmd.name.replace(/\s+/g, "-")}.md.`
      })
    }
  }
}

function checkStyleCompliance() {
  const requiresExamples = stylePolicy.toLowerCase().includes("example")
  for (const doc of docs_corpus) {
    const content = doc.content.toLowerCase()
    if (requiresExamples && !content.includes("`") && !content.includes("example")) {
      findings.push({
        page: doc.title,
        issue: "Missing code examples — style policy requires examples.",
        severity: "minor",
        doc_evidence: `"${doc.title}" has no inline code or examples.`,
        product_surface_evidence: `Style policy: ${stylePolicy}`,
        proposed_fix_scope: `Add code examples to "${doc.title}".`
      })
    }
  }
}

checkCommandCoverage()
checkStyleCompliance()

const reportFindings = findings.length > 0 ? findings : []
const reportCoverageMap = {
  total_commands: commands.length,
  documented_commands: commands.filter(c =>
    docTitles.some(t => t.includes(c.name.toLowerCase()))
  ).length,
  total_endpoints: endpoints.length,
  documented_endpoints: endpoints.filter(e =>
    docText.includes(e.path.toLowerCase())
  ).length,
  total_schemas: schemas.length,
  documented_schemas: schemas.filter(s =>
    docText.includes(s.name.toLowerCase())
  ).length
}
const reportPatchPlan = reportFindings.map(f => ({
  target: f.page,
  action: f.severity === "critical" ? "create" : "update",
  reason: f.issue
}))
const reportProposal = reportFindings.length > 0 ? {
  title: `Docs Doctor: ${reportFindings.length} documentation ${reportFindings.length === 1 ? "issue" : "issues"} found`,
  summary: `Analysis of ${docs_corpus.length} docs against ${commands.length} commands identified ${reportFindings.filter(f => f.severity === "critical").length} critical, ${reportFindings.filter(f => f.severity === "major").length} major, and ${reportFindings.filter(f => f.severity === "minor").length} minor issues.`,
  files: reportFindings.map(f => ({ path: f.page, change: f.proposed_fix_scope }))
} : null

const output = {
  doc_findings: reportFindings,
  coverage_map: reportCoverageMap,
  patch_plan: reportPatchPlan,
  docs_pr_proposal: reportProposal
}

if (reportFindings.length === 0) {
  process.stdout.write(JSON.stringify(output) + "\n")
  process.exit(3)
}

process.stdout.write(JSON.stringify(output) + "\n")
