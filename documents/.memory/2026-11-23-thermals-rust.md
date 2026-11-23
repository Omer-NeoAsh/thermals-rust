# Memory: thermals-rust (2026-11-23)

## Wave 0: Setup & discovery

- Classification: new app (app proposal). Guided learning build; Claude writes code in chat only, never to project source files.
- Repo state: README.md, LICENSE, one commit. No Cargo.toml yet.
- documents/ created at Omer's choice.
- No graphify-out/ graph.
- claude.ai chat https://claude.ai/chat/b254ae1d-fd78-4df9-b863-e81bf706f3bc needs sign-in; Omer signing in via the Browser pane. Read it before Wave 1 spawns so research doesn't redo decisions already made there.
- Skill reference doc-structures.md is outside readable dirs; using the section list from SKILL.md instead.

### claude.ai chat read (2026-11-23)

Summarized into project-specs.md "Context from the earlier claude.ai chat". Notable for research:
- Throttling detection is the headline feature; needs per-hardware throttle limits or OS throttle signals.
- Daemon + SQLite (30 days, configurable) + graphing UI.
- Suggested crate name `thermald` collides with Intel's Linux thermal daemon (`thermald` package/service). Raise in Wave 2.
- License GPL-3.0-or-later decided; LICENSE already GPLv3.

### Spawn roster (Wave 1 plan, pending chat context)

- engineering:Software Architect @ opus: workspace/crate split, sensor trait design, UI options (TUI vs GUI), local storage.
- general-purpose @ sonnet: platform sensor access. macOS (SMC/IOKit, Apple Silicon HID sensors, root needs), Linux (hwmon, thermal_zone, throttling signals). Existing crates and maintenance state.
- engineering:Security Engineer @ sonnet: privacy guarantees (no network, dependency auditing with cargo-deny, data at rest, permissions).
- Inline: feature-by-feature learning roadmap.

## Wave 1: agent returns (raw notes, consolidate into wave1-results.md)

### B. Linux (general-purpose @ sonnet): returned, usable
- hwmon `temp*_input/_label/_crit/_max/_fault`, millidegrees, world-readable (docs.kernel.org/hwmon/sysfs-interface.html). Fans `fan*_input`.
- k10temp: temp1=Tctl (arbitrary scale), temp2=Tdie, temp3..=Tccd; prefer Tdie. amdgpu hwmon temp1-3 S_IRUGO.
- Throttle: Intel `/sys/devices/system/cpu/cpuX/thermal_throttle/{core,package}_throttle_count`, `_total_time_ms` cumulative since boot; dedupe package. Best signal. AMD CPU: no counter found (weak: freq drop, Tctl vs crit). ARM: cooling_device cur_state. amdgpu `gpu_metrics` binary struct, instantaneous throttle_status. NVIDIA: nvml-wrapper `current_throttle_reasons` (bit docs 404). NVMe: temp vs crit only.
- Crates: sysinfo 0.40.4 (MIT) skip (no fans/throttle, unstable hwmonN ids); lm-sensors skip (C dep); nvml-wrapper 0.13.0 (MIT/Apache, runtime libloading) use behind feature; libmedium 0.13.4 (MIT) candidate vs plain std::fs.
- Identity: `name` + resolved `device` symlink + label. Not tested.
- thermald collision confirmed (intel/thermal_daemon, packaged everywhere).
- Unknowns: file modes of thermal_throttle/gpu_metrics on real kernel; NVML bit meanings; systemd sandbox vs /sys; AMD CPU throttle policy; NVML optional feature?

### D. Privacy (Security Engineer @ sonnet): returned, usable
- No cargo-level proof of "no network": cargo-deny bans crates only; std::net/libc bypass it. Three layers: crate bans, runtime test (strace/unshare in CI, asserting no AF_INET/6), OS confinement where available. Clippy disallowed-types for std::net (unverified).
- systemd user unit: RestrictAddressFamilies=AF_UNIX best fit; NoNewPrivileges, UMask=0077, SystemCallFilter=@system-service. PrivateNetwork/ProtectSystem need user namespaces (conflicting docs), IPAddressDeny likely no-op in user units. Verify with `systemd-analyze --user security`.
- macOS: no cheap network deny; App Sandbox needs signing + entitlements.
- Deps checked: eframe/egui, sysinfo, rusqlite have no network crates (agent claim: eframe `inspection` feature opens TCP if env var set; ban egui_inspection). Tauri updater is opt-in plugin; webview = large surface. iced/ratatui/nvml-wrapper unchecked.
- Starter deny.toml provided (targets 4 triples, advisories, license allow-list incl GPL-3.0-or-later, bans list of network crates + sentry + tauri-plugin-updater + egui_inspection, sources crates.io only). Copy verbatim into wave1-results.md.
- Data at rest: XDG data dir / ~/Library/Application Support, dir 0700, file 0600. Don't store serials/UUIDs/hostnames. No SQLCipher.
- IPC: if any, Unix socket 0600, never TCP localhost. Or UI reads SQLite read-only (no IPC).
- macOS: powermetrics needs root; IOPMPowerSource temperature regression reported on macOS 27 (forum). Signing: Homebrew/cargo install avoids quarantine; notarization needs paid account.
- Questions: hard macOS network guarantee? UI toolkit? UI reads SQLite directly? root helper ever OK? distribute binaries? AF_NETLINK? SystemCallFilter now or later?

### C. Architecture (Software Architect @ opus): returned, usable, sizes MEASURED on this Mac
- Layout (4 crates): thermals-core (lib: types, Backend trait, cfg-gated linux/macos modules, FakeBackend behind `fake` feature), thermals-store (lib: rusqlite schema, migrations via user_version, record/series/prune, default_db_path), thermalsd (bin: config, std-only deadline loop, mpsc recv_timeout + ctrlc for shutdown, prune), thermals (bin: CLI report then egui GUI, DB read-only). Rejected per-OS crates, storage in daemon/core, single binary with subcommands, config crate, name thermald.
- Types: Celsius newtype, SensorKind enum, SensorId(String) stable key, Sensor, Reading, ThrottleLevel, ThrottleState; trait Backend { sensors, read, throttle } with &mut self; static dispatch via cfg `Native` alias; thiserror 2.0.21 libs, anyhow 1.0.105 bins.
- Daemon: user-level LaunchAgent + systemd --user (linger caveat). No tokio.
- Data path: UI reads SQLite WAL directly, polls 1-5s. Unknown: read-only WAL needs -shm.
- UI sizes (stripped, tuned profile, arm64): ratatui 0.30.2 0.52 MB; eframe 0.37.0+egui_plot 0.38.0 6.05 MB wgpu / 3.21 MB glow; iced 0.14.0 4.14 MB (no chart widget); Slint 1.19.0 7.41 MB; Tauri <600KB per docs, two languages. eframe 0.37 App trait uses `fn ui(&mut self, ui, frame)` (old update() stale).
- rusqlite 0.43.0 bundled (daemon probe 1.50 MB). Schema: sensor, reading(ts,sensor_id,centi_c) WITHOUT ROWID PK(ts,sensor_id), reading_1m min/avg/max, throttle_event (transitions). Measured 20 sensors: 30d raw@5s ~158 MB extrapolated; 48h raw + 30d 1m rollup ~51 MB. Rollup deferred to step 12.
- Config: toml 1.2.2 + serde, directories 6.0.0 ProjectDirs("", "", "thermals"); keys sample_interval_secs=5, retention_days=30, db_path; deny_unknown_fields.
- Tests: sysfs parsers take root &Path, fixture trees, compile on all OSes; inject `now: i64`.
- 13-step roadmap (workspace+types -> FakeBackend loop -> Linux hwmon -> macOS -> throttle -> store -> shutdown/prune -> config -> report CLI -> egui GUI -> service install -> rollup -> release/cargo-deny/Homebrew).
- Decisions: names; 4th store crate; Linux or macOS first; root; GUI vs TUI; interval/raw retention; thresholds; bundled vs system sqlite.
- Toolchain: ~/.cargo not in additionalDirectories (CLAUDE.md toolchain rule: ask Omer add/no/elsewhere). Scratchpad probe workspace `sizes/` has large target/.

### A. macOS (general-purpose @ opus): returned, usable, EXEC on M3 Air macOS 27.2.1
- Full detail in wave1-results.md section A. Key: pressure notify key public/no root; IOHID + SMC private, no root; powermetrics root; no M-series throttle temp tables; SMC Tp* 115C at nominal.

## Wave 1: complete
- wave1-results.md written; Research Summary appended to project-specs.

## Wave 2: questions asked (2026-11-23)
15 questions, table in project-specs.md "Questions & Answers". Waiting on Omer.

## Wave 3: User answers (2026-11-23)
Verbatim-equivalent answers recorded in project-specs Q&A table. Highlights:
- Service name `neem-thermals` (company neemtech). 4 crates OK. Linux first. Private macOS APIs OK.
- Linux box: AMD Ryzen 9 9900X (Zen 5) + Radeon RX 9070 XT (RDNA4, Sapphire Nitro+). Both AMD, so the weak-AMD-throttle finding is now central.
- Throttle: OS signal first, fallback (may need per-chip data) second.
- Avoid root, not "never".
- Notifications + per-sensor thresholds: backlog.
- Terminal UI (ratatui) first. GUI later; export-as-graph idea or multiple clients.
- Storage plan, privacy bar accepted. Source-only distribution.
- Delivery format agreed; may run cargo build/test/clippy.
- ~/.cargo added to additionalDirectories.

Contradictions to flag:
- Omer: "M-chips do not publish when they throttle". Research: macOS publishes the thermal pressure level (public notify key) on Apple Silicon. Unverified whether it rises under real throttling.
- Omer: "Intel and AMD normally do". Research: Linux exposes Intel throttle counters; no AMD CPU equivalent found. AMD publishes Tjmax (spec), not a runtime OS signal. Wave 4 must research Zen 5 / RDNA4 specifically.

Follow-ups (F1-F3) answered: prefix everything neem-thermals-*, TUI = neem-thermals-tui, label com.neemtech.neem-thermals; inferred throttle labelled "Likely"; no export for now. Recorded in project-specs.

## Wave 4: Gap analysis (started 2026-11-23)
wave4-setup.md written. Spawned: G1 AMD Zen5/RDNA4 (general-purpose @ opus), G2 architecture gaps (Software Architect @ opus), G3 test/CI (DevOps Automator @ sonnet). Inline: docs gaps, TUI accessibility (don't rely on color alone), residual security.
Verify in results: 9900X Tjmax 95 C (I stated it to Omer; unverified).

### G3 test/CI (DevOps Automator @ sonnet): returned, usable
Full report in task output a0b13ce51f1600bb6; consolidate into wave4-results.md section G3.
- Checked: remote origin is GitHub (Omer-NeoAsh/thermals-rust), visibility unknown. rustc/cargo 1.99.0 stable aarch64 only. cargo-deny NOT installed, so starter deny.toml never run.
- EXEC on this Mac: sandbox-exec `(deny network* (with send-signal SIGKILL))` kills on connect (exit 137); AF_INET socket creation alone not blocked. clippy.toml disallowed-types flags std::net::TcpStream.
- Parsers must compile on all OSes; only `Native` alias cfg-gated. Identity via `fs::read_link(...).file_name()`, not canonicalize (fixture symlinks dangle).
- Collector = lib.rs + thin main.rs; `tick(&mut backend, &mut store, now)`. Needs `--samples N` and `fake` feature for deterministic no-network e2e.
- Fixtures: capture-sysfs.sh (allowlist, no root, dangling device symlink, not executed); dirs ryzen9-9900x-rx9070xt, intel-synthetic, hwmon-renumbered, edge-cases. Scrub with rg + strings on gpu_metrics.
- no-network-check.sh: strace on Linux (non-AF_UNIX socket fails), sandbox-exec on macOS; negative control with curl must fail.
- CI YAML (fmt, clippy -D warnings, test, strace, fake build, no-network, deny job). Unverified: actions/checkout@v7, cargo-deny-action@v2, macos-latest arm64, strace/ptrace on ubuntu-latest, sandbox-exec on hosted runner, env!("CARGO_BIN_EXE_...").
- Lints: unsafe_code deny (not forbid; macOS FFI allows), unwrap/expect/dbg/todo/undocumented_unsafe/cast lints warn. No rust-toolchain.toml, no rust-version, no rustfmt.toml.
- Needs .gitignore target/ and committed Cargo.lock.
- Owner questions: repo public/private; --samples + fake feature (rec yes); zero sensors -> exit 0 + warn (rec); commit PCI-address symlinks (rec yes); weekly deny (rec no).

### G2 architecture (Software Architect @ opus): returned, usable
Full report in task output a31baebf30be35d8e; consolidate into wave4-results.md section G2 (gap table, type sketch, schema v1, 14-step roadmap, backlog).
- EXEC probes (scratchpad arch-probe, sig-probe): read-only WAL open works (missing DB fails without creating; leaves -wal/-shm; sees live commits; survives writer crash); unwritable dir -> first query fails, so treat like open error. File::try_lock stable on 1.99. ctrlc 3.5.3 `termination` feature handles SIGTERM. WAL volume 20 sensors/5 s: PK (ts,sensor) 152 MB/day vs (sensor,ts) 1.27 GB/day.
- Read in crate src: ratatui 0.30.2 `ratatui::run/init/restore` + panic hook; directories ProjectDirs::from("", "", "neem-thermals") (NOT com/neemtech, which renames mac folder).
- Key changes: throttle stored as samples in `reading` (drop throttle_event); Kind{Temperature, Throttle(Source), Pressure}, Source{Os, Inferred}, DeviceId scope; Celsius::new rejects non-finite/outside (0,150]; counter_throttled(prev, now) -> Option<bool>; infer.rs const LIMITS table, driver _crit wins, hysteresis limit-1 on / limit-5 off; Writer/Reader split, Opened{Ready, Missing, NotInitialised, SchemaMismatch}; stale = now - last_ts > 3x interval; TUI never touches lock.
- Loop: Instant deadline, wall ts, next=max(next+interval, now), INSERT OR IGNORE, rollup recompute last 2 min, rescan every 60 s, skip-and-continue on tick errors, exit 78 on startup errors, Restart=on-failure/KeepAlive SuccessfulExit=false.
- Constants instead of config file (SAMPLE_INTERVAL, RAW/ROLLUP_RETENTION); std::env::args, no clap; log + env_logger (features UNVERIFIED).
- Migrations: collector only, VACUUM INTO backup first; --db on both binaries so Claude never touches Omer's real DB. user_version rollback in txn UNVERIFIED (unit test).
- `service install` subcommand (include_str! template, current_exe, launchctl bootstrap / systemctl --user); linger printed not enabled.
- Owner questions: uninstall keeps data + --purge (rec yes); auto linger (rec no); config file to backlog (rec yes); macOS pressure > Nominal = OS throttling (rec yes, revisit after load test).
- ANSWERED by Omer: A1 yes, A2 linger never by default (install may ask, default no; he asked whether linger leaves userspace: explained it doesn't), A3 yes, A4 yes. Recorded in project-specs "Wave 4 questions".
- A5 (Omer, domain neemtech.ca): ProjectDirs::from("ca", "neemtech", "neem-thermals"); mac folder ca.neemtech.neem-thermals, Linux unchanged (lin.rs ignores qualifier/org, verified); launchd label -> ca.neemtech.neem-thermals (supersedes com.neemtech).
- A2 revised by Omer: no linger at all, not even offered (battery; throttling happens while logged in under load).

### G1 AMD (general-purpose @ opus): returned. No unprivileged 9900X throttle signal; Tctl only (no Tdie), Tccd 6.16+; Tjmax 95 C confirmed. 9070 XT gpu_metrics v1_3 throttle_status raw bits (thermal vs power-limited), EPERM = asleep. Board Super I/O unknown. Consolidated in wave4-results.md.

## Wave 4: complete
wave4-results.md written; Gap Analysis in project-specs; status updated. Proposed ThrottleState{None, PowerLimited, AtLimit, Throttling} to reconcile G1/G2 (decide at step 4).
- T1-T6 ANSWERED: public repo; --samples + fake yes; zero sensors exit 0 + warn; PCI symlinks OK; no weekly deny; board Gigabyte X870E AORUS ELITE, Arch (current kernel). In project-specs.

## Wave 5: Verification (started 2026-11-23)
wave5-setup.md written. Inline VERIFIED: user_version rolls back with txn (system sqlite3 CLI: 0 after rollback, 7 after commit). Spawned V1 hardware/kernel (Arch kernel, X870E Super I/O, gpu_metrics offsets, 9900X cpuinfo) and V2 tooling (action tags, runners, clippy keys, CARGO_BIN_EXE, env_logger, launchd Umask/~, deny.toml syntax), both general-purpose @ opus. Next: wave5-results.md, Open Items in spec, CHECKPOINT with Omer.
### V2 tooling: returned
- VERIFIED: checkout@v7 (v7.0.1), cargo-deny-action@v2 (v2.1.3, cargo-deny 0.22.0, DOCKER action -> deny job Linux only); macos-latest = macOS 26 arm64 (runner-images README:32); clippy allow-unwrap-in-tests + allow-expect-in-tests (probe); CARGO_BIN_EXE_<bin name> (probe, test must be in the bin's package); env_logger 0.11.12 default [auto-color, humantime, regex], stderr default (target.rs:8-9); plist Umask <integer or string> (decimal 63 or "077"); deny `{ crate = "x" }` correct (package_spec.rs:70), all starter keys parse in 0.22.0 (source read, not executed).
- REFUTED: strace not preinstalled on ubuntu-latest (24.04) -> `sudo apt-get update && sudo apt-get install -y strace` (plan already installs).
- STILL UNVERIFIED: ptrace on runner (Yama 1 allows descendants; first CI run answers), sandbox-exec on hosted mac runner, launchd ~ expansion (not documented -> write absolute paths).
- Side effect: cargo info downloaded env_logger 0.11.12 source into ~/.cargo/registry cache.
### V1 hardware: returned. Arch linux 7.3.5; tags confirmed; board = ITE IT8696E + IT87952E (mainline lacks IT8696E, out-of-tree frankcrawford/it87, no autoload, MMIO default, avoid sensors-detect); gpu_metrics offsets compile-checked; 9900X model 68, "AMD Ryzen 9 9900X 12-Core Processor".

## Wave 5: complete
wave5-results.md written; corrections applied to wave4-results.md and wave1-results.md:208; Open Items + status in spec. CHECKPOINT: waiting on Omer's go-ahead to finalize (Wave 6).
## Wave 6: finalized 2026-11-23
Omer approved ("Finalize and start"). Spec status Final; Decisions Log + 14-step Implementation plan in Next Steps. /graphify not run (repo nearly empty).

## Build log
- Step 1 (workspace, lints, clippy.toml, core types Celsius/SensorId/Value/Sample, main prints sample): code compile-checked in scratchpad/step1 (clippy -D warnings clean, 3 tests pass, run prints "k10temp@0000:00:18.3/temp1: 48.5 °C" + "rejected reading: NaN"). Given to Omer in chat; waiting for him to type it.
- (old) Still to ask (G3): repo public/private, --samples + fake feature, zero sensors policy, PCI-address symlinks in fixtures, weekly deny.

### Omer's global rules to pass to every spawned agent

Docs written from the code, never memory; grep-verify cross-refs; prefer built-in tools; Read for files, rg for search; no .env/credential reads; never commit; never apply migrations.
