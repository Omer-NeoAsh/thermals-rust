# thermals-rust: Project Specification

**Status:** Final (approved by Omer 2026-11-23); build in progress
**Type:** App proposal (new app, built feature by feature as a guided Rust learning project)
**Last updated:** 2026-11-23

## Overview

A thermal monitoring application written in Rust. It reads temperature (and throttling) data from the local machine and keeps every piece of data on that machine.

## Original Request

> I want to build a thermal monitoring app using rust. The application data is private to the user's computer with no shared data leaving their computer.
>
> I will separate the app into three separate crates (or more as we discuss the plan further). Right now I want it to work for mac and linux and we will add windows support later.
>
> Let's use this folder as the root for the rust cargo. I also do not want you to build everything but help walk me through building the app. This would be a good use of learning rust further. You will still write the code in this chat, but not to a file.
>
> I want to build feature by feature to ensure we are building upon the app. We should follow proper design principles and industry standards reducing any code duplication and overuse of comments.

### Context from the earlier claude.ai chat (licensing discussion)

- The tool reads the OS's hardware thermal readouts and should run on a wide variety of devices. Omer uses a Mac and a daily Linux machine.
- Main goal: know when any hardware is thermally throttled. That likely needs knowledge of each hardware type's throttling limit.
- Report-only. Not a web server.
- A light background service collects data (simpler than HWiNFO-style tools). A SQLite database keeps about 30 days, with configurable retention. A frontend shows temperature graphs.
- Privacy is a core requirement: data never leaves the system.
- Starts small, grows many features later. Rust chosen for memory safety and a small binary.
- License: GPL-3.0-or-later (LICENSE file is already GPLv3). Declare `license = "GPL-3.0-or-later"` in Cargo.toml. Use cargo-deny to enforce a dependency license allow-list.
- The sensor library could later be relicensed permissively (MIT OR Apache-2.0, or MPL-2.0) while the apps stay GPL. Not decided.
- Crate layout sketched in the chat (a suggestion, not a decision): `crates/thermal-core` (sensor library), `crates/thermald` (background collector writing SQLite), `crates/thermal-ui` (graphs). Workspace with `resolver = "3"`, `edition = "2024"`, shared `[workspace.package]` license.
- Mac App Store is off the table (GPL and sandbox). Homebrew or direct distribution is fine.

### Constraints stated so far

- Language: Rust. Cargo workspace root: this repository.
- Privacy: no data leaves the user's computer.
- Structure: at least three crates.
- Platforms: macOS and Linux now; Windows later.
- Process: guided build. Claude writes code in chat; Omer types it into files.
- Delivery: feature by feature, each building on the last.
- Quality: established design principles, minimal duplication, sparse comments.

## Research Summary

Full findings, sources and the starter `deny.toml`: `documents/wave1-results.md`.

- **Throttling comes from the OS, not temperature tables.** No published per-chip throttle temperatures exist for Apple Silicon; an M3 read 115 °C at nominal pressure. Sources of truth:
  - macOS (Intel and Apple Silicon): the public thermal pressure level (`com.apple.system.thermalpressurelevel`, Nominal/Moderate/Heavy/Trapping/Sleeping) and `NSProcessInfo.thermalState`. No root.
  - Linux Intel CPU: cumulative `thermal_throttle/*_throttle_count` counters in sysfs. No root.
  - Linux AMD CPU: no reliable signal found. NVIDIA via NVML; AMD GPU via `gpu_metrics`.
- **macOS temperatures need private APIs.** IOHID (what sysinfo uses) and SMC both work as a normal user but are undocumented and vary by chip and OS release. powermetrics needs root.
- **Linux temperatures** come from hwmon sysfs (world-readable). Stable sensor IDs need the chip name plus device path, not the `hwmonN` index.
- **Write own backends.** sysinfo lacks fans, throttle signals and stable Linux IDs.
- **Proposed layout: four crates** (Wave 1 names, since renamed; see F1). The store crate keeps the SQL in one place for both binaries. `thermald` is taken by Intel's Linux daemon.
- **Daemon:** user-level LaunchAgent / systemd user unit, plain `std` loop, no tokio, no root.
- **UI reads SQLite directly** (WAL, read-only, polling). No IPC.
- **UI:** CLI report first, then eframe + egui_plot (3.2 MB measured). ratatui terminal UI is 0.5 MB.
- **Storage:** rusqlite (bundled). 48 h raw + 30 days of 1-minute rollups ≈ 51 MB for 20 sensors; raw-only for 30 days ≈ 158 MB.
- **Privacy:** cargo-deny bans, a runtime no-socket test, and `RestrictAddressFamilies=AF_UNIX` on Linux. macOS has no cheap OS-level network block.
- **A 13-step build order** with Rust concepts per step (see wave1-results.md section C).

## Questions & Answers

Asked 2026-11-23 (Wave 2). **B** = blocking.

| # | Question | Answer |
|---|---|---|
| 1 | B. Crate names: `thermals-core`, `thermals-store`, `thermalsd`, `thermals`? | The service is named `neem-thermals` (company: neemtech). `thermalsd` is too close to `thermald`. Names should be distinct but clear. Scope of the prefix: see follow-up F1. |
| 2 | B. Four crates (adds `thermals-store`) instead of three? | Yes. |
| 3 | B. Build the Linux backend or the macOS backend first? | Linux first. |
| 4 | B. Are private macOS APIs (IOHID, SMC) acceptable for per-sensor temperatures? | Yes. Seeing temperatures on the Mac is one of the main things missing today. |
| 5 | What hardware is the Linux machine (CPU vendor, GPU vendor)? Is Intel Mac support in scope? | AMD Ryzen 9 9900X and AMD Radeon RX 9070 XT (Sapphire Nitro+). Intel Mac: not answered; default is best effort, untested. |
| 6 | B. What counts as "throttled"? OS signals only, or also inferred signals? What shows on AMD CPUs? | OS-reported throttling first. When the OS gives no signal, use a fallback, which may need per-chip details. Omer's understanding: Intel and AMD publish when they throttle; M-chips do not. (See Wave 3 notes: research partly differs.) |
| 7 | B. Is user-level only (no root, ever) the rule? | Stay in normal userspace and avoid root wherever possible, since root apps are not widely trusted. Not "never": some OS may require it. |
| 8 | Is a report enough, or do you want a desktop notification when throttling starts? | Backlog. For now, graph and temperature values are enough. |
| 9 | Per-sensor temperature warning thresholds in addition to throttling? | Yes, good idea. Backlog; not needed for the first working app. |
| 10 | GUI (egui) as the target, or is a terminal UI (ratatui) enough? | Start with a terminal UI. A GUI with graphs is still liked; alternatively export data in a format that opens as a graph, or let multiple client types read the data. Focus: terminal UI. |
| 11 | Sample every 5 s, keep raw 48 h, keep 1-minute summaries 30 days? | Yes. |
| 12 | Privacy bar: CI bans + runtime test + Linux systemd restriction, with no OS-level block on macOS? | Yes. |
| 13 | Distribution: source / `cargo install` / Homebrew only, or signed binaries too? | Source only for now. A paid Apple Developer account is possible later. |
| 14 | B. How should each feature be delivered, and may Claude run `cargo build` / `cargo test` on code you type? | Agreed: goal, then code per file with explanation, Omer types it, Claude reviews. Claude may run `cargo build`, `cargo test`, `cargo clippy`. |
| 15 | Toolchain folder `~/.cargo` (crate sources): add to readable directories, no, or lives elsewhere? | Add. Done 2026-11-23 (`permissions.additionalDirectories`). |

**Follow-ups (Wave 3, 2026-11-23):**

| # | Question | Answer |
|---|---|---|
| F1 | How far does the `neem-` prefix go? | Everything: libraries `neem-thermals-core`, `neem-thermals-store`; service `neem-thermals`; terminal UI `neem-thermals-tui`. Config/data folder `neem-thermals`; launchd label `com.neemtech.neem-thermals`. Superseded by A5: identifier `ca.neemtech.neem-thermals`. |
| F2 | How is inferred throttling shown when the OS gives no signal? | Labelled separately: "Throttling (OS reported)" vs "Likely throttling (inferred)". |
| F3 | Which export format goes in the backlog? | None for now. Other clients can read the SQLite file directly. |

**Wave 4 questions (architecture review, 2026-11-23):**

| # | Question | Answer |
|---|---|---|
| A1 | Uninstall keeps data by default, `--purge` deletes it? | Yes. |
| A2 | Should `service install` enable `loginctl enable-linger`? | No. Install neither enables nor offers it. Linger keeps the same unprivileged user, but collecting while logged out costs battery for no benefit: throttling happens under load while someone is logged in. |
| A3 | Config file to the backlog; interval and retention as constants for now? | Yes. |
| A4 | macOS: any pressure level above Nominal counts as "Throttling (OS reported)"? | Yes, shown with the level name. Revisit after a load test. |
| A5 | Data folder identifier (raised by Omer) | Use his domain `neemtech.ca`: `ProjectDirs::from("ca", "neemtech", "neem-thermals")`. macOS: `~/Library/Application Support/ca.neemtech.neem-thermals`. Linux ignores qualifier and organisation (`directories-6.0.0/src/lin.rs:92-93`), so it stays `~/.local/share/neem-thermals` and `~/.config/neem-thermals`. The launchd label becomes `ca.neemtech.neem-thermals` to match, replacing F1's `com.neemtech.neem-thermals`. |

**Wave 4 questions (testing and hardware, 2026-11-23):**

| # | Question | Answer |
|---|---|---|
| T1 | GitHub repo public or private? | Public. CI runs on both Linux and macOS. |
| T2 | Collector `--samples N` flag and `fake` feature for testing? | Yes. |
| T3 | Collector on a machine with no sensors? | Exit 0 with a warning. |
| T4 | Commit the AMD fixture with dangling `device` symlinks (PCI addresses)? | Yes. |
| T5 | Weekly scheduled `cargo deny` run? | No. |
| T6 | Linux kernel version and motherboard? | Gigabyte X870E AORUS ELITE. Arch Linux, so target the current Arch `linux` package kernel. |

**Where answers and research differ (flagged in Wave 3):**

- Answer 6 says M-chips do not publish throttling. Research found macOS does publish a public thermal pressure level on Apple Silicon (no root). Not yet confirmed that it rises under real throttling.
- Answer 6 says AMD normally publishes throttling. Research found no AMD CPU throttle counter on Linux (Intel has one). AMD publishes a temperature limit in its specs, not a runtime signal. Wave 4 researches Zen 5 and RDNA 4 specifically.

## Gap Analysis

Full findings, sources, type sketches, schema v1, CI layout: `documents/wave4-results.md`.

- **Ryzen 9 9900X: no OS throttle signal a normal user can read.** CPU throttling on Omer's Linux box is inferred only. The heuristic uses Tctl ≥ 94 °C for 3 samples, load ≥ 50 % and busy-core clocks below 98 % of base (4.4 GHz). Tctl at 95 °C with clocks above base is a separate state, "At thermal limit (boost regulated)". Tjmax 95 °C is confirmed from AMD's page.
- **The 9070 XT has an OS signal.** `gpu_metrics` (`v1_3`, readable by everyone) carries `throttle_status`. Temperature and VR bits count as thermal throttling; PPT, TDC and FIT bits count as power-limited, not throttling. EPERM means the GPU is asleep.
- **Wave 1 corrections:**
  - There is no Tdie on Zen 5; use Tctl (`temp1`).
  - Per-CCD temperatures need kernel 6.16, and `cpuinfo_avg_freq` needs 6.15.
- **Throttle state is stored as samples** in the `reading` table, and the `throttle_event` table is dropped. Sensors carry `Kind` (temperature, throttle with OS or inferred source, pressure) and a device scope.
- **Collector rules:**
  - monotonic deadline with wall-clock `ts`
  - `INSERT OR IGNORE`
  - rollup recomputes a sliding window
  - rescan every 60 s
  - skip-and-continue on tick errors
  - single-instance lock
  - exit 78 on startup errors, with restart left to systemd or launchd
  - migrations by the collector only, with a backup first
- **TUI:** reads the DB read-only and handles four open states (Ready, Missing, NotInitialised, SchemaMismatch) plus a stale-collector banner. It never touches the lock. Read-only WAL open was verified by probes.
- **Testing:**
  - The collector gets a lib/bin split, a `--samples N` flag and a `fake` feature, so the runtime no-network test (strace on Linux, `sandbox-exec` on macOS, plus a negative control) is deterministic.
  - Linux parsers compile and run fixture tests on every OS.
- **Accessibility:** every state has a text label (never colour alone), `NO_COLOR` is respected, and numeric values are shown next to charts.
- **Board sensors (corrected in Wave 5):** the X870E AORUS ELITE uses ITE IT8696E + IT87952E. Mainline it87 lacks the IT8696E, so board fans need the out-of-tree `frankcrawford/it87` driver and a `modules-load.d` entry. That is a root step the user performs, never the app, and board sensors are optional.
- **The build order grows to 14 steps**, with Linux first, then the terminal UI, then the service, then macOS.

## Open Items

Verification details: `documents/wave5-results.md`. Nothing here blocks starting the build. Each item is settled by doing the step it belongs to.

| Item | Settled at |
|---|---|
| Inferred-throttle thresholds for the 9900X and 9070 XT (margins, durations, clock floor) | Load runs on the Linux box, build step 5 |
| `ThrottleState` shape (None, PowerLimited, AtLimit, Throttling) | Build step 4 |
| Whether macOS pressure rises under real throttling (A4 revisit) | Load test on the M3, step 11 |
| macOS duplicate sensor names | Step 12 |
| ptrace (strace) and `sandbox-exec` on hosted CI runners | First CI run |
| Mainline it87 behaviour on this board (if Omer doesn't use the out-of-tree driver) | When board sensors are wanted |
| CPU and memory budget on the M3 (under 0.1 % CPU, under 15 MB RSS) | Step 13 measurement |
| Forward clock jump during prune | Accepted (low) |

## Decisions Log

| Date | Decision | Reason |
|------|----------|--------|
| 2026-11-23 | Planning docs live in `documents/` | Chosen by Omer |
| 2026-11-23 | Four crates: `neem-thermals-core`, `neem-thermals-store`, `neem-thermals` (collector), `neem-thermals-tui` | Q1, Q2, F1. The store crate keeps the SQL in one place. |
| 2026-11-23 | Identifier `ca.neemtech.neem-thermals` (macOS folder, launchd label); `neem-thermals` folders on Linux | A5, Omer's domain |
| 2026-11-23 | Linux first, then the terminal UI, then the service, then macOS | Q3, Q10; roadmap below |
| 2026-11-23 | Own sensor backends, no sysinfo: hwmon sysfs on Linux, IOHID and the pressure notify key on macOS (private APIs accepted) | Q4, Wave 1 |
| 2026-11-23 | Throttle: OS signal first; inferred fallback labelled "Likely throttling (inferred)"; power limits shown as "Power-limited", not throttling | Q6, F2, Wave 4 G1 |
| 2026-11-23 | No root; user-level systemd unit and LaunchAgent; linger never enabled or offered | Q7, A2 |
| 2026-11-23 | Plain `std` loop, no async runtime, no clap; `thiserror` in libraries, `anyhow` in binaries | Wave 1 C, Wave 4 G2 |
| 2026-11-23 | SQLite (rusqlite, bundled), WAL. 5 s samples, 48 h raw, 1-minute rollups for 30 days. Throttle stored as samples in `reading`. | Q11, Wave 4 G2 |
| 2026-11-23 | Interval and retention are constants; config file in the backlog | A3 |
| 2026-11-23 | TUI reads the DB read-only, with no IPC; it never touches the collector lock | Wave 1 C, Wave 4 G2 |
| 2026-11-23 | Privacy: cargo-deny bans, clippy `disallowed-types` for `std::net`, runtime no-network test with a negative control, systemd `RestrictAddressFamilies=AF_UNIX`, data dir 0700 | Q12, Wave 4 G3 |
| 2026-11-23 | Source-only distribution; public GitHub repo with CI on Linux and macOS | Q13, T1 |
| 2026-11-23 | Collector gets `--samples N`, a `fake` feature and `--db PATH`; exits 0 with a warning when there are no sensors | T2, T3, G2 |
| 2026-11-23 | Uninstall keeps data; `--purge` deletes it | A1 |
| 2026-11-23 | Notifications, per-sensor thresholds, GUI, export: backlog | Q8, Q9, Q10, F3 |

## Next Steps

### Implementation plan

A guided build. Claude gives the goal and the code for each file in chat; Omer types it; Claude reviews with `cargo build`, `cargo test` and `cargo clippy`. Crates are created at the step that first needs them. The type sketches, schema and test layout are in `wave4-results.md`.

| # | Step | Runnable result |
|---|---|---|
| 1 | Workspace, lints, core types (`Celsius`, `SensorId`, `Value`, `Sample`) | `neem-thermals` prints a hard-coded sample; core unit tests pass |
| 2 | `Backend` trait, `FakeBackend`, deadline loop, Ctrl-C/SIGTERM, `--samples N`; CI (fmt, clippy, test, no-network check) | Fake samples every 5 s, clean exit; CI green |
| 3 | Linux hwmon discovery and read, stable ids, junk filter; fixtures | Real temperatures on Linux; fixture tests pass on the Mac |
| 4 | Linux OS throttle signals: amdgpu `gpu_metrics`, Intel counters; `ThrottleState` | Throttle state per device |
| 5 | Inferred fallback: limits table, 9900X heuristic, hysteresis | "Likely throttling (inferred)" and "At thermal limit" |
| 6 | Store `Writer`: migrations, register, record; single-instance lock; `--db` | Collector fills a SQLite file |
| 7 | Rollup, prune, rescan, tick error policy, logging | Runs for hours, logs only state changes |
| 8 | TUI v1: `Reader`, latest values, throttle labels, open states, stale banner | Live table next to a running collector |
| 9 | TUI v2: charts, time ranges, gap splitting | Graphs for 10 min to 30 d |
| 10 | `service install/uninstall`: systemd user unit | Starts at login; logs in the journal |
| 11 | macOS pressure signal | "Throttling (OS reported)" on the M3 |
| 12 | macOS IOHID temperatures | M3 sensors in the TUI |
| 13 | LaunchAgent; measure the budget on the M3 | CPU and RSS figures recorded |
| 14 | Privacy hardening and release: cargo-deny, unit sandbox, release profile | `cargo deny check` and `systemd-analyze --user security` pass |

**Backlog:** GUI (eframe + egui_plot), notifications, per-sensor thresholds, config file, export, NVIDIA/NVML, Intel Mac testing, SMC temperatures, slower sampling on battery, system-wide daemon, Homebrew/signed binaries, Windows.
