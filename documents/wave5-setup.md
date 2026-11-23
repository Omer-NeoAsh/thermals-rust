# Wave 5 Setup: Verification

**Date:** 2026-11-23
**Inputs:** `documents/project-specs.md` (Q&A including A1-A5 and T1-T6), `documents/wave4-results.md`
**Result location:** `documents/wave5-results.md`

## Verified inline before spawning

| Claim | Result |
|---|---|
| `PRAGMA user_version` rolls back with its transaction | Yes. The macOS `sqlite3` CLI printed 0 after a rolled-back set, and 7 after a committed one. This used the system SQLite, not the bundled 3.54.1. A store unit test will repeat the check against the bundled version. |

## Objectives

| # | Objective | Agent | Tier |
|---|---|---|---|
| V1 | Hardware and kernel facts for Omer's box: <ul><li>current Arch `linux` kernel version, and whether it is at least 6.16 (Tccd) and 6.15 (fan, `cpuinfo_avg_freq`)</li><li>Super I/O chip on the Gigabyte X870E AORUS ELITE, and whether mainline `it87` supports it or an out-of-tree driver is needed</li><li>re-derive the `gpu_metrics_v1_3` offsets and the 120-byte size from the header source</li><li>9900X `/proc/cpuinfo` model number and model name string</li></ul> | general-purpose | opus |
| V2 | Tooling claims marked UNVERIFIED in wave4-results: <ul><li>`actions/checkout` and `EmbarkStudios/cargo-deny-action` current major tags</li><li>whether `macos-latest` is arm64</li><li>strace and ptrace on `ubuntu-latest`</li><li>clippy `allow-unwrap-in-tests` / `allow-expect-in-tests`</li><li>`env!("CARGO_BIN_EXE_<name>")` for integration tests</li><li>`env_logger` default features</li><li>launchd plist `Umask` key and `~` expansion</li><li>cargo-deny `[bans] deny` entry syntax (`crate` vs `name`) for the current release</li></ul> | general-purpose | opus |

## Success criteria

- Each claim ends as VERIFIED (with source or command output), REFUTED (with the correct fact), or STILL UNVERIFIED (with the reason).
- Nothing already decided in project-specs Q&A is re-opened.
