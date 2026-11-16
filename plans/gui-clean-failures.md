# GUI clean failures (v0.4.3 report)

Summary: a full GUI clean reported 10 failures. Five root causes, all fixed in one PR.

1. Go module cache scanned as projects: `~/go/pkg/mod/.../v2@x/bin` listed as Go build output.
2. Read-only dirs (Go module cache is 0555) make every file delete fail with EACCES.
3. `rustup toolchain remove` and `swift package purge-cache` refused by the GUI allow-list.
4. Per-project `DerivedData/*/SourcePackages` item carries `swift package purge-cache`, which
   cleans the global SwiftPM cache, not that folder.
5. An item already removed earlier in the same batch fails with "System item not found".

## Changes

| # | File | Change |
|---|------|--------|
| 1 | `crates/null-e-core/src/scanner/parallel.rs` | skip `.../pkg/mod` holding `cache/download` (Go module cache; the caches module owns it) |
| 2 | `crates/null-e-core/src/fsutil.rs` | on `PermissionDenied`, if we own the parent and it lacks the owner write bit, add it, retry once, restore the old mode |
| 3 | `tauri/src/commands/cleaners.rs` | allow exactly `rustup toolchain remove` and `swift package purge-cache` (prefix list, not whole binaries) |
| 4 | `crates/null-e-core/src/cleaners/ios_deps.rs` | per-project SourcePackages item: `clean_command: None` (plain path delete) |
| 5 | `tauri/src/commands/cleaners.rs` | item not re-detected and its path is NotFound under home: success, 0 bytes, "Already removed" |
| 6 | `fsutil.rs`, `runtimes.rs`, `cleaners.rs` | `resolve_tool` / `tool_search_path`: a Finder-launched app finds tools in `~/.cargo/bin`, Homebrew, bun, Go, and children get that PATH. `rustup default` now resolves, so the default toolchain is marked active instead of deletable |

Review: two rounds (code-reviewer). Round 1 found 4 issues (placeholder Docker paths, `pkg/mod` false
positives, mode left changed, PATH in a Finder-launched app); all fixed and confirmed in round 2.

## Verify

- Unit tests: scanner skips `pkg/mod`; `remove_file_robust` deletes a file inside a 0555 dir.
- `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`.
- Real run: build the CLI, scan a temp tree holding a fake `go/pkg/mod/x@v1/{go.mod,bin}`: not listed.

## Risks

- Fix 2 changes a parent dir's mode only when we own it and only to add `u+w`; the dir is being
  emptied anyway. Root-owned dirs are untouched (still NeedsAdmin).
- Fix 3: both binaries run with no shell and only with commands from our own detection.
