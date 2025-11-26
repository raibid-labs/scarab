# Scarab Terminal - Deep Technical Audit
**Date:** November 25, 2025
**Auditor:** Gemini CLI Agent

## 1. Executive Summary
Scarab is an ambitious, high-performance terminal emulator project employing a client-daemon architecture and a unique, vertically integrated plugin system based on "Fusabi" (an F# dialect). The project is currently in an **Alpha state (v0.1.0-alpha.13)**.

While the core architecture (split process, shared memory, zero-copy serialization) is technically sound and sophisticated, the project is currently in a **fragile transitional phase** regarding its configuration system. The migration from TOML to Fusabi-based configuration is incomplete, leaving the "new" system non-functional ("using defaults").

## 2. Architecture Review

### 2.1 Core Components
*   **Daemon (`scarab-daemon`):** The "brain" of the operation. Manages PTYs, plugins, and state. It effectively leverages `fusabi-vm` for high-performance, sandboxed plugin execution.
*   **Client (`scarab-client`):** A lightweight frontend built with the **Bevy** game engine (v0.15). This provides GPU acceleration but introduces a heavy dependency.
*   **Protocol (`scarab-protocol`):** Uses `rkyv` for zero-copy serialization, a distinct performance advantage over typical JSON/Protobuf implementations.
*   **Plugins:** A dual-runtime approach:
    *   **Daemon:** Compiled `.fzb` bytecode executed by `fusabi-vm`.
    *   **Client:** Interpreted `.fsx` scripts handled by `fusabi-frontend`.

### 2.2 The "Fusabi" Ecosystem
The project relies heavily on `fusabi`, a custom F# dialect.
*   **Strengths:** Vertical integration allows for deep optimization (<1ms execution). The dual runtime (AOT vs. Interpreted) balances performance and developer experience (DX).
*   **Risks:** The project is tightly coupled to this specific language implementation. Any stagnation in `fusabi` development directly blocks Scarab features (as seen in the configuration system).

## 3. Critical Findings

### 3.1 🚨 Incomplete Configuration Migration (High Risk)
The project is mid-migration from TOML to Fusabi configuration (`config.fsx`).
*   **Evidence:** `crates/scarab-config/src/fusabi_loader.rs` contains multiple TODOs:
    ```rust
    // TODO: Query the module for [<TerminalConfig>] attribute values
    // TODO: Query the module for [<FontConfig>] attribute values
    ```
*   **Impact:** The code explicitly prints `⚠️ Fusabi config loader is WIP - using defaults`. Users attempting to use the "new" configuration system will find it silently ignored or non-functional, despite documentation encouraging its use.

### 3.2 📉 Documentation vs. Reality Mismatch (Medium Risk)
*   **Version Confusion:** `Cargo.toml` defines `fusabi-vm` and `fusabi-frontend` at version **0.12.0**. However, multiple documentation files (e.g., `CLAUDE.md`, `README.md`, `CHANGELOG.md`) refer to version **0.5.0**.
*   **Feature Claims:** Documentation claims Fusabi configuration is a "major feature" of the current release, while the code confirms it is non-functional scaffolding.

### 3.3 🛠️ Technical Debt & Feature Gaps
A grep of the codebase revealed significant TODOs:
*   **Clipboard:** `crates/scarab-clipboard/src/clipboard.rs` - `// TODO: Implement proper X11 primary selection support`. This is a standard terminal feature that is missing.
*   **UI/Client:**
    *   `search_overlay.rs`: Case sensitivity and regex support are hardcoded to `false` with TODOs to make them configurable.
    *   `scrollback_selection.rs`: Coordinate conversion logic is missing (`// TODO: Convert cursor_pos to scrollback line`).
*   **Security:** `crates/scarab-config/src/registry/security.rs` - `// TODO: Implement GPG signature verification`. Plugin security is currently minimal.

## 4. Code Quality & Testing

### 4.1 Codebase
*   **Style:** The Rust code is idiomatic, making good use of traits, async/await (`tokio`), and error handling (`anyhow`, `thiserror`).
*   **Structure:** The workspace structure is clean and modular.

### 4.2 Testing
*   **Strengths:** `crates/scarab-daemon/tests/plugin_integration.rs` is excellent. It exhaustively tests the plugin lifecycle, bytecode loading, error propagation, and timeouts. This suggests the *plugin system* is robust, even if the *configuration system* is not.
*   **Gaps:** Integration tests for the configuration loader are likely failing or non-existent given the "WIP" state.

## 5. Recommendations

### Immediate Actions (Next Sprint)
1.  **Freeze Configuration Migration:** Either complete the `fusabi_loader.rs` implementation immediately or roll back the default to TOML. Do not leave the main release in a state where the primary config method is "WIP - defaults only".
2.  **Synchronize Versions:** Update all documentation to match `Cargo.toml` (Fusabi v0.12.0).
3.  **Implement X11 Selection:** Prioritize X11 primary selection support in `scarab-clipboard`.

### Strategic Improvements
1.  **Security Audit:** Before encouraging third-party plugins, implement the missing GPG signature verification.
2.  **Client Polish:** Address the "TODOs" in the client UI (search, scrollback) to improve the user experience.

## 6. Conclusion
Scarab is a technically impressive project with a unique selling point (Fusabi). However, it is currently in a dangerous "uncanny valley" where its documentation promises features (Fusabi config) that the code has not yet delivered. Closing this gap is the highest priority for stability.
