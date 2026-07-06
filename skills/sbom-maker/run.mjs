function parseInput() {
  const inputStr = process.env.RUNX_INPUTS_JSON;
  if (!inputStr) {
    return refuse("No input provided via RUNX_INPUTS_JSON");
  }

  let parsedInput;
  try {
    parsedInput = JSON.parse(inputStr);
  } catch (e) {
    return refuse("Invalid JSON input");
  }
  return parsedInput;
}

function parseLockfile(lockfile) {
  try {
    const parsed = typeof lockfile === 'string' ? JSON.parse(lockfile) : lockfile;
    if (!parsed || typeof parsed !== 'object') {
      return refuse("Malformed or missing lockfile content");
    }
    return parsed;
  } catch (e) {
    return refuse("Malformed or missing lockfile content");
  }
}

function extractDependencies(parsedLockfile) {
  const components = [];
  const licenses = new Map();
  const license_risks = [];

  const packages = parsedLockfile.packages;
  const dependencies = parsedLockfile.dependencies;

  const processDep = (name, version, pkgLicense) => {
    const finalLicense = typeof pkgLicense === 'string' ? pkgLicense : "UNKNOWN";
    
    components.push({
      name: name,
      version: version,
      license: finalLicense,
      evidence_location: `dependencies["${name}"]`
    });

    licenses.set(finalLicense, (licenses.get(finalLicense) || 0) + 1);

    if (finalLicense.toLowerCase().includes("gpl-3.0")) {
      license_risks.push({
        component: name,
        risk: "high",
        reason: "GPL-3.0 may trigger viral licensing requirements"
      });
    }
  };

  if (packages && typeof packages === 'object') {
    for (const [path, details] of Object.entries(packages)) {
      if (path === "") continue; 
      const name = path.split('node_modules/').pop();
      processDep(name, details.version || "unknown", details.license);
    }
  } else if (dependencies && typeof dependencies === 'object') {
    for (const [name, details] of Object.entries(dependencies)) {
      processDep(name, details.version || "unknown", details.license);
    }
  } else {
    return refuse("No dependencies found in lockfile");
  }

  return { components, licenses, license_risks };
}

function main() {
  const parsedInput = parseInput();
  const { lockfile, lockfile_type } = parsedInput;

  if (!lockfile || !lockfile_type) {
    return refuse("Missing lockfile or lockfile_type in input");
  }

  if (lockfile_type !== 'npm-shrinkwrap' && lockfile_type !== 'package-lock') {
    return refuse("Unsupported lockfile type: " + lockfile_type);
  }

  const parsedLockfile = parseLockfile(lockfile);
  const { components, licenses, license_risks } = extractDependencies(parsedLockfile);

  const sbom = {
    bomFormat: "CycloneDX",
    specVersion: "1.4",
    version: 1,
    metadata: {
      timestamp: new Date().toISOString()
    },
    components: components
  };

  const license_summary = {
    total_components: components.length,
    license_counts: Object.fromEntries(licenses)
  };

  seal({
    sbom: sbom,
    components: components,
    license_summary: license_summary,
    license_risks: license_risks
  });
}

function seal(data) {
  console.log(JSON.stringify(data, null, 2));
  process.exit(0);
}

function refuse(reason) {
  console.error(reason);
  process.exit(1);
}

main();
