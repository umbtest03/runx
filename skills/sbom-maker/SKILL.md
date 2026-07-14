---
name: sbom-maker
description: Reads a lockfile fixture, resolves dependencies locally, and emits an SBOM with license risk analysis.
---

# SBOM Maker

This skill gives security review a reproducible bill of materials from pinned dependency inputs. It operates completely locally (no network requests) and parses a provided lockfile fixture.

## Capabilities

- **Lockfile Parsing**: Extracts components (name, version, license) from the npm v2/v3 `packages` map and the classic v1 `dependencies` map.
- **SBOM Generation**: Emits a CycloneDX-style SBOM (JSON) detailing all resolved dependencies. Every component records an `evidence_location` that points at the exact key it resolved from in the supplied lockfile (`packages["node_modules/<name>"]` for the v2/v3 format, `dependencies["<name>"]` for the classic format).
- **License Summary**: Reads each component's license directly from the lockfile entry, aggregates the counts, and flags high-risk licenses (e.g. GPL-3.0) as `license_risks`. No value is guessed or mocked — a component with no license field in the lockfile is reported as `UNKNOWN`.

## Inputs

- `lockfile` (string): The raw content or JSON of the dependency lockfile.
- `lockfile_type` (string): The format identifier (e.g. `npm-shrinkwrap`, `yarn`).

## Outputs

- `sbom` (object): The structured Bill of Materials.
- `components` (array): A list of dependency components.
- `license_summary` (object): Overall summary of detected licenses.
- `license_risks` (array): Any high-risk licenses flagged.

## Harness Cases

- **supported-lockfile**: Given a valid npm v2/v3 lockfile (`packages` map), the skill seals the output with the generated SBOM, components, license summary, and license risks. Each `evidence_location` resolves to `packages["node_modules/<name>"]`.
- **supported-lockfile-classic**: Given a classic v1 lockfile (`dependencies` map), the skill seals with each `evidence_location` resolving to `dependencies["<name>"]`, proving location grounding is correct per format.
- **unsupported-lockfile**: Given an unparseable or unsupported lockfile format, the skill refuses execution (returns no SBOM).
