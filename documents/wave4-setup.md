# Wave 4 Setup: Gap Analysis

**Date:** 2026-11-23
**Inputs:** `documents/project-specs.md` (Research Summary, Q&A incl. follow-ups F1-F3), `documents/wave1-results.md`, memory file.
**Result location:** `documents/wave4-results.md`

## Objectives

| # | Objective | Gap categories | Agent | Tier |
|---|-----------|---------------|-------|------|
| G1 | Omer's Linux hardware (Ryzen 9 9900X, Radeon RX 9070 XT): sensors, throttle signals, and an inferred-throttle fallback with per-chip data, all without root | Functional, integration | general-purpose | opus |
| G2 | Architecture update for the decisions taken (names, TUI, throttle source model, per-chip data), plus edge cases and non-functional needs of a long-running collector | Functional, non-functional, edge cases, integration, operational | engineering:Software Architect | opus |
| G3 | Test and CI plan: fixtures without hardware, capturing fixtures from real machines, CI matrix, cargo-deny, the runtime no-network test | Test, operational | engineering:DevOps Automator | sonnet |

Inline (skill): documentation gaps, accessibility of the TUI, security gaps not already covered in Wave 1 D.

## Expected outputs

Each agent returns a gap table (gap, why it matters, proposed resolution or OPEN) and a short summary of its 2-3 most important gaps. G1 also returns concrete sysfs paths and data per component; G2 returns updated type sketches and roadmap; G3 returns CI and test layout.

## Success criteria

- Every API, path or version claim cites a source or is marked unverified.
- Every gap has a resolution or is flagged OPEN with the reason.
- Proposals respect decisions already taken (see project-specs Q&A); no re-litigating.
