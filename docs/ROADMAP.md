# Scarab Terminal Emulator - Development Roadmap

## 🎯 Vision
Build a high-performance, GPU-accelerated terminal emulator with a split-process architecture, hybrid plugin system, and game-engine-driven UI.

## 📊 Current State (v0.1.0)

### ✅ Completed
- Basic Cargo workspace structure (6 crates)
- Shared memory IPC bridge (lock-free sync with AtomicU64)
- PTY process management via portable-pty
- Protocol definitions (#[repr(C)] layout)
- Basic Bevy client scaffolding

### ❌ Gaps
- VTE parser integration (alacritty_terminal disabled)
- Text rendering pipeline (cosmic-text)
- Control socket for IPC commands (resize, input, etc.)
- Fusabi VM bytecode execution
- Fusabi Interpreter AST walker
- Plugin API and hot-reloading
- Input handling and key bindings
- Session persistence and multiplexing

---

## 🚀 Development Phases

### Phase 1: Core Terminal Emulation (Weeks 1-3)
**Goal**: Functional terminal with basic rendering

#### Workstream 1A: VTE Parser & Grid State Management
- **Owner**: Terminal Emulation Specialist
- **Dependencies**: None (can start immediately)
- **Deliverables**:
  - Integrate VTE parser (fix alacritty_terminal deps or use alternative)
  - Parse ANSI escape sequences
  - Update SharedState grid from parsed commands
  - Handle cursor positioning, colors, attributes
  - Support scrollback buffer
- **Success Criteria**: Can run `ls`, `cat`, `vim` with correct rendering

#### Workstream 1B: Text Rendering Engine
- **Owner**: Graphics/Rendering Specialist
- **Dependencies**: Needs SharedState contract (already defined)
- **Deliverables**:
  - Integrate cosmic-text with Bevy
  - Create texture atlas for glyph caching
  - Implement mesh generation from grid cells
  - Handle font loading and fallbacks
  - Support Unicode rendering
- **Success Criteria**: 60 FPS rendering at 200x100 cells

#### Workstream 1C: IPC Control Channel
- **Owner**: Systems/IPC Specialist
- **Dependencies**: None (independent of rendering)
- **Deliverables**:
  - Unix Domain Socket (macOS/Linux) / Named Pipes (Windows)
  - ControlMessage serialization (rkyv)
  - Resize event handling
  - Input forwarding (client → daemon)
  - Client discovery and connection
- **Success Criteria**: Multi-client support, proper window resizing

---

### Phase 2: Plugin System & Extensibility (Weeks 4-6)
**Goal**: Hot-reloadable plugins with dual runtime

#### Workstream 2A: Fusabi VM (AOT Runtime)
- **Owner**: Compiler/VM Specialist
- **Dependencies**: Phase 1 completion for integration testing
- **Deliverables**:
  - Bytecode format definition (.fzb)
  - Stack-based VM with register optimization
  - rkyv serialization for zero-copy loading
  - FFI bridge to Rust (call daemon functions)
  - Security sandbox (memory limits, syscall restrictions)
- **Success Criteria**: Execute compiled plugins at <1ms overhead

#### Workstream 2B: Fusabi Interpreter (Script Runtime)
- **Owner**: Language/Interpreter Specialist
- **Dependencies**: Phase 1B (rendering) for UI scripting
- **Deliverables**:
  - F# dialect parser (.fsx)
  - AST walker with hot-reload
  - Bevy UI integration (overlays, menus)
  - File watcher for auto-reload
  - Error reporting with line numbers
- **Success Criteria**: <100ms reload time, no Rust recompilation needed

#### Workstream 2C: Plugin API & Lifecycle
- **Owner**: API Design Specialist
- **Dependencies**: 2A and 2B (VM and Interpreter)
- **Deliverables**:
  - Plugin trait definitions
  - Hook system (pre-output, post-input, etc.)
  - Configuration loading (TOML/YAML)
  - Plugin discovery and loading
  - Version compatibility checks
- **Success Criteria**: 3rd-party plugin template working

---

### Phase 3: Advanced Features (Weeks 7-9)
**Goal**: Production-ready terminal with power-user features

#### Workstream 3A: Session Management & Multiplexing
- **Owner**: Session Management Specialist
- **Dependencies**: Phase 1 complete
- **Deliverables**:
  - Named sessions (like tmux)
  - Session persistence across client disconnects
  - Multiple PTY support in single daemon
  - Tab/split management
  - Session attach/detach
- **Success Criteria**: Survive client crashes, resume from different machine

#### Workstream 3B: Advanced UI/UX
- **Owner**: UI/UX Specialist
- **Dependencies**: Phase 1B (rendering), Phase 2B (interpreter)
- **Deliverables**:
  - Vimium-style link hints
  - Spacemacs-like leader key menus
  - Command palette (fuzzy search)
  - Configurable key bindings
  - Theme system
- **Success Criteria**: Keyboard-only workflow, <200ms menu open time

#### Workstream 3C: Configuration System
- **Owner**: Config Management Specialist
- **Dependencies**: Phase 2C (plugin API)
- **Deliverables**:
  - TOML configuration format
  - Hot-reload configuration
  - Per-shell/per-directory configs
  - Sensible defaults
  - Config validation
- **Success Criteria**: Zero-config startup, full customization available

---

### Phase 4: Production Hardening (Weeks 10-12)
**Goal**: Battle-tested, documented, cross-platform

#### Workstream 4A: Performance Optimization
- **Owner**: Performance Engineer
- **Dependencies**: All features implemented
- **Deliverables**:
  - Profiling and bottleneck analysis
  - Memory usage optimization (<100MB baseline)
  - GPU memory management
  - Input latency reduction (<10ms)
  - Benchmark suite
- **Success Criteria**: <1% CPU idle, <50ms P99 frame time

#### Workstream 4B: Testing & Documentation
- **Owner**: QA/Documentation Specialist
- **Dependencies**: All features implemented
- **Deliverables**:
  - Unit tests (80%+ coverage)
  - Integration tests (PTY, IPC, rendering)
  - User documentation
  - Plugin development guide
  - Architecture deep-dive
- **Success Criteria**: CI passing, docs.rs published

#### Workstream 4C: Platform Support
- **Owner**: Platform Engineer
- **Dependencies**: Core features stable
- **Deliverables**:
  - Linux support (X11, Wayland)
  - macOS support (Metal backend)
  - Windows support (DirectX backend, Named Pipes)
  - Package managers (Homebrew, AUR, Cargo)
  - Release automation
- **Success Criteria**: Single binary per platform, <10MB compressed

---

## 🎯 Milestones

### M1: MVP Terminal (End of Phase 1)
- Can replace iTerm2/Alacritty for daily use
- Basic text rendering works
- PTY I/O is reliable

### M2: Plugin Ecosystem (End of Phase 2)
- 5+ example plugins working
- Hot-reload functional
- Community can write plugins

### M3: Feature Parity (End of Phase 3)
- Matches tmux + Alacritty feature set
- Unique Fusabi scripting advantage
- Production-ready for early adopters

### M4: General Availability (End of Phase 4)
- Cross-platform support
- Performance competitive with Alacritty
- Documentation complete

---

## 🔧 Technical Decisions

### VTE Parser Strategy
- **Option A**: Fix alacritty_terminal dependency conflicts (preferred)
- **Option B**: Use vte crate directly with custom state machine
- **Decision Point**: Issue #1

### Fusabi Language Design
- Syntax: F# subset (functional-first, type-safe)
- Interop: Direct Rust FFI with zero-copy data
- Security: WASM-style sandbox with capability-based access

### Rendering Architecture
- **Primary**: Bevy ECS for UI and terminal grid
- **Text**: cosmic-text for shaping + GPU texture atlas
- **Fallback**: Software rendering for SSH/remote

---

## 📦 Dependencies Audit

### Critical Path
1. VTE parser (blocks terminal functionality)
2. Text rendering (blocks usability)
3. IPC control channel (blocks multi-client)

### Nice-to-Have
- Plugin system (can be delayed)
- Advanced UI (can be iterative)
- Windows support (macOS/Linux first)

---

## 🚨 Risk Management

| Risk | Impact | Mitigation |
|------|--------|------------|
| alacritty_terminal deps | High | Alternative: vte crate, custom parser |
| Bevy breaking changes | Medium | Pin to 0.15.x, upgrade in maintenance window |
| Shared memory portability | Medium | Abstract with trait, platform-specific impls |
| Fusabi VM security | High | WASM-style sandbox, extensive fuzzing |
| Performance regressions | Medium | Continuous benchmarking in CI |

---

## 📈 Success Metrics

- **Latency**: <10ms input-to-display (P99)
- **Memory**: <100MB baseline, <500MB with 10 sessions
- **CPU**: <1% idle, <5% during scrolling
- **FPS**: 60+ FPS sustained
- **Startup**: <500ms cold start
- **Plugin Load**: <100ms for interpreted, <1ms for compiled

---

## 🎓 Learning Resources

- [Alacritty Architecture](https://github.com/alacritty/alacritty)
- [VTE Spec](https://vt100.net/docs/vt100-ug/)
- [cosmic-text Examples](https://github.com/pop-os/cosmic-text)
- [Bevy Best Practices](https://bevy-cheatbook.github.io/)

---

**Last Updated**: 2025-11-21
**Version**: 0.1.0
**Status**: 🟡 In Progress (Phase 1)
