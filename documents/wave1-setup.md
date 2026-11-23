# Wave 1 Setup: Initial Research

**Date:** 2026-11-23
**Inputs:** `documents/project-specs.md` (Original Request, chat context), memory file.
**Result location:** `documents/wave1-results.md`

## Objectives

| # | Objective | Agent | Tier |
|---|-----------|-------|------|
| A | macOS sensor and throttling access (Intel and Apple Silicon): APIs, privilege needs, crates, maintenance state | general-purpose | opus |
| B | Linux sensor and throttling access (CPU, GPU, NVMe, battery): sysfs interfaces, throttle signals, crates | general-purpose | sonnet |
| C | Workspace architecture: crate split, core types and traits, daemon model, daemon-to-UI data path, UI toolkit, SQLite crate, config, binary size | engineering:Software Architect | opus |
| D | Privacy and security: enforcing "no data leaves the machine", dependency policy, privileges, data at rest | engineering:Security Engineer | sonnet |

Inline (skill): feature-by-feature learning roadmap, drafted from A to D results in Wave 1 integration.

## Expected outputs

- A, B: per sensor class, the API or file path, whether root or entitlements are needed, how throttling is detected, recommended crates with version and last release date, cited URLs.
- C: recommended layout with crate names and responsibilities, key type and trait sketches (signatures only), options with trade-offs for UI toolkit, IPC, storage, and service management; decisions Omer must make.
- D: concrete controls (cargo-deny config ideas, banned crates, file permissions, privilege model), risks, decisions Omer must make.

## Success criteria

- Every claim about an API, file path, or crate cites a source (URL, docs page, or crate version).
- Unknowns are listed rather than guessed.
- Each output ends with a list of questions for Omer.
