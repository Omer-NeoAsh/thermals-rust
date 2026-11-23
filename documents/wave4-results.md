# Wave 4 Results: Gap Analysis

**Date:** 2026-11-23
**Setup:** `documents/wave4-setup.md`
**Agents:**
- G1, Omer's Linux hardware (general-purpose @ opus). It read kernel source at torvalds/linux master (newest tag v7.3.1) and ran nothing on Linux.
- G2, architecture (Software Architect @ opus). It ran probes on this Mac: rustc 1.99.0, rusqlite 0.43.0 with bundled SQLite 3.54.1.
- G3, test and CI (DevOps Automator @ sonnet). It ran sandbox-exec and clippy probes on this Mac.

**Decisions made during this wave:** A1-A5 in `project-specs.md` "Wave 4 questions".

Claims marked UNVERIFIED were neither run nor read in a primary source.

## Top gaps

1. **The Ryzen 9 9900X has no OS throttle signal that a normal user can read.** CPU throttling on Omer's Linux box can only be inferred (G1). The RX 9070 XT does have one: the `gpu_metrics` throttle bits.
2. **Store throttle state as samples, not transitions.** It goes in the same `reading` table as temperatures, which gives one pipeline for record, rollup, prune and charts. It also removes the open-interval bug after a crash (G2).
3. **Collector and TUI coordination.** The read-only WAL open has now been verified (G2). The TUI must handle a missing DB, a schema mismatch and a stale collector, and it must never touch the collector's lock.
4. **Loop time and failure policy.** Wall-clock `ts` with a monotonic deadline, `INSERT OR IGNORE`, skip-and-continue on tick errors, exit 78 on startup errors, and restart through systemd or launchd (G2).
5. **The privacy promise needs a deterministic runtime test.** That needs a `--samples N` flag, a `fake` feature and a lib/bin split in the collector. A negative control stops the test from passing vacuously (G3).
6. **Board sensors need an out-of-tree driver.** The X870E AORUS ELITE uses ITE IT8696E + IT87952E. Mainline it87 lacks the IT8696E, so board fans need frankcrawford/it87 plus a `modules-load.d` entry, a root step the user does (G1, corrected in Wave 5).

## Corrections to Wave 1

| Wave 1 said | Correct | Source |
|---|---|---|
| k10temp: prefer Tdie (`temp2`) | On the 9900X there is no `temp2`/Tdie. Use `temp1` = Tctl, which has no offset for this chip. `temp3+` = Tccd1..N, kernel 6.16 or later. | k10temp.c lines 124-131, 411-433, 525-547 (G1) |
| Intel `thermal_throttle/*` file mode unverified | 0444; present only when Intel thermal support is enabled | therm_throt.c line 134 (G1) |
| Read-only WAL open needs `-shm` (unknown) | Works. It fails cleanly when the DB is missing. When the folder is not writable, the first query fails, not the open. | G2 probes S1-S5 |
| `throttle_event` table of transitions | Dropped. Throttle state is stored as samples in `reading`. | G2 |
| ProjectDirs `("", "", "thermals")` | `("ca", "neemtech", "neem-thermals")` (A5) | directories-6.0.0 mac.rs 91-97, lin.rs 92-93 |
| Config file (toml + serde) in the first build | Backlog. Interval and retention are constants in the store crate (A3). | Omer |
| 9900X Tjmax 95 °C (unverified) | 95 °C, from AMD's product page via search extracts. G1's direct fetch timed out. | amd.com 9900X page (G1) |

## G1: Omer's Linux hardware

### CPU: Ryzen 9 9900X (Zen 5, family 1Ah = 26)

- **Temperatures:** k10temp, hwmon name `k10temp`, files 0444.
  - `temp1_input` is Tctl.
  - `temp3+` are Tccd, shown only for CCDs whose valid bit is set; Zen 5 desktop support arrived in kernel 6.16. Expect Tccd1 and Tccd2 (UNVERIFIED).
  - There is no `temp1_crit` or `temp1_max` on Zen.
- **OS throttle signal: none.** G1 checked each of these:
  - amd-pstate sysfs (static or hint values only).
  - Intel `thermal_throttle` (Intel only).
  - RAPL: AMD has energy-status registers only, and `energy_uj` is root only.
  - perf RAPL events (need `CAP_PERFMON` at the default `perf_event_paranoid=2`).
  - amd_hsmp (EPYC and MI300A only).
  - PROCHOT/HTC (pre-Zen only).
  - thermal_zone and cooling devices on AM5: UNVERIFIED.
- **Effective clock without root:**
  - `cpuinfo_avg_freq` (0444, kernel 6.15 or later) and `scaling_cur_freq` both use APERF/MPERF over the last tick.
  - On an idle CPU (sample older than 20 ms) the value falls back to the requested frequency, so trust it only for busy CPUs.
  - `cpuinfo_cur_freq` is root only.
- **Specs:** Tjmax 95 °C, base 4.4 GHz, boost 5.6 GHz, 120 W TDP, 12 cores / 24 threads.
  - AMD regulates Zen 4 to hold 95 °C under load (TechPowerUp, for Zen 4).
  - So sitting at 95 °C means boost is temperature-limited. That alone is not throttling below spec.
- **Identification:**
  - `/proc/cpuinfo`: `vendor_id == AuthenticAMD`, `cpu family == 26`, `model` in 0x40..0x4f.
  - The model name contains the whole token `9900X`, so `9900X3D` must not match. Verified in Wave 5: `model : 68` (0x44), `model name : AMD Ryzen 9 9900X 12-Core Processor`.

### GPU: RX 9070 XT (Navi 48, GC 12.0.1, MP1 14.0.3)

- **hwmon** (name `amdgpu`, all `S_IRUGO`):
  - `temp1..3_input` labelled `edge`, `junction`, `mem`.
  - `_crit`, `_crit_hyst` and `_emergency` come from the board's own limits, so no per-chip table is needed.
  - `power1_average` present, `power1_input` hidden.
  - `fan1_input` from kernel 6.15.
  - `freq1_input` is sclk.
  - `gpu_busy_percent` and `pp_dpm_sclk` are readable by everyone.
- **`gpu_metrics`** (`/sys/class/drm/card*/device/gpu_metrics`, 0444):
  - Format: SMU 14.0.2/3 emits `gpu_metrics_v1_3` (header: size u16, format 1, content 3), in mainline since 6.12.
  - Unset fields are all 0xFF.
  - Layout: compile-checked for x86_64 Linux with `_Static_assert` in Wave 5. Still assert `structure_size == 120` at runtime. `throttle_status` (u32) is at offset 68 and `indep_throttle_status` (u64) at offset 112.
  - When the GPU is runtime-suspended, the read returns **EPERM**. Treat that as "asleep, not throttling", not as a permissions error.
- **Decoding:**
  - Decode the raw firmware bits of `throttle_status` (smu14_driver_if_v14_0.h lines 195-216): 0 TEMP_EDGE, 1 TEMP_HOTSPOT, 2 HOTSPOT_GFX, 3 HOTSPOT_SOC, 4 TEMP_MEM, 5 VR_GFX, 6 VR_SOC, 7 VR_MEM0, 8 VR_MEM1, 9 LIQUID0, 10 LIQUID1, 11 PLX, 12 TDC_GFX, 13 TDC_SOC, 14-17 PPT0-3, 18 FIT, 19 GFX_APCC_PLUS, 20 GFX_DVO.
  - The kernel's generic `indep_throttle_status` map for SMU14 has holes: bits 2, 3 and 11 map to PPT0, and bit 20 indexes past the end of the map. That comes from reading the code, not running it.
- **Classes:**
  - TEMP_*, HOTSPOT_* and VR_* bits mean **"Throttling (OS reported)"**.
  - PPT, TDC and FIT bits mean **power-limited**. That is normal under gaming load, so it is not shown as thermal throttling.

### Board and NVMe

- **Board Super I/O** (Gigabyte X870E AORUS ELITE, corrected in Wave 5):
  - Two ITE chips: IT8696E (main) and IT87952E (second). This comes from user reports in the frankcrawford/it87 repo for the WIFI7 and X3D variants.
  - Mainline it87 has no IT8696E. Full support needs the out-of-tree frankcrawford/it87 driver (DKMS), which uses MMIO by default, so no `ignore_resource_conflict` is needed.
  - Neither driver autoloads on this board, so the user adds a `modules-load.d` entry. That is a root step the user does, never the app.
  - Don't run `sensors-detect`: a contributor reports it can leave the second chip stuck until a cold boot.
  - The app treats board sensors as optional.
- **NVMe** (hwmon `nvme`, 0444):
  - Labels `Composite` and `Sensor 1..8`.
  - `temp1_crit` = CCTEMP, `temp1_max` = WCTEMP, `temp1_alarm` = SMART critical temperature bit.
  - The only throttle signal without root is `temp1_alarm`. The thermal-management counters need `/dev/nvme*` (root, UNVERIFIED).

### Minimum kernel

| Feature | Kernel |
|---|---|
| SMU14 `gpu_metrics` | 6.12 |
| RDNA 4 launch support (Phoronix, secondary source) | 6.14 |
| amdgpu `fan1_input`, `cpuinfo_avg_freq` | 6.15 |
| Zen 5 per-CCD Tccd | 6.16 |

Tccd and fan readings are optional: the app keeps working without them. Wave 5 confirmed the first tags (2c183963 in v6.16-rc1; 90df6db6 and fbb4a475 in v6.15-rc1). Arch `linux` is 7.3.5, so Omer's box has all of them.

### Inferred heuristics

These thresholds are named constants (A3: no config file yet). They are calibration values to tune against real load runs on the box.

**9900X.** Per-chip entry: `tjmax_c = 95`, `base_mhz = 4400`.

Each sample reads:
- Tctl.
- The per-CPU busy fraction from `/proc/stat` deltas.
- The effective MHz of CPUs that are at least 90 % busy.

States:
- `at_limit` = Tctl ≥ tjmax − 1 for 3 consecutive samples.
- `loaded` = mean busy ≥ 50 %.
- **"Likely throttling (inferred)"** = `at_limit && loaded && median busy MHz < 0.98 × base_mhz`.
- `at_limit && loaded` with clocks at or above base gives a separate informational state, **"At thermal limit (boost regulated)"**.

**9070 XT.** The fallback is used only when `gpu_metrics` is unreadable or its version is unknown. All of these must hold:
- junction ≥ `temp2_crit` − 2 °C
- `gpu_busy_percent` ≥ 80
- `freq1_input` < 0.9 × the top `pp_dpm_sclk` level
- `power_dpm_force_performance_level == auto`

## G2: Architecture

### Verified on this Mac

- **Read-only WAL open (probes S1-S5):**
  - A missing DB fails to open and nothing is created.
  - The open works after a clean writer close and leaves `-wal`/`-shm` behind.
  - It sees live commits and survives a writer crash.
  - An unwritable folder makes the first query fail, not the open.
- **`std::fs::File::try_lock`:** returns `Err(WouldBlock)` on a second handle. Stable on 1.99.
- **ctrlc 3.5.3 with the `termination` feature:** handles SIGINT, SIGTERM and SIGHUP. Checked by probe and in `ctrlc-3.5.3/src/platform/unix/mod.rs:98-123`.
- **WAL write volume** (20 sensors, 5 s, `synchronous=NORMAL`, one commit per tick):
  - PK `(ts, sensor)`: about 152 MB/day.
  - PK `(sensor, ts)`: about 1.27 GB/day.
  - Keep `(ts, sensor)`.
- **ratatui 0.30.2:** `ratatui::run`/`init`/`restore` exist, and `init` installs a terminal-restoring panic hook (`ratatui-0.30.2/src/init.rs:318-367`). ratatui re-exports crossterm.

### Gap resolutions

| Area | Resolution |
|---|---|
| Throttle model | `Kind::{Temperature, Throttle(Source), Pressure}`, `Source::{Os, Inferred}`, and a `DeviceId` scope. Unknown = no sample this tick. Unsupported = a device with no throttle sensor. |
| Throttle storage | Samples in `reading`: the throttle value, or the macOS pressure level 0..4. Rollup max = "throttled at some point that minute". |
| Linux counters | `counter_throttled(prev, now) -> Option<bool>`. Returns `None` on the first tick or when the counter goes backwards. |
| Per-chip limits | Compiled-in `const` table in core (`infer.rs`). A driver-reported limit (`_crit`) wins. Looked up once at discovery. A user override file goes to the backlog. |
| Flapping | Inferred state turns on as defined above and turns off below limit − 5 °C. Both are named constants. |
| Config and arguments | Constants `SAMPLE_INTERVAL` (5 s), `RAW_RETENTION_SECS` (48 h) and `ROLLUP_RETENTION_SECS` (30 d) in the store crate. `std::env::args` with `match`, no clap: `neem-thermals [run] [--db PATH]`, `neem-thermals service install\|uninstall [--purge]`, `neem-thermals-tui [--db PATH]`. |
| Suspend and resume | The deadline uses `Instant`; the stored `ts` is wall-clock unix seconds (UTC). The TUI splits a chart series where a gap is more than 2 × the interval. |
| Stall burst | `next = max(next + interval, now)`. |
| Same-second restart, or clock stepped back | `INSERT OR IGNORE` on `reading`. |
| Rollup | Each minute, recompute the last 2 minutes with `INSERT OR REPLACE`. At startup, catch up from `max(reading_1m.ts)`, bounded by raw retention. |
| Forward clock jump during prune | Accepted. OPEN (low). |
| Hot-plug and `hwmonN` renumbering | Rescan every 12 ticks. A sensor that fails to read is skipped for that tick. The stable `SensorId` keeps the series. |
| Duplicate labels | Keep the id separate from the label. Linux id is `{chip}@{device}/{tempN}`. macOS duplicates: OPEN until step 12. |
| Junk values | `Celsius::new(f32) -> Option<Celsius>` rejects non-finite values and anything outside (0, 150] °C. |
| DB busy, disk full, I/O errors | `busy_timeout` 1 s. Skip the tick and continue. Log only when the error state changes. No in-memory backlog. |
| Single instance | `File::try_lock` on `<data_dir>/collector.lock`. The TUI never touches it; it decides staleness from `now - last_ts > 3 × SAMPLE_INTERVAL`. |
| Budget (fanless M3) | One thread, one wakeup per tick, handles kept open, `prepare_cached`, one transaction per tick, rollup each minute, prune each hour. LaunchAgent `ProcessType=Background`, `LowPriorityIO=true`. Targets: under 0.1 % CPU and under 15 MB RSS (UNVERIFIED; measure in step 13). |
| Logging | `log` macros in the libraries and `env_logger` 0.11.12 in the binaries; it writes to stderr by default. Default features are `auto-color`, `humantime` and `regex`; `default-features = false` gives a smaller dependency tree. systemd sends stderr to the journal. launchd uses `StandardErrorPath` set to `~/Library/Logs/neem-thermals/neem-thermals.log` as an absolute path. Log only startup, discovery changes, error-state changes and shutdown. |
| Panic and restart | No `catch_unwind`. systemd: `Restart=on-failure`, `RestartSec=10`, `RestartPreventExitStatus=78`. launchd: `KeepAlive{SuccessfulExit=false}`. Exit 78 on startup errors (newer schema, unusable data dir). |
| Migrations | `const MIGRATIONS: &[&str]`, applied only by the collector at startup. Each runs in a transaction that also sets `user_version`; a unit test checks that `user_version` rolls back with it (UNVERIFIED). `VACUUM INTO '<db>.v{N}.bak'` first. Exit 78 if the DB is newer than the binary. |
| Development safety | `--db PATH` on both binaries. Claude only runs tests against temp or in-memory DBs and never runs the collector without a scratch `--db`. |
| TUI open states | `Opened::{Ready(Reader), Missing, NotInitialised, SchemaMismatch{found, expected}}`, plus a stale banner. Errors on the first query are treated like open errors. Never keep a transaction open across polls. Poll with `crossterm::event::poll(1 s)` and re-query only when `last_ts` changes. |
| Service install | `neem-thermals service install` fills an `include_str!` template with `current_exe()`. It then runs `launchctl bootout` + `bootstrap gui/<uid>`, or `systemctl --user daemon-reload` + `enable --now`. It warns when the exe path contains `/target/`. Linger is never enabled or offered; the collector runs only while the user is logged in (A2). Uninstall keeps data; `--purge` deletes it (A1). |
| Upgrade | `cargo install --path crates/<crate> --locked`, then run `service install` again. A stale TUI shows `SchemaMismatch`. |
| Several OS users | One collector per user, accepted. A system-wide daemon goes to the backlog (needs root). |

### Types (signatures only)

```rust
// neem-thermals-core
pub struct Celsius(f32);
impl Celsius { pub fn new(raw: f32) -> Option<Celsius>; pub fn get(self) -> f32; }

pub struct SensorId(String);   // e.g. "k10temp@0000:00:18.3/temp1"
pub struct DeviceId(String);   // "cpu", "gpu@0000:03:00.0", "nvme@nvme0", "system"

pub enum Source { Os, Inferred }
#[repr(u8)] pub enum Pressure { Nominal, Moderate, Heavy, Trapping, Sleeping }
pub enum Kind { Temperature, Throttle(Source), Pressure }

pub struct Sensor {
    pub id: SensorId,
    pub label: String,
    pub device: DeviceId,
    pub kind: Kind,
    pub limit: Option<Celsius>,
}

pub enum Value { Temperature(Celsius), Throttled(bool), Pressure(Pressure) }
pub struct Sample { pub sensor: SensorId, pub value: Value }

pub trait Backend {
    fn discover(&mut self) -> Result<Vec<Sensor>, BackendError>;
    fn sample(&mut self) -> Result<Vec<Sample>, BackendError>;
}
#[cfg(target_os = "linux")] pub type Native = linux::Hwmon;
#[cfg(target_os = "macos")] pub type Native = macos::Iohid;

fn counter_throttled(prev: Option<u64>, now: u64) -> Option<bool>;

// infer.rs
pub fn table_limit(chip: &str) -> Option<Celsius>;
pub struct Inferrer { /* per device: hot sensor ids, limit, active */ }
impl Inferrer {
    pub fn new(sensors: &[Sensor]) -> (Inferrer, Vec<Sensor>);
    pub fn step(&mut self, samples: &[Sample]) -> Vec<Sample>;
}

// neem-thermals-store
pub const SCHEMA_VERSION: i32;
pub const SAMPLE_INTERVAL: Duration;
pub const RAW_RETENTION_SECS: i64;
pub const ROLLUP_RETENTION_SECS: i64;
pub fn data_dir() -> Result<PathBuf, StoreError>;
pub fn default_db_path() -> Result<PathBuf, StoreError>;

pub struct Writer { /* Connection, HashMap<SensorId, i64> */ }
impl Writer {
    pub fn open(path: &Path) -> Result<Writer, StoreError>;
    pub fn register(&mut self, sensors: &[Sensor]) -> Result<(), StoreError>;
    pub fn record(&mut self, ts: i64, samples: &[Sample]) -> Result<(), StoreError>;
    pub fn rollup(&mut self, from: i64, to: i64) -> Result<(), StoreError>;
    pub fn prune(&mut self, now: i64) -> Result<(), StoreError>;
}
pub enum Opened { Ready(Reader), Missing, NotInitialised, SchemaMismatch { found: i32, expected: i32 } }
pub struct Reader { /* read-only Connection */ }
impl Reader {
    pub fn open(path: &Path) -> Result<Opened, StoreError>;
    pub fn sensors(&self) -> Result<Vec<SensorRow>, StoreError>;
    pub fn last_ts(&self) -> Result<Option<i64>, StoreError>;
    pub fn latest(&self) -> Result<Vec<(i64, Value)>, StoreError>;
    pub fn series(&self, row: i64, from: i64, to: i64) -> Result<Vec<(i64, f64)>, StoreError>;
}
```

**Reconciling G1 with G2.** G2's `Value::Throttled(bool)` is too narrow for what G1 found. It needs to carry these states:
- the 9900X's "At thermal limit (boost regulated)"
- the 9070 XT's "power-limited"

The proposal is a small state enum, `ThrottleState::{None, PowerLimited, AtLimit, Throttling}`, stored as 0..3. The exact shape is decided at build step 4, when the first real signal exists.

### Schema v1

```sql
sensor(id INTEGER PRIMARY KEY, key TEXT NOT NULL UNIQUE, label TEXT NOT NULL,
       device TEXT NOT NULL, kind INTEGER NOT NULL, source INTEGER)
reading(ts INTEGER, sensor INTEGER, value INTEGER, PRIMARY KEY (ts, sensor)) WITHOUT ROWID
reading_1m(ts INTEGER, sensor INTEGER, min INTEGER, avg INTEGER, max INTEGER,
           PRIMARY KEY (ts, sensor)) WITHOUT ROWID
```

- `value` holds centi-°C, a throttle state, or a pressure level. `kind` says which.
- Sensor upsert uses `ON CONFLICT(key) DO UPDATE ... RETURNING id`, which needs SQLite 3.35 or later. The bundled version is 3.54.1.

## G3: Test and CI

### Checked on this Mac

- The git remote is GitHub (`Omer-NeoAsh/thermals-rust`); visibility is unknown.
- Toolchain: stable 1.99.0 aarch64 only.
- **cargo-deny is not installed**, so the starter `deny.toml` has never run.
- **`sandbox-exec`** with `(deny network* (with send-signal SIGKILL))`:
  - a `connect()` kills the process (exit 137)
  - `AF_UNIX` sockets work
  - creating an `AF_INET` socket alone is not blocked
- **`clippy.toml` `disallowed-types`** flags `std::net::TcpStream`, and `-D warnings` turns that into an error. It catches only those types, not `libc::socket` or dependencies.

### Test layout

- **core:**
  - Linux parsers compile on every OS, so they are not cfg-gated. Only the `Native` alias is.
  - Device identity uses `fs::read_link(...).file_name()`, not `canonicalize`, because the fixture symlinks dangle.
  - `tests/linux_fixtures.rs` runs on the Mac as well.
  - `tests/hardware.rs` is `#[ignore]`, for Omer's machines, and checks loose invariants only.
- **store:**
  - `Connection::open_in_memory()` for schema, record, series, prune and migration tests.
  - `tempfile` (a dev-dependency) for the WAL and read-only tests.
- **collector:**
  - `lib.rs` plus a thin `main.rs`.
  - Tests call `tick` with FakeBackend and a scripted `now`.
  - `tests/e2e.rs` runs the binary via `env!("CARGO_BIN_EXE_neem-thermals")` (verified by probe; the test must live in the binary's package) with `--samples 3` and a temp DB.
- **tui:** render functions take `&State` and are tested with ratatui's `TestBackend`.

### Fixtures

Captured as a normal user with `scripts/capture-sysfs.sh OUT_DIR` (not yet run). The script:
- copies an allowlist of globs, with no `find` traversal
- skips files it can't read
- recreates the hwmon `device` symlinks as dangling links

Fixture directories:

| Directory | Contents |
|---|---|
| `ryzen9-9900x-rx9070xt/` | Real capture from Omer's box |
| `intel-synthetic/` | Hand-written, labelled synthetic |
| `hwmon-renumbered/` | Same chip and device, different `hwmonN`; expects the same ids |
| `edge-cases/` | Missing label, `_fault=1`, empty file, garbage value, negative value |

Before committing a fixture:
- `rg -i '([0-9a-f]{2}:){5}[0-9a-f]{2}|serial|uuid|hostname'` over the fixtures should print nothing.
- Run `strings` on `gpu_metrics` and look at the output.
- The PCI addresses in the symlinks are machine-specific but not personal.

### Runtime no-network test

`scripts/no-network-check.sh COMMAND...`:

| OS | How it checks | Fails when |
|---|---|---|
| Linux | `strace -f -e trace=socket,socketpair` | any socket other than `AF_UNIX` is created |
| macOS | `sandbox-exec` deny network with SIGKILL | the process connects, binds or sends over the network |

- CI runs it on the collector (`--features fake --samples 3`).
- It also runs a negative control that must fail: `! scripts/no-network-check.sh curl -s http://127.0.0.1:9`.
- Unverified: strace and ptrace on `ubuntu-latest`, and `sandbox-exec` on hosted macOS runners.

The systemd unit test does `include_str!` of the unit file and asserts the `RestrictAddressFamilies=AF_UNIX` line. It arrives with the service step.

### CI (GitHub Actions)

- **Matrix:** `ubuntu-latest` and `macos-latest`. Steps:
  - `cargo fmt --check`
  - `clippy --locked --workspace --all-targets --all-features -D warnings`
  - `test --locked --workspace --all-features`
  - install strace on Linux
  - build with `--features fake`
  - the no-network check and the negative control
- **Separate job:** `EmbarkStudios/cargo-deny-action@v2` on `ubuntu-latest` only, because it is a Docker action.
- **Verified in Wave 5:** `actions/checkout@v7` (v7.0.1), `cargo-deny-action@v2` (v2.1.3, cargo-deny 0.22.0), and `macos-latest` is macOS 26 arm64. strace is not preinstalled, so install it with `sudo apt-get update && sudo apt-get install -y strace`.
- **Repo housekeeping:** commit `Cargo.lock` and add `target/` to `.gitignore`.
- **Before the first push:** install cargo-deny locally and run it once.

### Lints and toolchain

```toml
[workspace.lints.rust]
unsafe_code = "deny"
rust_2018_idioms = { level = "warn", priority = -1 }

[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
dbg_macro = "warn"
todo = "warn"
undocumented_unsafe_blocks = "warn"
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
```

- `unsafe_code` is `deny`, not `forbid`, so the macOS FFI module can allow it.
- `clippy.toml` adds `disallowed-types` for `std::net::{TcpStream, TcpListener, UdpSocket}`, plus `allow-unwrap-in-tests = true` and `allow-expect-in-tests = true` (both verified by probe on 1.99).
- No `rust-toolchain.toml`, no `rust-version` and no `rustfmt.toml`.

## Inline gaps (skill)

| Category | Gap | Resolution |
|---|---|---|
| Accessibility | Throttle state shown by colour alone | Every state has a text label: "Throttling (OS reported)", "Likely throttling (inferred)", "At thermal limit", "Power-limited". Colour is extra. Respect `NO_COLOR`. Units always printed (°C). |
| Accessibility | Charts carry no readable values | The latest numeric value sits next to every chart. The table view is the default screen. |
| Accessibility | Keyboard help | One help line listing the keys; `?` shows it. No mouse-only actions. |
| Documentation | Nothing tells a user how to install, what is collected, where it is stored, or how to remove it | A README section per binary, added when each binary ships. A privacy section names the data path, the 0700/0600 modes, "no network" and how it is tested. Every claim is written from the code. |
| Documentation | Linux board sensors may need a root `modules-load.d` step | Documented as optional, never done by the app. |
| Security | The log file and the `VACUUM INTO` backup could be created world-readable | Both go inside the 0700 data dir, or a log dir created as 0700. `UMask=0077` in the systemd unit. In the plist, `Umask` set to `<integer>63</integer>`, verified in `man launchd.plist`. |
| Security | Fixtures could leak identifiers | The G3 scrub step must pass before a fixture is committed. |
| Security | `service install` writes the unit or plist with the user's exe path | Only in the user's own config dirs. It never runs `sudo`. |

## Questions for Omer (answered 2026-11-23 as T1-T6 in project-specs)

From G3:
1. Is the GitHub repo public or private? Private repos may bill macOS CI minutes at a higher rate (UNVERIFIED).
2. Should the collector get a `--samples N` flag and a `fake` feature? Recommended: yes.
3. What should the collector do when a machine has no sensors? Recommended: exit 0 with a warning.
4. Is it OK to commit the dangling `device` symlinks, which hold PCI addresses, in the AMD fixture? Recommended: yes.
5. Should a weekly scheduled `cargo deny` run be added? Recommended: no.

From G1:
6. What is the Linux box's kernel version, and what motherboard does it have? Running the capture script, or the G1 command list, on the box answers both.
