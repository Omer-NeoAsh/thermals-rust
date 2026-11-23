# Wave 1 Results: Initial Research

**Date:** 2026-11-23
**Status:** COMPLETE (4 of 4 objectives met)
**Setup:** `documents/wave1-setup.md`

## Agent performance

| Objective | Agent @ tier | Result | Notes |
|---|---|---|---|
| A. macOS sensors and throttling | general-purpose @ opus | Usable | Ran FFI probes on Omer's Mac (M3 MacBook Air, fanless, macOS 27.2.1). No Intel Mac available. |
| B. Linux sensors and throttling | general-purpose @ sonnet | Usable | Docs and source only. No Linux machine to test. |
| C. Workspace architecture | engineering:Software Architect @ opus | Usable | Built size probes on this Mac. Sizes are measured. |
| D. Privacy and security | engineering:Security Engineer @ sonnet | Usable | Several items marked unverified by the agent; flagged below. |

Probe code from A and C is in the session scratchpad. It is throwaway and not part of the project.

---

## A. macOS

**Temperatures (all worked as a normal user on the M3, except powermetrics):**

| Source | API status | Apple Silicon | Intel | Notes |
|---|---|---|---|---|
| IOHID event system (`IOHIDEventSystemClientCreate`, usage page 0xff00 / usage 5) | Private | Yes. 39 sensors on the M3: `PMU tdie*`, `NAND CH0 temp`, `gas gauge battery`, ... | No | What sysinfo uses on Apple Silicon. Names differ by chip. Some return junk (-1.5, NaN): filter. |
| SMC via IOKit user client (`AppleSMCKeysEndpoint` on Apple Silicon, `AppleSMC` on Intel) | IOKit public, protocol and keys undocumented | 155 readable `T*` keys on the M3 | Yes (`sp78` values) | Key meanings differ per chip; published key tables did not match this M3. `Tp*` read 96-115 °C while HID die sensors read 50-73 °C. |
| powermetrics | Apple CLI | Root only | Root only | Skip. |
| `AppleSmartBattery` IORegistry `Temperature` | Public | Missing on macOS 27 | n/a | Use HID or SMC battery sensors instead. |

Fans: SMC `F%Ac` / `F%Mx` / `FNum`, no root. The M3 Air has none.

**Throttling signals, most reliable first:**

1. **Thermal pressure via notify(3)**, key `com.apple.system.thermalpressurelevel`. Public, no root, Intel and Apple Silicon. Levels: Nominal 0, Moderate 1, Heavy 2, Trapping 3, Sleeping 4. Can push changes (`notify_register_dispatch`) or be polled.
2. **`NSProcessInfo.thermalState`** (nominal, fair, serious, critical). Public, via `objc2-foundation`. Mapping to (1) is undocumented.
3. **IOReport residency channels** (CLTM GPU states, SOCHOT, CPMS power reduction, NVMe throttle time). Private, no root, still updating on macOS 27.2.1. Meanings inferred from names only. Must exclude Low Power Mode capping (`isLowPowerModeEnabled`).
4. `IOPMCopyCPUPowerStatus` / `pmset -g therm`: not published on Apple Silicon; likely Intel-only.

**No Apple-published throttle temperatures exist for M-series chips.** The SMC read 115 °C at nominal pressure. Treat the OS signal as truth and temperatures as context.

**Crates:** sysinfo 0.40.4 (MIT; IOHID names, no fans, no throttle), macmon 0.12.0 (MIT; best reference code, Apple Silicon only, opinionated), objc2 0.6.5 / objc2-foundation 0.3.2 / objc2-io-kit 0.3.2 (active, permissive). Avoid macsmc, IOKit-sys (dead). `smc` 0.2.4 is stale (2023). No crate exists for IOReport or the pressure notify key; both are small hand-written FFI.

**Risks:** private APIs change between macOS releases (macOS 27 already locked IOReport energy counters behind an entitlement). Sensor names and SMC keys vary per chip.

## B. Linux

**Temperatures:** hwmon (`/sys/class/hwmon/hwmon*/temp*_input`, millidegrees C, world-readable per kernel docs; `_label`, `_crit`, `_max`, `_fault`). Drivers: coretemp (Intel), k10temp (AMD: `temp1` Tctl is an arbitrary scale, prefer `temp2` Tdie, `temp3+` per-CCD), amdgpu, nvme, battery. Thermal zones (`/sys/class/thermal/thermal_zone*/temp`, trip points) as a fallback. Fans: hwmon `fan*_input`.

**Throttling signals:**

| Vendor | Signal | Reliability |
|---|---|---|
| Intel CPU | `/sys/devices/system/cpu/cpuN/thermal_throttle/{core,package}_throttle_count` and `_total_time_ms`. Cumulative since boot; poll the delta. Package values repeat per CPU, so deduplicate. | Best. Kernel docs recommend it over MSRs. |
| AMD CPU | No counter found. Weak fallbacks: clock drop, Tdie near `temp1_crit`. | Weak |
| ARM SoC | Cooling device `cur_state > 0` plus thermal zone trip points | Medium |
| AMD GPU | `gpu_metrics` binary struct, `throttle_status` (instantaneous) | Medium, versioned layout |
| NVIDIA GPU | NVML `current_throttle_reasons` via `nvml-wrapper` | Good, but bit meanings not yet sourced |
| NVMe | Temperature vs `_crit` only | Weak |

**Crates:** sysinfo skip (no fans or throttle; IDs use the unstable `hwmonN` index). lm-sensors skip (C library). `nvml-wrapper` 0.13.0 (MIT/Apache, loads NVML at runtime) behind an optional feature. `libmedium` 0.13.4 is a candidate, but plain `std::fs` is enough and better for learning.

**Stable sensor identity:** `hwmonN` is not stable. Key on chip `name` + resolved `device` symlink + `temp*_label` (untested design).

**Naming:** `thermald` is Intel's Linux thermal daemon (github.com/intel/thermal_daemon), packaged in Debian, Ubuntu, Arch and others. Do not reuse it.

**Unverified:** actual file modes of `thermal_throttle/*` and `gpu_metrics` on a real kernel; systemd sandbox effects on `/sys`.

## C. Architecture

**Recommended layout (4 crates):**

```
Cargo.toml          workspace: resolver = "3", edition = "2024", [workspace.package], [workspace.dependencies], [workspace.lints]
crates/
  thermals-core/    lib  Domain types, Backend trait, cfg-gated linux/ and macos/ modules, FakeBackend (feature "fake")
  thermals-store/   lib  SQLite schema, migrations (PRAGMA user_version), record / series / prune, default DB path
  thermalsd/        bin  Collector: config, sampling loop, shutdown, retention. Run by launchd / systemd --user.
  thermals/         bin  CLI report first, GUI graphs later. Opens the DB read-only.
```

The store crate exists so the daemon and UI share one copy of the SQL. Rejected: per-OS crates, storage in the daemon or core, one binary with subcommands (daemon would link the GUI), a config crate.

**Type sketch:** `Celsius(f32)` newtype, `SensorKind` enum, `SensorId(String)` stable key, `Sensor`, `Reading`, `ThrottleState`. `trait Backend { fn sensors(&mut self); fn read(&mut self); fn throttle(&mut self); }` returning `Result`. Static dispatch: `cfg` picks a `Native` type alias per OS; the daemon is generic over `B: Backend` so tests use `FakeBackend`. Errors: `thiserror` 2.0.21 in libraries, `anyhow` 1.0.105 in binaries.

**Daemon:** user-level LaunchAgent and systemd user unit, no root. Plain `std` loop on a deadline (`Instant`), `mpsc::recv_timeout` doubles as sleep and shutdown signal, `ctrlc` 3.5.3 for SIGINT/SIGTERM (SIGTERM via `termination` feature, untested). No tokio.

**Daemon to UI:** the UI opens the SQLite file read-only (WAL mode) and polls for new rows every 1-5 s. No IPC. Unix socket only if this feels laggy.

**UI toolkit (stripped, size-tuned release, arm64, measured):**

| Toolkit | Size | Graphs | Learning curve |
|---|---|---|---|
| ratatui 0.30.2 (terminal) | 0.52 MB | Built-in `Chart` | Low |
| eframe 0.37.0 + egui_plot 0.38.0 | 3.21 MB (glow) / 6.05 MB (wgpu) | Zoom, pan, multi-line | Low. Note: 0.37 uses `fn ui(&mut self, ui, frame)`; older tutorials using `update()` are stale. |
| iced 0.14.0 | 4.14 MB | No chart widget | Medium-high |
| Slint 1.19.0 | 7.41 MB | No chart widget | Medium |
| Tauri 2 | <0.6 MB per docs (not built) | JS libraries | High (two languages, webview) |

Recommendation: CLI report first, then eframe + egui_plot with `glow`.

**Storage:** rusqlite 0.43.0 with `bundled` (daemon probe 1.50 MB). Rejected sqlx (async, tokio) and diesel (heavy). Schema: `sensor`, `reading (ts, sensor_id, centi_c) WITHOUT ROWID, PK (ts, sensor_id)`, `reading_1m` (min/avg/max), `throttle_event` (transitions only). Measured for 20 sensors: 30 days of raw 5 s samples ≈ 158 MB; 48 h raw + 30 days of 1-minute rollups ≈ 51 MB. Rollup deferred until real sensor counts are known.

**Config:** TOML via `toml` 1.2.2 + serde; `directories` 6.0.0 `ProjectDirs::from("", "", "thermals")` (macOS path verified: `~/Library/Application Support/thermals`). First keys: `sample_interval_secs = 5`, `retention_days = 30`, optional `db_path`. Missing file means defaults; `deny_unknown_fields`.

**Testability:** Linux parsers take a root `&Path` and run against committed fixture trees, so they compile and test on the Mac. macOS conversions are pure functions over raw bytes. Hardware tests are `#[ignore]`. Time is injected as `now: i64`.

**Build order (each step runnable):**

| # | Step | Rust concepts |
|---|---|---|
| 1 | Workspace + core types; daemon prints a hard-coded reading | workspaces, structs, enums, derive, `Display`, newtype |
| 2 | `Backend` trait + `FakeBackend`; timed loop | traits, generics, iterators, borrowing, `Instant` |
| 3 | Linux hwmon backend + fixture tests | `Result`, `?`, error enums, `std::fs`, `Path`, `cfg`, tests |
| 4 | macOS backend | FFI, `unsafe` behind safe wrappers, `Drop`, target deps |
| 5 | Throttle detection, print transitions | enums with data, `match`, `Option`, state diffing |
| 6 | `thermals-store`, daemon writes rows | external crates, lifetimes, transactions |
| 7 | Clean shutdown + retention prune | closures, `move`, `mpsc`, threads |
| 8 | Config file | serde, `Default`, `Option` combinators |
| 9 | `thermals report` CLI | args, SQL aggregates, formatting |
| 10 | GUI: eframe + egui_plot | immediate-mode UI, state |
| 11 | Service install/uninstall (plist, unit) | `include_str!`, `Command`, permissions |
| 12 | 1-minute rollup | SQL transactions, time-injected tests |
| 13 | Release profile, cargo-deny, Homebrew | profiles, features, supply chain |

## D. Privacy and security

**Key finding:** no Cargo tool can prove "no network". `cargo-deny` bans crates, but `std::net` or raw `libc::socket` bypass it. A credible guarantee has three layers:

1. `cargo deny` bans on network crates (CI).
2. A runtime test that the collector opens no `AF_INET`/`AF_INET6` socket (e.g. under `strace` or `unshare -rn` on Linux CI). Optionally Clippy `disallowed-types` for `std::net` (unverified).
3. OS confinement: systemd user unit `RestrictAddressFamilies=AF_UNIX` (best fit for user units), plus `NoNewPrivileges=yes`, `UMask=0077`, `SystemCallFilter=@system-service`. `PrivateNetwork=`/`ProtectSystem=` depend on user namespaces; `IPAddressDeny=` is likely a no-op in user units. Verify with `systemd-analyze --user security`. macOS has no cheap equivalent; App Sandbox needs signing and entitlements (and would also block IOHID).

**Dependencies checked:** eframe/egui, sysinfo and rusqlite pull in no network crates. Agent claim (unverified by me): eframe's non-default `inspection` feature can open a TCP port; ban `egui_inspection`. Tauri's updater is an opt-in plugin; its webview is a large surface. iced, ratatui, nvml-wrapper not checked.

**Data at rest:** data dir mode 0700, DB 0600. Store no serials, UUIDs, MAC addresses or hostnames. No SQLCipher (pulls OpenSSL). Retention deletion is itself a privacy feature.

**IPC:** none if the UI reads SQLite. If ever added: Unix socket 0600, never TCP localhost.

**Telemetry:** none. Suggested README line: "This software makes no network connections. It does not collect telemetry, report crashes, or check for updates." Only claim what CI tests.

**macOS signing:** Homebrew or `cargo install` from source avoids quarantine. Signed and notarized binaries need a paid Apple Developer account.

**Starter `deny.toml`** (agent-written, syntax checked against cargo-deny docs, not yet run):

```toml
[graph]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
]
exclude-dev = true

[advisories]
unmaintained = "all"
unsound = "all"
yanked = "deny"
ignore = []

[licenses]
confidence-threshold = 0.8
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Zlib",
    "MPL-2.0",
    "CC0-1.0",
    "GPL-3.0-or-later",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"
allow-wildcard-paths = true
external-default-features = "allow"
deny = [
    { crate = "reqwest",      reason = "no network" },
    { crate = "hyper",        reason = "no network" },
    { crate = "ureq",         reason = "no network" },
    { crate = "isahc",        reason = "no network" },
    { crate = "surf",         reason = "no network" },
    { crate = "attohttpc",    reason = "no network" },
    { crate = "curl",         reason = "no network" },
    { crate = "h2",           reason = "no network" },
    { crate = "tungstenite",  reason = "no network" },
    { crate = "native-tls",   reason = "no network" },
    { crate = "openssl",      reason = "no network" },
    { crate = "rustls",       reason = "no network" },
    { crate = "sentry",       reason = "no telemetry" },
    { crate = "tauri-plugin-updater", reason = "no update checker" },
    { crate = "egui_inspection", reason = "opens a TCP port" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

The `deny` entry key is `crate`. Wave 5 confirmed this, and that every key in this file parses, by reading cargo-deny 0.22.0's `package_spec.rs:70` (source read, not executed). Note `WTFPL` (the `smc` crate) is not in the allow-list; irrelevant unless that crate is used.

---

## Integration notes (skill)

- **sysinfo:** all three platform-facing agents agree it is too thin (no fans, no throttle, unstable Linux IDs). Write own backends; use sysinfo and macmon as reference code only.
- **Throttle model must allow "unknown".** macOS gives a 5-level OS pressure value; Linux Intel gives cumulative event counters; AMD CPU gives nothing reliable. The architect's `ThrottleLevel { None, Light, Moderate, Heavy, Critical }` needs an "unsupported/unknown" state and probably a separate "event count" shape for Linux.
- **macOS temperatures require private APIs.** Public APIs give the throttle signal but no per-sensor temperatures on Apple Silicon. This is the biggest decision for Omer.
- **App Store is already out** (GPL, from the earlier chat), which also removes the sandbox conflict with IOHID.
- **No hardcoded per-chip temperature limits.** Both platform agents found no reliable tables; OS signals are the source of truth. This reframes the chat's "needs an idea about each hardware's throttling limit".
- **Unverified items carried forward:** Linux file modes for throttle counters; LaunchAgent sensor access (not executed); `ctrlc` SIGTERM; read-only WAL open; NVML bit meanings; Intel Mac behaviour; whether macOS pressure actually rises under throttling (not induced).
- **Toolchain:** builds used `~/.cargo/registry`, which is not in the session's readable directories.

## Missing agents

None.
