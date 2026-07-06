---
name: sbom-maker
description: Reads a lockfile fixture, resolves dependencies locally, and emits an SBOM with license risk analysis.
---

# SBOM Maker

This skill gives security review a reproducible bill of materials from pinned dependency inputs. It operates completely locally (no network requests) and parses a provided lockfile fixture.

## Capabilities

- **Lockfile Parsing**: Extracts components (name, version) from `npm-shrinkwrap` and similar formats.
- **SBOM Generation**: Emits an SBOM (JSON) detailing all resolved dependencies.
- **License Summary**: Emits a mock/heuristic-based license summary and license-risk analysis for the dependencies found.

## Inputs

- `lockfile` (string): The raw content or JSON of the dependency lockfile.
- `lockfile_type` (string): The format identifier (e.g. `npm-shrinkwrap`, `yarn`).

## Outputs

- `sbom` (object): The structured Bill of Materials.
- `components` (array): A list of dependency components.
- `license_summary` (object): Overall summary of detected licenses.
- `license_risks` (array): Any high-risk licenses flagged.

## Harness Cases

- **supported-lockfile**: When given a valid `npm-shrinkwrap` or supported lockfile, the skill will seal the output with the generated SBOM and components.
- **unsupported-lockfile**: When given an unparseable or unsupported lockfile format, the skill refuses execution (returns no SBOM).
