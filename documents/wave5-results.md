# Wave 5 Results: Verification

**Date:** 2026-11-23
**Setup:** `documents/wave5-setup.md`

The corrections below have already been applied to `wave4-results.md`, and the deny-key note to `wave1-results.md`.

## Verdicts

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| 0 | `user_version` rolls back with its transaction | VERIFIED (system SQLite) | `sqlite3 :memory:`: 0 after rollback, 7 after commit (inline) |
| V1.1 | Arch `linux` is ≥ 6.16 | VERIFIED: 7.3.5.arch1-1 (built 2026-11-17) | archlinux.org/packages/core/x86_64/linux/json/ |
| V1.2 | k10temp Zen 5 Tccd in 6.16; amdgpu `fan1_input` and `cpuinfo_avg_freq` in 6.15 | VERIFIED: first tags v6.16-rc1, v6.15-rc1, v6.15-rc1 | `gh api compare` on 2c183963, 90df6db6, fbb4a475 |
| V1.3 | Board Super I/O | REFUTED (was "nct6775 or it87, unknown"). ITE IT8696E (0x8696) plus IT87952E (0x8695). | frankcrawford/it87 PR #131 (WIFI7) and issue #89 (X3D): user reports in the maintainer repo. ICE variant: no source. |
| V1.4 | Mainline it87 supports the board | REFUTED: no IT8696E in mainline (it87.c lines 145-169). The out-of-tree driver has it (fc-it87.c line 280). Mainline might bind only the IT87952E (inferred from code, not run). | driver sources |
| V1.5 | Driver autoloads | REFUTED for both drivers. The mainline DMI table has one board. The out-of-tree table lacks "X870E AORUS ELITE" (PR #131 closed without merging). | it87.c 3975-3980; fc-it87.c 7398-7409 |
| V1.6 | Resource-conflict parameter needed | Out-of-tree: no (MMIO is the default, fc-it87.c line 322). Mainline: STILL UNVERIFIED. | as cited |
| V1.7 | `gpu_metrics_v1_3`: `throttle_status` @68, `indep_throttle_status` @112, size 120 | VERIFIED by compiling `_Static_assert`s (clang, x86_64-linux) | kgd_pp_interface.h 554-558, 897-967 |
| V1.8 | SMU 14.0.2/3 emits v1_3 | VERIFIED | amdgpu_smu.c 842-844; smu_v14_0_2_ppt.c 2264, 2330 |
| V1.9 | 9900X cpuinfo | VERIFIED: family 26, model 68 (0x44), `AMD Ryzen 9 9900X 12-Core Processor` | two independent real dumps on GitHub |
| V2.1 | `actions/checkout@v7`, `cargo-deny-action@v2` | VERIFIED: v7.0.1; v2.1.3 bundling cargo-deny 0.22.0. The deny action is a Docker action, so it runs on Linux only. | `gh api` releases; action.yml:52 |
| V2.2 | `macos-latest` is arm64 | VERIFIED: macOS 26 arm64 | runner-images README.md:32 |
| V2.3 | strace preinstalled on `ubuntu-latest` | REFUTED. The plan already installs it; use `apt-get update` first. | Ubuntu2404-Readme.md package list |
| V2.4 | ptrace of children on the runner | STILL UNVERIFIED. Yama mode 1 allows tracing descendants; the first CI run will show. | Yama.rst:58-61 |
| V2.5 | clippy `allow-unwrap-in-tests`, `allow-expect-in-tests` | VERIFIED by probe on 1.99 | scratch crate |
| V2.6 | `env!("CARGO_BIN_EXE_neem-thermals")` | VERIFIED by probe. The test must be in the binary's package. | scratch crate |
| V2.7 | env_logger | VERIFIED: 0.11.12, defaults `auto-color`, `humantime`, `regex`; stderr by default | env_logger-0.11.12/src/writer/target.rs:8-9 |
| V2.8 | plist `Umask` | VERIFIED: integer or string. Use decimal `63` for 077. | `man launchd.plist` line 302 |
| V2.9 | launchd expands `~` | STILL UNVERIFIED (not documented). Write absolute paths. | `man launchd.plist` |
| V2.10 | deny.toml syntax | VERIFIED from source (not executed): `{ crate = "x" }`, and every starter key parses in 0.22.0 | cargo-deny package_spec.rs:70 and the cfg files |

## Design changes from Wave 5

1. **Board sensors** on the X870E AORUS ELITE need the out-of-tree `frankcrawford/it87` driver and a `modules-load.d` entry. That is a root step Omer does, documented and never done by the app. Don't run `sensors-detect`. The app treats `it8696`/`it87952` hwmon devices as optional.
2. **The CI deny job is pinned to `ubuntu-latest`.**
3. **Kernel baseline:** every feature the design uses is present on Omer's box (Arch 7.3.5).

## Still unverified

Each of these is answered by doing the step, not by more research:
- ptrace and `sandbox-exec` on hosted runners: answered by the first CI run.
- The mainline it87 conflict behaviour on this board: answered by loading it on the box.
- launchd `~` expansion: not needed, because the installer writes absolute paths.
- The `user_version` rollback against the bundled SQLite: a store unit test repeats it.
