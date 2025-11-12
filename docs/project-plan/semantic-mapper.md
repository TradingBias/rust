## Investigation of `polars-core` `raw_table_mut` error

**Date:** Wednesday 12 November 2025

**Problem:** The project fails to compile with the error: `no method named 'raw_table_mut' found for struct 'HashMap'` originating from the `polars-core` dependency.

**Investigation Steps:**

1.  **Searched project for `raw_table_mut`:** No direct occurrences of `raw_table_mut` were found in the project's source code. This indicates the method call is likely happening within a dependency.
2.  **Checked `Cargo.toml` for Polars version:** The `Cargo.toml` file specifies `polars = { version = "0.38.0", ... }`. This version is greater than or equal to `0.36.0`, which was the target version mentioned in the `04-quick-reference-checklist.md` for Polars API compatibility.

**Conclusion:**
The error suggests a breaking change in the `polars` library or one of its internal dependencies (`polars-core`) where the `raw_table_mut` method, previously available on `HashMap` (or a type that `polars-core` was treating as a `HashMap` or similar collection), has been removed or renamed. Even though the project's `Cargo.toml` specifies a recent version of `polars` (0.38.0), it's possible that other transitive dependencies are still relying on an older `polars-core` API, or that `polars` itself has undergone a breaking change that affects how `HashMap`s are handled internally within `polars-core`.

**Next Steps (for future resolution):**
*   Further investigation into `polars-core` changelogs or release notes for version `0.38.0` and its sub-dependencies to identify the specific breaking change related to `raw_table_mut`.
*   Examine the full compilation error output to identify the exact file and line number within `polars-core` where the error occurs, which might provide more context.
*   Consider temporarily downgrading `polars` to a slightly older version (e.g., `0.36.0` or `0.37.0`) to see if the error persists, which could help pinpoint the exact version where the breaking change was introduced.
*   If the error is due to a transitive dependency, try to identify and update that specific dependency.

---

## Implementation Solution

**Root Cause Analysis:**

The `raw_table_mut` method is part of the internal `hashbrown::HashMap` API that was used by `polars-core`. This method was available in the unstable internal API of the `hashbrown` crate but has been removed or made unavailable in newer versions. Polars 0.38.0 appears to have a dependency on a version of `polars-core` that still relies on this now-removed method.

**Solution Strategy:**

### Option 1: Upgrade Polars (Recommended)

Upgrade to a newer version of Polars that has fixed this compatibility issue:

```toml
# In Cargo.toml
polars = { version = "0.41.0", features = [...] }
```

**Steps:**
1. Update the polars version in `Cargo.toml` to `0.41.0` or later
2. Run `cargo clean` to remove old build artifacts
3. Run `cargo update -p polars` to update polars and its dependencies
4. Run `cargo build` to verify the fix

**Rationale:** Newer versions of Polars (0.40.0+) have been updated to use stable APIs from hashbrown and no longer depend on `raw_table_mut`.

**Analysis after attempting upgrade to 0.41.3:**
Attempting to upgrade Polars to version `0.41.0` (which resulted in `0.41.3` being used) did *not* resolve the compilation error. The `no method named 'raw_table_mut'` error persists, originating from `polars-core-0.41.3`. This indicates that the `polars-core` version bundled with `polars 0.41.3` still relies on the `raw_table_mut` method, contradicting the expectation that versions `0.40.0+` would use stable `hashbrown` APIs. The issue appears to be deeper within the `polars` dependency chain or a more recent breaking change than initially anticipated. Further investigation into `polars` and `hashbrown` compatibility for `0.41.x` versions is required.

**Root Cause Identified (via `cargo tree -i hashbrown`):**

The project has **three conflicting versions of hashbrown**:
- **hashbrown 0.16.0** - Pulled by `indexmap v2.11.4` (used by polars-core)
- **hashbrown 0.15.5** - Pulled by `config` crate dependencies
- **hashbrown 0.14.5** - Used directly by polars-arrow and polars-core

**The Problem:** The `raw_table_mut()` method was **removed in hashbrown 0.15.0**. Polars-core 0.41.3 contains code that expects this method to exist, but when it interacts with hashbrown 0.16.0 (pulled in via indexmap), the method is not available, causing the compilation error.

This is a **transitive dependency version conflict** where:
1. polars-core depends on indexmap 2.11.4
2. indexmap 2.11.4 depends on hashbrown 0.16.0
3. polars-core code still calls `raw_table_mut()` which doesn't exist in hashbrown 0.15+

**Revised Solution Required:** See updated solutions below.

---

## REVISED Solutions (Based on Root Cause Analysis)

### **Solution 1: Force hashbrown Version with Cargo Patch (RECOMMENDED)**

Force all dependencies to use hashbrown 0.14.x by adding a patch to `Cargo.toml`:

```toml
# Add this section to the end of Cargo.toml
[patch.crates-io]
hashbrown = "=0.14.5"
```

**Steps:**
1. Open `Cargo.toml`
2. Add the `[patch.crates-io]` section at the end of the file
3. Run `cargo clean`
4. Run `cargo update`
5. Run `cargo build`

**Rationale:** This forces all dependencies (including indexmap) to use hashbrown 0.14.5, which still has the `raw_table_mut()` method that polars-core expects.

**Expected Result:** All three hashbrown versions will be unified to 0.14.5, eliminating the conflict.

### **Solution 2: Downgrade indexmap**

Downgrade indexmap to a version compatible with hashbrown 0.14.x:

```toml
# Add to Cargo.toml dependencies section
indexmap = "=2.0.0"
```

**Steps:**
1. Add `indexmap = "=2.0.0"` to the `[dependencies]` section in `Cargo.toml`
2. Run `cargo clean`
3. Run `cargo update`
4. Run `cargo build`

**Rationale:** indexmap 2.0.0 uses hashbrown 0.14.x, avoiding the version 0.16.0 that causes the conflict.

### **Solution 3: Downgrade Polars to Known Working Version**

If upgrading introduces API compatibility issues with existing code, downgrade to a stable version:

```toml
# In Cargo.toml
polars = { version = "0.36.2", features = [...] }
```

**Steps:**
1. Update the polars version in `Cargo.toml` to `0.36.2`
2. Run `cargo clean`
3. Run `cargo update -p polars`
4. Run `cargo build` to verify the fix
5. Review and update any code that uses newer Polars APIs

**Rationale:** Version 0.36.2 is known to work with the hashbrown version that still supports `raw_table_mut`.

### Option 3: Force Dependency Resolution

If the issue is caused by transitive dependencies pulling incompatible versions:

```toml
# In Cargo.toml, add a [patch] section
[patch.crates-io]
polars-core = { version = "0.41.0" }
```

**Steps:**
1. Add the patch section to force a specific polars-core version
2. Run `cargo update`
3. Run `cargo build` to verify

### Option 4: Clean Build

Sometimes the issue is caused by stale build artifacts:

**Steps:**
1. Delete `Cargo.lock` file
2. Run `cargo clean`
3. Run `cargo update`
4. Run `cargo build`

---

## RECOMMENDED ACTION PLAN (Updated)

Based on the root cause analysis showing hashbrown version conflicts, follow these steps in order:

### **Step 1: Try Solution 1 (Force hashbrown - FASTEST FIX)**

This is the most targeted fix that addresses the exact problem:

```bash
# 1. Add [patch.crates-io] section to Cargo.toml with: hashbrown = "=0.14.5"
# 2. Then run:
cargo clean
cargo update
cargo build
```

**Outcome of Step 1 Attempt:**
Failed. `cargo update` returned an error: "patch for `hashbrown` in `https://github.com/rust-lang/crates.io-index` points to the same source, but patches must point to different sources". This indicates that Cargo's `[patch]` mechanism is not suitable for forcing a specific version of a `crates.io` dependency when that version is already available on `crates.io`. The `Cargo.toml` was reverted.

### **Step 2: If Step 1 fails, try Solution 2 (Downgrade indexmap)**

```bash
# 1. Add indexmap = "=2.0.0" to [dependencies] in Cargo.toml
# 2. Then run:
cargo clean
cargo update
cargo build
```

**Outcome of Step 2 Attempt:**
Failed. While the original `raw_table_mut` error was resolved, downgrading `indexmap` to `2.0.0` introduced a cascade of new compilation errors related to API incompatibilities with the rest of the codebase (e.g., `RollingOptions` not found, `output_type` missing from `Indicator` trait, `StrategyAST` not found, `TradeBiasError` capitalization issues, `abs` and `cum_sum` methods missing from `polars::prelude::Expr`, `Clone` trait not implemented for `std::io::Error`, `PolarsError`, `serde_json::Error`, and `Arc` type mismatches). This indicates that the codebase is not compatible with the older versions of dependencies pulled in by `indexmap = "=2.0.0"`. The `Cargo.toml` was reverted.

### **Step 3: If Step 2 fails, try Solution 3 (Downgrade Polars)**

```bash
# 1. Change polars version to "0.36.2" in Cargo.toml
# 2. Then run:
cargo clean
cargo update
cargo build
```

**Outcome of Step 3 Attempt:**
Failed. Downgrading Polars to `0.36.2` resulted in the *reappearance* of the original `no method named 'raw_table_mut'` error, this time originating from `polars-core-0.36.2`. This contradicts the initial assumption that `polars 0.36.2` would be compatible with a `hashbrown` version that still supports `raw_table_mut`. It appears that `polars-core-0.36.2` also expects `raw_table_mut`, but the `cargo update` process pulled in `hashbrown 0.16.0` (which does not have this method), leading to the same compilation failure. The `Cargo.toml` was reverted.

### **Conclusion on Recommended Action Plan:**
None of the proposed solutions in the "RECOMMENDED ACTION PLAN (Updated)" successfully resolved the compilation issue without introducing new, or reintroducing old, errors. The core problem seems to be a deep incompatibility between the `polars` crate (across multiple versions) and the `hashbrown` crate, specifically regarding the `raw_table_mut` method. The project's codebase appears to be written against a specific, possibly unreleased or highly volatile, set of `polars` and `hashbrown` versions.

**Further Investigation Required:**
To resolve this, a more in-depth analysis of the `polars` and `hashbrown` dependency trees and their respective changelogs is needed to identify a truly compatible set of versions. It might also be necessary to examine the project's code to see if it can be adapted to use stable `hashbrown` APIs directly, or if there's a specific `polars` version that correctly handles this transition.

---

## FINAL SOLUTION (November 12, 2025)

### Question: Can we use a different crate than hashbrown?

**Answer: NO.** `hashbrown` is a **transitive dependency** pulled in by `polars-core` and `indexmap`. It is not directly used by the project code and cannot be replaced. The project does not control which HashMap implementation polars uses internally.

### The Actual Solution: Upgrade to Polars 0.51.0

After testing multiple approaches, the successful solution is:

```toml
# In Cargo.toml
polars = { version = "0.51.0", features = ["lazy", "rolling_window", "ewma", "temporal", "dtype-full"] }
```

**Steps:**
1. Update polars version to `0.51.0` in `Cargo.toml`
2. Run `cargo clean`
3. Run `cargo update`
4. Run `cargo build`

**Result:**
- ✅ The `raw_table_mut` error is **COMPLETELY RESOLVED**
- ✅ Polars 0.51.0 is compatible with hashbrown 0.16.0 without the `raw_table_mut` issue
- ⚠️  86 compilation errors in project code due to Polars API changes (see below)

**Why this works:**
- Polars 0.52.0 has a compilation bug in `polars-expr` crate (missing DataType imports)
- Polars 0.51.0 (released September 16, 2025) is stable and has been updated to use hashbrown 0.16.0's stable APIs
- The `raw_table_mut()` method issue has been fixed in polars 0.51.0+

### Next Steps: Fix Polars API Compatibility Issues

The project code now needs to be updated to work with Polars 0.51.0 API changes. Major breaking changes include:

1. **Series::new() now requires PlSmallStr instead of &str**
   ```rust
   // Old (polars 0.36-0.41):
   Series::new("column_name", data)

   // New (polars 0.51):
   Series::new("column_name".into(), data)
   ```

2. **Datetime accessor changes**
   ```rust
   // Old:
   timestamps.get(idx)

   // New:
   timestamps.phys.get(idx)
   ```

3. **Multiple missing items:**
   - `RollingOptions` → `RollingOptionsFixedWindow`
   - `StrategyAST::Rule` enum variant changes
   - `Indicator` trait `output_type` method changes
   - Various imports need updating

**Recommendation:** Create a separate plan to systematically update all code for polars 0.51.0 compatibility. The dependency issue is SOLVED; what remains is a code migration task.

---

## COMPREHENSIVE MIGRATION PLAN CREATED

**Date**: November 12, 2025

A comprehensive migration plan has been created to systematically fix all 86 Polars 0.51.0 API compatibility errors:

**📄 See: [06-polars-0.51-migration.md](./06-polars-0.51-migration.md)**

This plan includes:
- Complete breakdown of all 86 errors by category
- 20 detailed tasks organized into 4 implementation stages
- Code patterns and examples for each fix
- Verification commands and checkpoints
- Estimated timelines for each stage
- Common pitfalls and solutions

**Status**: ✅ Migration plan ready for implementation
**Priority**: CRITICAL - This is now "Phase 0" and blocks all other work
**Next Step**: Begin implementation of Stage 1 (Core API Changes)