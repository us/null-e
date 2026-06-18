# null-e — Scan / Search / Clean / Delete & Permissions Fix Plan

## Context

null-e is a Tauri v2 macOS disk cleaner: Rust library `null-e-core` + a CLI (`null-e-cli`)
+ the Tauri GUI (`null-e-gui` under `tauri/`) with a React/Zustand UI under `ui/`.
The user reports four failing pillars: **scan quality**, the **clean flow**, **search**, and
**delete reliability** — concretely: "~59 GB shows in System/cache but cannot be deleted,
permissions are granted yet deletes throw nonsensical errors, some files never delete," and
the in-app **guide is hard to follow**.

This plan fixes root causes on three fronts: **algorithm** (scan, size, search, delete),
**permissions** (macOS signing / entitlements / FDA / privileged paths), and **UX** (guide,
result trust, failure reporting). It builds on what already ships — much of the UI the first
draft proposed to "build" already exists and just needs to be *improved*, not rebuilt.

### Verified facts (code + parallel research + Iteration-2 review)

**Source tree (corrected — NOT a mirror)**
- `cargo metadata --no-deps` reports exactly three packages with targets:
  `null-e-core` (`crates/null-e-core/src/lib.rs`), `null-e-cli` (`crates/null-e-cli/src/main.rs`),
  `null-e-gui` (`tauri/src/lib.rs` + `main.rs`). **The top-level `src/` is referenced by no
  build target** — root `Cargo.toml` is `[workspace]`-only.
- Top-level `src/` is the **orphaned legacy single-binary tree**: it contains `main.rs` (76 KB),
  `lib.rs` (declares `pub mod tui;`), and a `tui/` module that do **not** exist in `null-e-core`.
  It is *not* a byte-identical mirror; it is dead code superseded by core + cli.
- The current WIP edits touch **both** `src/cleaners/system.rs`/`src/trash/mod.rs` **and** the
  `null-e-core` copies; the `src/` edits are wasted duplicates against dead code. The
  `null-e-core` edits are the ones that matter.

**Distribution decision (user-confirmed)**
- **No Apple Developer ID.** Releases are an **unsigned `.dmg` on GitHub** (no paid Developer
  Program, no notarization). This is a hard constraint that reshapes Phase 4 (below) — we are NOT
  doing Developer ID signing or notarization CI.
- **Two consequences flow directly from this** and both feed the user's "permissions granted yet
  it fails" complaint, so they must be designed for, not ignored:
  1. **Gatekeeper friction at install.** An unsigned, non-notarized app from the internet is
     quarantined. On Sequoia the Control-click→Open override was removed; the user must go to
     **System Settings → Privacy & Security → "Open Anyway"** (or `xattr -dr com.apple.quarantine`).
     The README + first-launch guide must cover this explicitly.
  2. **FDA is lost on every update.** A pure ad-hoc/unsigned build has no stable Designated
     Requirement, so TCC falls back to **cdhash** binding, which changes with every new build →
     **the Full Disk Access grant evaporates on each release.** This *is* a major driver of "I
     granted access but it still fails." **Mitigation (recommended, $0):** sign every release with
     a **stable self-signed certificate** (created once, reused for all builds). That gives a
     *stable DR* (`identifier "<bundle id>" and certificate leaf = H"<cert hash>"`) so the FDA
     grant persists across updates. Gatekeeper still rejects it (not notarized → "Open Anyway"
     still needed once), but TCC persistence is the win. *(To validate on a real machine — this is
     the single highest-leverage permissions fix available without a Developer ID.)*

**Permissions / entitlements**
- `tauri/tauri.conf.json:51` sets `entitlements: "../Entitlements.plist"`. Tauri resolves bundle
  paths relative to `tauri.conf.json` (i.e. `tauri/`), so `../Entitlements.plist` points at the
  **repo root, where no file exists**. The real plist lives at **`tauri/Entitlements.plist`** —
  so the bug is a **path mismatch**, not an absent file. (First draft's "file does not exist" was
  wrong.) Correct reference is `"Entitlements.plist"`.
- The existing `tauri/Entitlements.plist` has `app-sandbox=false`,
  `files.user-selected.read-write`, `network.client` — and **no hardened-runtime keys**. Note:
  hardened-runtime entitlements (the JIT pair) only take effect under a hardened-runtime sign;
  for ad-hoc/self-signed they are harmless to include and become meaningful if the self-signed
  sign uses `-o runtime`. Keep them.
- TCC binds Full Disk Access to the app's **Designated Requirement** when signed (stable across
  rebuilds *iff* identifier + signing cert are constant — see the self-signed mitigation above),
  and to the **cdhash** when ad-hoc/unsigned (changes every build → grant lost). **Notarization is
  for Gatekeeper, not TCC.** Local dev builds will keep losing FDA regardless — expected.
  *(claude-code#55661; eclecticlight TCC)*
- A general cleaner must be **non-sandboxed** (Developer ID, not App Store). Don't enable
  `app-sandbox`. The hardened-runtime exceptions a WebView needs are the **JIT pair**
  (`com.apple.security.cs.allow-jit`, `…allow-unsigned-executable-memory`) — Tauri does **not**
  auto-inject these. `…disable-library-validation` is **broader and a known injection primitive**;
  it is only justified if a sidecar/external binary is bundled (this app bundles none), so it
  should be **omitted by default**. *(tauri#11992; HackTricks dangerous-entitlements)*
- Tauri v2 capabilities/ACL do **NOT** gate raw `std::fs` in custom `#[tauri::command]` fns —
  only the JS→Rust bridge and official plugins. Current `capabilities/default.json` is sufficient.
- FDA detection (`tauri/src/commands/system.rs`) probes `~/Library/Application Support/com.apple.TCC`
  readability — false negatives. A read-probe of a **genuinely TCC-protected** dir (`~/Library/Safari`,
  fallback `~/Library/Mail`) is better — but **`~/Library/Caches` is user-readable without FDA**
  and would false-*positive*. A FDA denial returns **EPERM(1)** specifically; treat ENOENT as
  inconclusive. TCC decisions are **cached per process**, so the focus-refresh in `useFdaCheck.ts`
  cannot flip state after a grant — an explicit **relaunch** is required (soften copy to "relaunch
  if status/deletion still fails"). The System-Settings deep link (`Privacy_AllFiles`) is dropped
  when launched via `/usr/bin/open` (Tauri shell `open`); it works via **`NSWorkspace.open`**.
  *(Apple Forums 114452/709289)*

**Scan / size accuracy**
- `calculate_dir_size` (core `cleaners/mod.rs`) and trash `calculate_size` sum
  `metadata().len()` — **apparent size (`st_size`), not on-disk allocation (`st_blocks*512`)**.
  On APFS (compression, sparse) these diverge; switching to blocks fixes the compression/sparse
  lie. *(man 2 stat)*
- **`st_blocks*512` does NOT predict freed space** for: (a) **APFS clones** — distinct inodes,
  `nlink==1`, sharing extents; each clone reports full blocks, deleting one frees ~0; (b) blocks
  **pinned by snapshots/COW**; (c) **purgeable** space. So `(dev,ino)+nlink>1` dedup handles
  **only hardlinks**, not clones (clones are `nlink==1`). The authoritative freed number is the
  **`statfs` `f_bavail` delta** — but that delta is **racy** (concurrent OS writes, lazy/async
  purgeable reclaim, container-shared free space can make it small, zero, or **negative**).
- **"System Data"/purgeable illusion**: `df -k /` (`commands/system.rs`) over-reports; the prime
  "59 GB shown, nothing frees" driver is purgeable + local snapshots, which are largely
  OS-managed. The single **user-reclaimable** chunk of that is **local Time Machine snapshots**
  (`tmutil thinlocalsnapshots`). The current Time Machine cleaner estimates a hard-coded **2 GB
  each** and its "delete" runs **`tmutil deletelocalsnapshots /`, which is not a valid command**
  (no-op/error).
- `jwalk` is a declared dep but the scanner uses **walkdir**. The **scanner already skip-logs**
  walk errors (`scanner/parallel.rs:97` does `continue`). The real blind spots are:
  `detect_cleaners` does `filter_map(|e| e.ok())` (`cleaners/mod.rs:187`) — a cleaner whose
  `detect()` errors is **silently dropped**; and `calculate_dir_size` swallows per-entry errors
  without surfacing them. So "aborts on first `?`" was misattributed.

**Delete reliability**
- On `trash::delete`/`remove_dir_all` success, `delete_path` (`trash/mod.rs:52`) returns the
  **pre-delete size** as "freed"; `start_clean` (`commands/clean.rs:46`) sums these → headline
  "freed" is fiction. For **Trash mode it is doubly wrong** (trashing frees **0 bytes** until the
  Trash is emptied). The best-effort fallback also sums `len()` and **swallows per-file errno**.
- **`clean_cache` bypasses `start_clean` entirely** (`commands/cache.rs:20` → core
  `caches/mod.rs:449`) and returns **`bytes_freed: size_before`** (a pre-clean estimate). Any
  freed-bytes fix must cover this path too, or cache cleans keep reporting fiction.
- Error classification (`commands/clean.rs`) treats `raw_os_error()==1` (EPERM) as TCC, but EPERM
  also = BSD immutable (`uchg`/`schg`); **EACCES(13) and EPERM(1) are BOTH used for TCC denials
  and for ownership** — errno alone cannot disambiguate **NeedsFDA vs NeedsAdmin** (must check
  `lstat` uid vs `geteuid()` + parent-dir writability). `chflags(path,0)` would **clobber all
  flags** (must read `st_flags` and clear only `UF_IMMUTABLE|UF_APPEND` if owner). `SF_RESTRICTED`
  (SIP) **cannot be cleared even by root** → terminal; `SF_IMMUTABLE` (schg) needs root → admin.
  EROFS(30) on a delete target means the **scanner listed a sealed-volume path** (scan-side bug).
  ENOENT(2) = already gone → **success (idempotent)**.
- `cancel_clean` is **structurally dead**: `start_clean` never writes `state.clean_progress`
  (stays `None`) and the loop captures `_state` (unused), so `cancel_clean`'s
  `if let Some(progress) = …` is always `None`. Cancellation is impossible, not merely ignored.
- `remove_dir_all` (Permanent mode) does **not** follow a top-level dir symlink and uses
  `openat`/`unlinkat` nofollow internally on recent std — it is the *safer* path. The
  **best-effort fallback walker is the weaker path** (path-based `remove_file`/`remove_dir`,
  classic TOCTOU). `O_NOFOLLOW`/`*at` are not reachable via `std::fs`/`trash-rs`.
- No privileged helper: **root-owned paths** (`/Library/Caches`, `/private/var`) can't be removed
  by the user process → permanent EACCES. *(Pearcleaner; GHSA-gr2j-65fh-8pvc: XPC must validate
  the caller by **audit token**, never PID — PID-reuse TOCTOU.)*

**Existing UI (don't rebuild — improve)**
- Search **already exists** end-to-end: `SearchBar.tsx` + `ViewToolbar.tsx` + filtering over
  name/path/category/kind in `ResultsView.tsx:219` with `searchQuery` in store, an "N of M"
  indicator and an empty state. Deficit is *quality* (no fuzzy, no facets), not absence.
- Failure handling **already exists**: `CelebrationView.tsx` renders a TCC banner + "Open
  Settings", a collapsible failure list, `is_tcc` flag, and "Completed with errors" vs "Space
  reclaimed" states; DTOs carry `CleanFailureDto{path,reason,is_tcc}` + `CleanSummaryDto`.
- Results are **already grouped** (`GroupedList`, `SystemSection`), with `SelectionPresets` and a
  `ConfirmDialog` naming count + size + trash/permanent toggle (defaults `useTrash=true`).
- The UI computes `selectedSize`/`totalCleanable`/`maxSize` by **client-side summing
  `artifact.size`** (`ResultsView.tsx:285-425`) — so any backend dedup/allocated-bytes change
  must be reflected **per item** or the UI sums will disagree with the backend total (re-creating
  "numbers don't add up").

### What we are NOT doing (scope guard)
- Not touching Windows/Linux behavior beyond keeping it compiling.
- Not building a Mac App Store sandboxed variant.
- **Not** doing Developer ID signing or notarization (no Apple Developer Program — unsigned `.dmg`
  on GitHub). We use a stable **self-signed** cert only to persist the FDA grant, and document the
  Gatekeeper "Open Anyway" flow.
- **Not** auto-elevating to delete root-owned/system paths in v1 (no signed XPC helper without a
  Developer ID) — those are classified `NeedsAdmin` and explained.
- Not faking reclaim of SIP-protected or truly OS-managed purgeable space — we **explain** it, but
  we **do** offer the one genuinely user-reclaimable slice (local Time Machine snapshots).
- Not adding cloud/telemetry. (A **local** failure log is in scope; cloud is not.)
- Not adding i18n/a11y phases (outside the user's four complaints).

---

## Approach

1. **Tell the truth about space.** Switch sizing to on-disk blocks (fixes compression/sparse),
   dedup hardlinks, and **stop claiming blocks predict freed space** for clones/snapshots/purgeable.
   Replace `df` with `statfs` for capacity, and make the **post-delete `statfs` delta (clamped,
   labeled estimate)** the freed figure — backed by a per-file block-sum captured immediately
   before each unlink as the primary, stable predictor.
2. **Fix the freed-bytes fiction first.** The most visible "59 GB won't delete" symptom is
   `delete_path` returning pre-delete bytes and `clean_cache` returning `size_before`. Fix result
   accounting across **all three** delete paths (fast, fallback, cache) before anything else.
3. **Make permissions actually work (unsigned-distribution reality).** Fix the entitlements path +
   add the JIT pair (no `disable-library-validation`); since there's no Developer ID, use a stable
   **self-signed** cert to persist the FDA grant across updates (validated on-device), document the
   Gatekeeper "Open Anyway" install flow, and fix FDA detection to probe a truly-protected path
   with a relaunch step (and a re-grant-after-update path as the honest floor).
4. **Make deletion honest and robust.** Per-item results with a correct errno + flags taxonomy
   (`lstat` flags before `chflags`; ownership check for NeedsFDA vs NeedsAdmin; SIP terminal),
   correct Trash accounting, cancellation that actually works, and a privileged path that is
   either deferred or a properly-hardened XPC helper.
5. **Make scan + search trustworthy.** Surface dropped cleaners + per-entry errors (the real
   blind spot), tune thresholds with small-item roll-ups, dedup nested artifacts, and **upgrade**
   the existing search to fuzzy + facets (safety / reclaimability / failure) — not rebuild it.
6. **Rebuild the guide & result trust.** A short, **state-aware, self-relaunching** FDA wizard;
   results grouped by **reclaimability** with an honest "X GB is OS-managed" explainer **and** a
   one-click local-snapshot reclaim; a Trash flow that never shows "0 bytes freed" confetti.
7. **Delete the orphan tree** so fixes land once (cheap, non-blocking).

### Trade-offs accepted
- We display **on-disk allocated size** ("size on disk — may be held by snapshots/clones") and the
  freed number is an **estimate** (clamped `statfs` delta cross-checked with deleted-block-sum).
- Privileged helper adds packaging/security complexity → gated behind opt-in "Advanced / system
  caches"; FDA-only flows ship working first.
- Exact reclaimable bytes for snapshots/clones is **fundamentally unknowable** cheaply → labels &
  ranges, not fake precision.

---

## Phases

> Ordering note: the cheap unblockers (entitlements path, orphan tree) and the **freed-bytes
> accounting truth** come first because they directly address the headline complaint and unblock
> dev builds. Sizing model precedes the full deletion engine; signing (cert-gated) and the
> privileged helper come later.

### Phase 0 — Cheap unblockers (prereq, ~0.5d, Low risk)
- **0a Entitlements path:** change `tauri/tauri.conf.json` `entitlements` from `"../Entitlements.plist"`
  to **`"Entitlements.plist"`** (the file that exists at `tauri/`). Add the JIT pair
  (`com.apple.security.cs.allow-jit`, `…allow-unsigned-executable-memory`); **remove** the
  `app-sandbox` key entirely (non-sandboxed is the default; the explicit `false` is noise) and do
  **not** add `disable-library-validation` (no sidecar). This unblocks unsigned dev builds and
  preps for signing.
- **0b Orphan tree:** confirm via `cargo metadata --no-deps` that no target uses top-level `src/`
  (verified), then `git rm -r src/`. **No WIP re-apply needed** — the `null-e-core` copies already
  carry the same edits; the `src/` edits are duplicates against dead code. (Quick-diff `src/tui/`
  vs `crates/null-e-cli/src/tui/` first to confirm nothing unique is lost; expected: cli is newer.)
- **Verification:** `cargo build` (all three crates) + `cargo metadata` clean; unsigned dev
  `tauri build` succeeds; app launches and WebView renders with the new entitlements.

### Phase 1 — Freed-bytes accounting truth (algorithm) — HIGH PRIORITY
- **Files:** `crates/null-e-core/src/trash/mod.rs` (`delete_path`, `calculate_size`, best-effort
  fallback), `crates/null-e-core/src/caches/mod.rs` (`clean_cache`), `tauri/src/commands/clean.rs`
  (`start_clean`), `tauri/src/commands/cache.rs` (`clean_cache`), `dto.rs`.
- **Steps:**
  1. Introduce a single **per-item result type** `{path, requested_method, outcome, errno_class,
     bytes_attributed, bytes_actually_freed}` used by **all three** delete paths (fast,
     best-effort fallback, **and** `clean_cache`) so accounting cannot diverge.
  2. Primary freed metric: sum `st_blocks*512` of each item captured by `lstat` **immediately
     before** its unlink (stable under concurrency, with the clone caveat noted to the user).
  3. Cross-check with a **`statfs` `f_bavail` delta** around the whole batch, **clamped to
     `max(0, after-before)`** and **labeled an estimate**; never report a raw/negative delta.
  4. **Trash mode reports `freed = 0` now + a "pending until Trash emptied: X" figure** —
     including the fallback path (which currently sums `len()` even when trashing).
  5. **`clean_cache`** must stop returning `size_before`; route through the same accounting (or
     explicitly label official-command/cache bytes "estimated").
- **Verification:** unit test — Trash mode returns `freed==0`, pending>0; integration — after a
  permanent delete, reported freed ≈ clamped `statfs` delta (±tolerance) and ≈ deleted-block-sum.
- **Effort:** 1.5 days · **Risk:** Medium.

### Phase 2 — Honest disk/space measurement (algorithm)
- **Files:** ALL sizing sites that currently sum `metadata().len()` —
  `crates/null-e-core/src/cleaners/mod.rs` (`calculate_dir_size`),
  `crates/null-e-core/src/trash/mod.rs` (`calculate_size`),
  **`crates/null-e-core/src/plugins/mod.rs` (`default_calculate_size:75` + the `calculate_size`
  trait default at :49)**, **`crates/null-e-core/src/caches/mod.rs` (`calculate_cache_size:407`)** —
  plus `tauri/src/commands/system.rs` (`get_disk_info`),
  `crates/null-e-core/src/cleaners/system.rs` (Time Machine), `dto.rs`,
  UI `DiskBar`/`ImpactCards` + **`ResultsView.tsx` sum sites (285-425)**.
- **Steps:**
  1. `disk_usage(path) -> (apparent, allocated)` via `MetadataExt::blocks()*512` on Unix; fall
     back to `len()` elsewhere. Route **every** sizing site above through it (project artifacts via
     `default_calculate_size`, package caches via `calculate_cache_size`, system via
     `calculate_dir_size`) — otherwise project/cache totals keep showing apparent size while disk
     totals show allocated, and the UI numbers stay inconsistent. Report **allocated** as "size on
     disk (may be held by snapshots/clones)" — **not** as "predicts freed."
  2. **Hardlink dedup** via `HashSet<(dev,ino)>` (use `nlink>1` only as a cheap filter for whether
     to insert). **Do not** attempt per-file clone accounting — mark clone-suspect subtrees and
     defer their true reclaim to the Phase-1 `statfs` delta.
  3. Replace `df -k /` with `statfs(2)` (`libc`/`nix`), keeping the fields distinct: **capacity =
     `f_blocks*f_bsize`**, **available-to-user free = `f_bavail*f_bsize`** (the honest "free"),
     raw free = `f_bfree*f_bsize`. Do not conflate capacity with free.
  4. Time Machine: enumerate real snapshots (`tmutil listlocalsnapshots /` — adjust header
     parsing vs the old `listlocalsnapshotdates`), drop the 2 GB constant, label "purgeable —
     reclaim varies," and replace the **invalid** `tmutil deletelocalsnapshots /` action with
     **`tmutil thinlocalsnapshots / <bytes> <urgency>`** behind explicit consent (see Phase 6).
     Note: `run_clean_command` (`caches/mod.rs:494`) parses `clean_command` via naive
     `split_whitespace` — pass the snapshot args **structurally** (or interpolate bytes/urgency
     safely), don't rely on whitespace-splitting a formatted string.
  5. **Reclaimability classification** on every item: `UserReclaimable`, `NeedsAdmin`,
     `OsManagedPurgeable`, `SipProtected` — and a **DTO contract** so the UI's client-side sums
     stay consistent: each item carries `allocated_bytes` + `reclaimable_bytes` (shared/clone or
     purgeable items get `reclaimable_bytes` reduced/0 with a `shared`/`os_managed` marker). Update
     `ResultsView` summation to sum `reclaimable_bytes`.
- **Verification:** unit tests on a fixture with a sparse file + a `clonefile` clone — assert
  allocated ≈ `du` (blocks) and hardlink dedup prevents double count; clone reclaim validated via
  before/after `statfs` in a macOS integration/manual test (not brittle `du -A` assertions).
- **Effort:** 2 days · **Risk:** Medium.

### Phase 3 — Robust deletion engine (algorithm + permissions)
- **Files:** `crates/null-e-core/src/trash/mod.rs`, `tauri/src/commands/clean.rs`, `dto.rs`,
  and for the action model: `CleanableItemDto`/`SystemEntry` DTOs + `ui/src/components/ResultsView.tsx`
  (`handleClean`).
- **Steps:**
  1. Precise errno + flags taxonomy (extends Phase-1 result type):
     - On **EPERM/EACCES**: `lstat` → if `st_flags` has `UF_IMMUTABLE`/`UF_APPEND` **and we own
       it** → clear *only* those bits via `chflags` (read-modify-write, never `chflags(0)`), retry
       once. If `SF_RESTRICTED` → **`SipProtected` (terminal, never retry/elevate)**. If
       `SF_IMMUTABLE` (schg) → **`NeedsAdmin`**. No immutable flags → disambiguate **NeedsFDA vs
       NeedsAdmin** by `lstat` uid vs `geteuid()` + parent-dir writability (do **not** route all
       EACCES to admin — admin can't fix TCC).
     - **EROFS(30)** → `SystemVolume`: this means the **scanner listed a sealed-volume path** →
       also fix Phase 5 to never enumerate it.
     - **ENOENT(2)** → **Success (idempotent)**. **ENOTEMPTY(66)** → re-walk, **bounded** retries
       (a daemon repopulating a cache can loop forever).
  2. **Typed system action model (safety-critical):** system cleaner items are **not all
     path-delete targets**, but `handleClean` (`ResultsView.tsx:463`) drops each item's
     `clean_command` and merges system paths into a generic `start_clean` path delete — so a Time
     Machine snapshot, the Trash, or an aggregate location (`/`, `~/.Trash`, `Downloads`) gets
     routed to a raw recursive delete (either fails → "won't delete," or is dangerous). Introduce
     an explicit action enum, but make execution **backend-owned — never trust frontend argv**.
     The UI sends `{item_id, action_id}` only; the backend re-resolves the item against its own
     latest scan result + a **fixed action registry** (`PathDelete | EmptyTrash |
     ThinTimeMachineSnapshots | RunCommand{command_id, params}`), validates against a fixed
     allow-list, and **constructs argv server-side**. Hard-refuse generic delete of `/`, `~`,
     `~/.Trash`, and known aggregate roots (allow-list of safe leaf paths only). Snapshot/Trash
     items dispatch to their typed backend action, not to `start_clean` as a path.
  3. **Cancellation rewire:** `start_clean` constructs an `Arc<CleanProgress>`/`AtomicBool`,
     stores it in `state.clean_progress` **before** spawning, moves a clone into the loop (which
     currently captures unused `_state`), checks `is_cancelled()` each iteration, clears the slot
     on completion, and guards against overlapping cleans.
  4. Symlink/TOCTOU: keep `remove_dir_all` for Permanent (already nofollow). The **best-effort
     fallback is the weaker path** — for user-owned trees, accept path-based unlink with a
     documented residual TOCTOU note; for the privileged/system path (step 5) implement an
     **fd-relative** deleter (`openat`+`unlinkat` w/ `O_NOFOLLOW|O_DIRECTORY`).
  5. **Privileged path — DEFERRED in v1 (decision: no Developer ID).** An SMAppService XPC helper
     realistically **requires a signed/notarized bundle**, which we don't have, so it is **out of
     scope for v1**. Instead: **classify `NeedsAdmin` and surface it honestly** ("X items are
     owned by the system and need admin rights; null-e doesn't auto-elevate"). Do **not** ship
     `osascript "with administrator privileges"` (whole batch runs as root, ~5-min auth caching,
     shell-string injection — especially bad on an unsigned app that already holds FDA). Root-owned
     system caches (`/Library/Caches`, `/private/var`) are a small slice of the "59 GB" vs
     purgeable/snapshots; deferring is an acceptable v1 trade-off. (If a Developer ID is obtained
     later: build the audit-token-validated SMAppService XPC helper — validate every connection by
     `SecCodeCreateWithAuditToken` + pinned `SecRequirement`, never by PID; explicit path
     allow-list, `realpath`-canonicalized, `O_NOFOLLOW`. Documented here so the door stays open.)
- **Verification:** unit tests for errno/flags mapping + `chflags` read-modify-write on a `uchg`
  fixture and a `schg`→`NeedsAdmin` fixture; cancellation integration test (cancel mid-batch stops
  the loop); Trash mode reports 0 freed; `NeedsAdmin` items surface in the failure summary (not a
  raw error).
- **Effort:** 2 days (helper deferred) · **Risk:** Medium.

### Phase 4 — Unsigned distribution, FDA persistence & detection (permissions)
> Reshaped by the no-Developer-ID decision: **no notarization, no Developer ID signing CI.** The
> goals here are (a) make the unsigned DMG installable with clear guidance, (b) make the FDA grant
> *persist across updates* via a self-signed cert, and (c) detect FDA correctly.
- **Files:** `tauri/Entitlements.plist`, `tauri/tauri.conf.json`, `.github/workflows/*release*`
  (self-signed sign step), `README` / install docs, `tauri/src/commands/system.rs` (FDA detect +
  NSWorkspace deep link), `tauri/Cargo.toml` (add `objc2`/`cocoa` for NSWorkspace — not a dep yet),
  `ui/src/hooks/useFdaCheck.ts`, `WelcomeView.tsx`.
- **Steps:**
  1. **Stable self-signed signing (FDA persistence, $0).** Create a self-signed code-signing cert
     once; store it as a CI secret; sign every release with the **same** cert + `-o runtime` so the
     TCC Designated Requirement (`identifier "<bundle id>" and certificate leaf = H"<hash>"`) is
     **stable across updates** → the FDA grant survives upgrades. Lock the bundle `identifier`.
     Keep `TAURI_SIGNING_*` (updater/minisign — unrelated). *Validate the persistence claim on a
     real machine before relying on it; if it doesn't hold, fall back to pure-unsigned + the
     re-grant-on-update UX (step 4 below) as the honest floor.*
  2. **Gatekeeper install guidance.** Since the app is not notarized, document the **"Open
     Anyway"** flow (System Settings → Privacy & Security) in the README and the first-launch
     guide; offer the `xattr -dr com.apple.quarantine /Applications/null-e.app` one-liner as the
     power-user path. This is a first-run UX surface, not a code change.
  3. **FDA detection:** read-probe `~/Library/Safari` (fallback `~/Library/Mail`) — **not**
     `~/Library/Caches` (false positive). Classify: success→`granted`; **EPERM(1)**→`not_granted`;
     ENOENT→try next then `unknown`. Don't rely on focus-refresh to flip state (per-process TCC
     cache) — require relaunch.
  4. **Re-grant-after-update handling.** If FDA persistence (step 1) can't be guaranteed, detect a
     *lost* grant on launch (was-granted → now-denied) and route straight into the FDA wizard
     (Phase 6) with copy that explains "an update reset this permission." This is the honest floor.
  5. Open System Settings via **`NSWorkspace.open`** (small `objc2`/`cocoa` call in a Rust command),
     **not** Tauri shell `open` (which drops the `Privacy_AllFiles` anchor); always render
     copy-paste manual steps as a guaranteed fallback.
- **Verification:** `codesign --verify` succeeds with the self-signed cert; `codesign -d
  --entitlements -` shows the JIT keys; install the DMG on a **second Mac**, confirm the "Open
  Anyway" flow works; **grant FDA on vN, install vN+1, confirm the grant persists** (self-signed
  DR) — the actual regression this targets; if it doesn't persist, confirm the re-grant wizard
  fires cleanly.
- **Effort:** 1.5–2 days · **Risk:** Medium (self-signed TCC persistence needs real-machine
  validation; no notarization complexity).

### Phase 5 — Scan quality & search (algorithm + UX)
- **Files:** `crates/null-e-core/src/scanner/parallel.rs`, `tauri/src/commands/cleaners.rs`
  (`cleaner.detect()` call sites, lines 15 & 41), `crates/null-e-core/src/cleaners/mod.rs`
  (`calculate_dir_size`), `cleaners/*.rs`, UI `shared/SearchBar.tsx`, `ResultsView.tsx`, `stores`.
- **Steps:**
  1. **Surface dropped cleaners + entry errors** (the real blind spot): the
     `if let Ok(detected) = cleaner.detect()` sites in `tauri/src/commands/cleaners.rs:15,41`
     silently swallow a cleaner whose `detect()` errors — record failed detections into a
     diagnostics field on the cleaner/system DTO instead of dropping; `calculate_dir_size`
     (`cleaners/mod.rs`) should add unreadable entries to `ScanProgress::add_error`. (The scanner
     already `add_error`s + `continue`s on walk errors.)
  2. If switching the scanner to `jwalk` for parallelism: prune excluded subtrees
     (`node_modules`, `target`) inside jwalk's **`process_read_dir`**, *not* a downstream filter
     (a naive filter descends and double-counts). **Keep `walkdir` in the deleter** (it relies on
     `contents_first` bottom-up order jwalk doesn't guarantee). If the perf win is marginal,
     keeping walkdir is acceptable — the bug is error-surfacing, not the walker.
  3. Re-tune min-size thresholds: aggregate small caches into a "+N small items (X MB)" roll-up.
  4. Dedup nested artifact dirs in results (a project and its child caches shouldn't double-list);
     ensure the sealed system volume / firmlink shadows are **never enumerated** (prevents EROFS
     at delete time).
  5. **Upgrade existing search** (don't rebuild): add debounced **fuzzy** match on top of the
     current substring filter (`ResultsView.tsx:219`) and **facet filters** (category, safety
     level, reclaimability, failed-only). Wire facets to the existing `searchQuery`/store.
- **Verification:** scan a seeded home dir → a `chmod 000` dir doesn't abort and **appears in
  diagnostics**; a cleaner whose detect errors is **reported, not vanished**; fuzzy search returns
  expected subset; totals match Phase 2 dedup.
- **Effort:** 1.5 days · **Risk:** Medium.

### Phase 6 — Guide & result UX (UX)
- **Files:** new `components/onboarding/*` (wizard), `WelcomeView.tsx`, `useFdaCheck.ts`,
  `CelebrationView.tsx` (extend, don't replace), `ResultsView.tsx`, `ConfirmDialog`, `SystemSection.tsx`.
- **Steps:**
  1. **State-aware FDA wizard** replacing the dense welcome/banner. Handle the full matrix:
     `granted` (skip), `not_listed` (first run — "click +, add null-e.app" / drag pattern),
     `denied` (re-enable instructions), `unknown` (**non-blocking** — let them scan anyway).
     Include a real **self-relaunch** action as the final step, since TCC is read per-process —
     reuse the existing JS `relaunch()` from `@tauri-apps/plugin-process` (already wired in
     `SettingsDrawer.tsx`/`useUpdateCheck.ts`), not a new Rust command. Skippable; returning granted users go straight to scan.
  2. **Results grouped by reclaimability** with safety badges + on-disk size, and an honest "X GB
     is OS-managed/purgeable — no app can remove it" explainer for the 59 GB case — **plus** a
     gated, one-click **local-snapshot reclaim** ("Frees space by removing local Time Machine
     snapshots; your external Time Machine backups are unaffected" → `tmutil thinlocalsnapshots`).
     This is the difference between the user *feeling* the fix vs being told "nothing we can do."
  3. **Trash flow honesty (decision: Trash default, permanent optional).** Keep the existing
     `ConfirmDialog` `useTrash=true` default **and** keep the user-facing toggle so they can pick
     **"Delete permanently"** when they want space immediately. The Trash-mode success screen must
     say **"Moved N items to Trash — empty Trash to reclaim X GB"** with a one-click **"Empty Trash
     now,"** never "0 bytes freed" confetti. Permanent mode shows the real freed delta.
  4. **Partial-failure summary** (extend the existing `CelebrationView` TCC banner): aggregate by
     cause → "X items need Full Disk Access" (→ wizard), "Y items need admin" (**explained — v1
     does not auto-elevate**, per Phase 3.5), "Z are OS-protected" (terminal, explained). Keep
     successes visible.
  5. **Local failure log** (not cloud): persist `{path, raw errno, classification, method}` for
     failed deletes, viewable/exportable from the failure summary — the safety net for the long
     tail of "some files never delete / nonsensical errors."
- **Verification:** manual walkthrough without FDA → wizard → grant → **self-relaunch** →
  successful clean; failure grouping with seeded root-owned + `uchg` + SIP items; Trash success
  shows pending + "Empty Trash"; snapshot reclaim frees space (before/after `statfs`).
- **Effort:** 2.5 days · **Risk:** Medium.

---

## Verification (overall)
- `cargo test -p null-e-core` (accounting, sizing, errno/flags, dedup) + `cargo clippy` +
  `cargo build` for all three crates.
- `cd ui && bun run build` + typecheck; component checks for wizard + fuzzy/facet search.
- Manual macOS matrix: {unsigned dev, self-signed release} × {FDA granted/denied/not-listed/unknown}
  × {trash, permanent, dry-run}, incl. root-owned cache (→ `NeedsAdmin` surfaced, not auto-deleted),
  `uchg`, `schg`, and a SIP path.
- Sanity: after a permanent clean, reported freed ≈ clamped `statfs` delta ≈ deleted-block-sum;
  Trash reports 0 + pending; `clean_cache` no longer returns `size_before`.
- FDA persistence: grant on vN → install vN+1 → grant survives.

## Open questions
All previously-blocking questions are now resolved (below). One item needs real-machine validation
during Phase 4, not a decision:
1. **Self-signed TCC persistence (validate, don't decide):** confirm on a real Mac that signing
   releases with a stable self-signed cert actually persists the FDA grant across a version bump.
   If it does → that's the FDA fix. If it doesn't → fall back to pure-unsigned + the
   re-grant-on-update wizard (Phase 4.4) as the honest floor. Either way the plan proceeds.
2. **Snapshot reclaim default (minor):** ship the one-click `tmutil thinlocalsnapshots` opt-in
   (planned), or instructions-only? Recommend the gated one-click (it's the one real lever on the
   "59 GB").

**Resolved (user decisions, 2026-06-13):**
- **No Developer ID — unsigned `.dmg` on GitHub.** → No notarization / Developer-ID CI. Phase 4
  reshaped to self-signed-for-FDA-persistence + Gatekeeper "Open Anyway" guidance.
- **Default = Trash, with a user-selectable "Delete permanently" option.** → Phase 6.3 keeps the
  `useTrash=true` default + toggle; Trash success shows "empty to reclaim X" + one-click Empty Trash.
- **Privileged helper deferred in v1** (follows from no signing): `NeedsAdmin` is classified and
  explained, no `osascript` admin. XPC helper documented for a future signed build.

*(Resolved since Iteration 0: authoritative tree = `null-e-core`+`null-e-cli`+`null-e-gui`;
top-level `src/` is a deletable orphan (`cargo metadata`). Entitlements file exists at
`tauri/Entitlements.plist`; bug is the config path + missing JIT keys.)*

## Order of execution & risk table
| Phase | Focus | Effort | Risk | Order |
|---|---|---|---|---|
| 0 Unblockers | entitlements path + JIT keys; delete orphan `src/` | 0.5d | Low | 1 |
| 1 Freed-bytes truth | `trash/mod.rs`, `caches/mod.rs`, `clean.rs`, `cache.rs`, DTO | 1.5d | Med | 2 |
| 2 Honest sizing | blocks, hardlink dedup, statfs, TM snapshots, reclaimability DTO | 2d | Med | 3 |
| 3 Deletion engine | errno/flags taxonomy, action model, cancellation, TOCTOU (admin deferred) | 2d | Med | 4 |
| 4 Unsigned dist + FDA | self-signed-for-persistence, Gatekeeper docs, FDA probe, NSWorkspace | 1.5–2d | Med | 5 |
| 5 Scan + search | surface dropped cleaners, jwalk pruning, fuzzy+facets | 1.5d | Med | 6 |
| 6 Guide + result UX | state-aware wizard, snapshot reclaim, Trash honesty, failure log | 2.5d | Med | 7 |

**Total: ~11–11.5 days.** No High-risk phases remain (Developer-ID/notarization complexity and the
privileged helper — the two High-risk items — are removed by the no-signing decision).

## Iteration log
- **Iteration 0** (2026-06-13): initial plan from codebase grounding + parallel research
  (sizing, deletion, UX, Tauri-signing).
- **Iteration 1** (2026-06-13): 5 review agents + Codex. Surfaced the APFS-clone dedup gap and
  errno-taxonomy weaknesses.
- **Iteration 2** (2026-06-13): 3 review agents + Codex, **re-grounded against code**. Corrected
  three wrong premises: (a) top-level `src/` is an **orphan**, not a byte-identical mirror (verified
  via `cargo metadata`); (b) `tauri/Entitlements.plist` **exists** — the bug is the config path
  `../` + missing JIT keys, not an absent file; (c) search + failure-grouping UI **already ship** —
  improve, don't rebuild. Added: freed-bytes accounting as the high-priority first fix (incl.
  `clean_cache` bypass), `statfs`-delta raciness handling, `chflags` read-modify-write +
  SF_RESTRICTED/schg distinction, EACCES≠NeedsAdmin (ownership check), structurally-dead
  cancellation, CI Apple-signing env contract, XPC audit-token validation, NSWorkspace deep link,
  actionable local-snapshot reclaim, Trash "0 bytes" fix, and a local failure log. Re-ordered to
  put accounting truth + cheap unblockers first; split signing into a cert-gated phase.
- **Iteration 3** (2026-06-13): 2 consensus agents → both "ready to implement" (algorithm +
  permissions/UX). Codex final pass added two more verified MAJOR items, now folded in: (a) block
  sizing must also cover `plugins/mod.rs::default_calculate_size` and
  `caches/mod.rs::calculate_cache_size` (else project/cache totals stay apparent-size); (b) a
  **typed system action model** — `handleClean` currently drops `clean_command` and routes system
  items (snapshots, Trash, aggregate roots) to a generic recursive delete; add a `PathDelete |
  EmptyTrash | ThinTimeMachineSnapshots | RunCommand` enum with a hard refuse-list for `/`, `~`,
  `~/.Trash`, and aggregate roots. Minor fixes: corrected the cleaner-drop citation to
  `tauri/src/commands/cleaners.rs:15,41`, structural `tmutil` args (not `split_whitespace`), reuse
  JS `relaunch()`, note `objc2`/`cocoa` dep for NSWorkspace. **No BLOCKING/MAJOR issues remain.**
  Final Codex hardening folded in: distinct `statfs` fields (capacity vs free), and a
  **backend-owned** action model (UI sends `{item_id, action_id}` only; backend constructs argv
  from a fixed registry — no frontend argv). **Final verdict: all reviewers + Codex →
  "CONSENSUS: ready to implement."**
- **Iteration 4** (2026-06-13): user decisions folded in — (1) **no Developer ID; unsigned `.dmg`
  on GitHub** → Phase 4 reshaped from Developer-ID/notarization to self-signed-for-FDA-persistence
  + Gatekeeper "Open Anyway" docs + re-grant-after-update floor; privileged XPC helper deferred
  (no signing) so `NeedsAdmin` is classified/explained, no `osascript`. (2) **Trash default with a
  user-selectable permanent option** → Phase 6.3 keeps the toggle. Net effect: both former
  High-risk phases drop to Medium; total ~11 days. Open questions reduced to one on-device
  validation (self-signed TCC persistence).
