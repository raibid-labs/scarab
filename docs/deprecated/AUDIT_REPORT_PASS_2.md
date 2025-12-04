# Scarab Technical Audit Report (Pass 2)
**Date:** December 1, 2025
**Auditor:** Gemini Agent

## 1. Critical Integration Failure: 'mimic' Missing
The most urgent finding is that the `mimic` library, required for VTE conformance testing, is **physically missing** from the environment.

- **Expected Path:** `~/raibid-labs/mimic` (per user instruction) or `../../../mimic` (per `Cargo.toml`).
- **Actual Status:** Neither path exists.
- **Impact:** `crates/scarab-daemon` will likely fail to compile its test suite.
- **Action Required:** The `mimic` repository must be cloned to `~/raibid-labs/mimic` or the `scarab` dependency path must be updated to point to a valid location.

## 2. Code Quality & Safety Audit
### A. Scarab Daemon (`crates/scarab-daemon`)
1.  **Unsafe Transmute in Session Manager:**
    -   **File:** `src/session/manager.rs:41`
    -   **Code:** `unsafe { std::mem::transmute(pair.master) }`
    -   **Risk:** This casts `Box<dyn MasterPty>` to `Box<dyn MasterPty + Send + Sync>`. This forces thread-safety traits onto an object that the compiler doesn't guarantee is thread-safe. If the underlying PTY implementation uses thread-local storage or non-atomic refcounts, this **will cause undefined behavior/crashes**.
    -   **Recommendation:** Wrap the PTY master in a `Mutex` (which provides `Sync`) rather than forcing the trait with `transmute`.

2.  **Database Connection Performance:**
    -   **File:** `src/session/store.rs`
    -   **Issue:** Every method (`save_session`, `load_sessions`, etc.) calls `self.connect()`, opening a *new* SQLite connection from disk.
    -   **Impact:** High latency on session operations. If the daemon saves state frequently (e.g., on every resize or output), this will cause significant I/O lag.
    -   **Recommendation:** Refactor `SessionStore` to hold a long-lived `Connection` (guarded by a `Mutex`) or use a connection pool (e.g., `r2d2`).

### B. Scarab Client (`crates/scarab-client`)
1.  **Dangerous Unwraps in UI Code:**
    -   **File:** `src/ui/keybindings.rs`
    -   **Issue:** `KeyBinding::from_string(...).unwrap()` is used. If this parsing logic is triggered by user configuration or runtime input, a typo will **crash the entire application**.
    -   **Recommendation:** Change to return `Result` and handle errors gracefully (log error + fallback).

2.  **Shared Memory Safety:**
    -   **File:** `src/integration.rs` and `src/rendering/text.rs`
    -   **Status:** Uses `unsafe` to dereference shared memory pointers. This is standard for this architecture but requires strict validation that the memory segment size matches `SharedState` layout.

## 3. Architecture Re-Verification
-   **Rendering:** Confirmed 2D (`Camera2d`, `Mesh2d`). The "3D renderer" concern is resolved.
-   **Dependencies:** `bevy_ratatui` is strictly a conceptual reference, not a dependency.

## 4. Updated Action Plan
1.  **Locate/Install Mimic:** We cannot proceed with VTE verification without the library.
2.  **Fix Session Store:** Refactor `SessionStore` to persist the DB connection.
3.  **Safety Refactor:** Remove `transmute` in `SessionManager` and dangerous `unwrap`s in `KeyBinding`.
4.  **Mimic Integration:** Once installed, implement the "Headless API" and "State Inspection" features requested in the previous audit.
