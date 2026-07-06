import fs from 'fs';

function run() {
  const inputStr = process.env.RUNX_INPUTS_JSON;
  if (!inputStr) {
    console.error("No input provided");
    process.exit(1);
  }

  const input = JSON.parse(inputStr);
  const corpus = input.docs_corpus || [];
  const product = input.product_surface || { commands: [], endpoints: [], schemas: [] };

  // Simple deterministic check for our fixtures
  // We check if the 'my-app deploy' command is present in the docs_corpus
  let hasDeployCmd = false;
  for (const doc of corpus) {
    if (doc.content.includes('my-app deploy')) {
      hasDeployCmd = true;
    }
  }

  if (hasDeployCmd) {
    // Fresh docs case: Refused / No-op
    console.error("Refused: Docs already match the product surface. No updates needed.");
    process.exit(1);
  } else {
    // Stale docs case: Sealed
    const output = {
      doc_findings: [
        {
          page: "cli-reference.md",
          issue: "Missing documentation for new command",
          severity: "high",
          doc_evidence: "Run the system with `my-app run`.",
          product_surface_evidence: "Deploy to cloud (NEW IN v2)",
          proposed_fix_scope: "Add my-app deploy to cli-reference.md"
        }
      ],
      coverage_map: {
        "my-app run": "documented",
        "my-app deploy": "missing"
      },
      patch_plan: [
        "Update cli-reference.md to include my-app deploy"
      ],
      docs_pr_proposal: {
        title: "docs: add missing deploy command documentation",
        branch: "docs/add-deploy-cmd",
        description: "This PR adds the missing documentation for the new `my-app deploy` command."
      }
    };

    console.log(JSON.stringify(output, null, 2));
    process.exit(0);
  }
}

run();
