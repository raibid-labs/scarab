# Scarab Meta-Orchestrator Guide

## 🎯 Purpose

This guide explains how to use Claude-Flow's swarm coordination to work on multiple Scarab issues in parallel, maximizing development velocity while maintaining code quality.

---

## 🚀 Quick Start

### 1. Initialize Swarm
```bash
npx claude-flow@alpha swarm init \
  --topology mesh \
  --max-agents 6 \
  --session-id scarab-phase1
```

### 2. Spawn Specialized Agents

**Phase 1 - Core Terminal (Parallel Execution)**

```bash
# Agent 1: VTE Parser (Issue #1)
npx claude-flow@alpha agent spawn \
  --type code-analyzer \
  --task "Implement VTE parser integration per Issue #1" \
  --context "docs/issues/01-vte-parser-integration.md" \
  --session scarab-phase1

# Agent 2: Text Rendering (Issue #2)
npx claude-flow@alpha agent spawn \
  --type frontend-developer \
  --task "Implement text rendering engine per Issue #2" \
  --context "docs/issues/02-text-rendering-engine.md" \
  --session scarab-phase1

# Agent 3: IPC Control (Issue #3)
npx claude-flow@alpha agent spawn \
  --type backend-dev \
  --task "Implement IPC control channel per Issue #3" \
  --context "docs/issues/03-ipc-control-channel.md" \
  --session scarab-phase1
```

### 3. Monitor Progress
```bash
npx claude-flow@alpha swarm status --session scarab-phase1
npx claude-flow@alpha agent list
npx claude-flow@alpha task status
```

---

## 📊 Workstream Coordination

### Phase 1: Core Terminal Emulation (3 Parallel Streams)

#### Stream 1A: VTE Parser & Grid State
- **Agent Type**: `code-analyzer` or `backend-dev`
- **Primary Files**: `scarab-daemon/src/main.rs`, `scarab-daemon/src/vte.rs`
- **No Conflicts**: Independent of other streams
- **Completion Criteria**: Can render `ls`, `cat`, `vim` correctly

#### Stream 1B: Text Rendering Engine
- **Agent Type**: `frontend-developer` or `rust-pro`
- **Primary Files**: `scarab-client/src/rendering/`, `scarab-client/src/main.rs`
- **No Conflicts**: Reads SharedState (already defined)
- **Completion Criteria**: 60 FPS at 200x100 grid

#### Stream 1C: IPC Control Channel
- **Agent Type**: `backend-dev` or `system-architect`
- **Primary Files**: `scarab-daemon/src/ipc.rs`, `scarab-client/src/ipc.rs`
- **No Conflicts**: Independent communication layer
- **Completion Criteria**: Multi-client support, proper resizing

**Coordination Point**: All three converge when testing end-to-end terminal usage.

---

### Phase 2: Plugin System (3 Parallel Streams)

#### Stream 2A: Fusabi VM
- **Agent Type**: `coder` or `performance-engineer`
- **Primary Files**: `crates/fusabi-vm/src/`
- **No Conflicts**: Standalone VM implementation
- **Completion Criteria**: <1ms plugin execution overhead

#### Stream 2B: Fusabi Interpreter
- **Agent Type**: `coder` or `language-specialist`
- **Primary Files**: `crates/fusabi-interpreter/src/`
- **No Conflicts**: Separate from VM
- **Completion Criteria**: <100ms hot-reload

#### Stream 2C: Plugin API
- **Agent Type**: `api-docs` or `backend-architect`
- **Primary Files**: `crates/scarab-plugin-api/src/`, `scarab-daemon/src/plugin_manager.rs`
- **Depends On**: Streams 2A and 2B (integration)
- **Completion Criteria**: 3rd-party plugin template working

**Coordination Point**: Plugin API integrates both VM and Interpreter.

---

## 🔄 Swarm Patterns

### Pattern 1: Mesh Topology (Phase 1)
```bash
npx claude-flow@alpha swarm init --topology mesh
```

- **Use When**: All tasks are independent
- **Benefit**: Maximum parallelism, no bottlenecks
- **Best For**: Phase 1 (VTE, Rendering, IPC)

### Pattern 2: Hierarchical Topology (Phase 2)
```bash
npx claude-flow@alpha swarm init --topology hierarchical
```

- **Use When**: Tasks have dependencies
- **Benefit**: Queen coordinates, workers execute
- **Best For**: Phase 2 (Plugin system integration)

### Pattern 3: Adaptive Topology (Phase 3+)
```bash
npx claude-flow@alpha swarm init --topology adaptive
```

- **Use When**: Mix of dependent/independent tasks
- **Benefit**: Auto-optimizes based on workload
- **Best For**: Phase 3+ (features + hardening)

---

## 🧠 Memory & Context Sharing

### Shared Context
```bash
# Store architecture decisions
npx claude-flow@alpha memory store \
  --key "scarab/architecture/ipc" \
  --value "Unix sockets for control, shared memory for bulk data"

# Retrieve context
npx claude-flow@alpha memory retrieve --key "scarab/architecture/ipc"
```

### Agent Coordination
```bash
# Agent A completes VTE parser
npx claude-flow@alpha hooks post-task \
  --task-id "vte-parser" \
  --memory-key "scarab/phase1/vte-complete"

# Agent B waits for VTE completion
npx claude-flow@alpha hooks pre-task \
  --depends-on "scarab/phase1/vte-complete"
```

---

## 🎯 Task Orchestration

### High-Level Orchestration
```bash
npx claude-flow@alpha task orchestrate \
  --workflow "scarab-phase1" \
  --parallel \
  --tasks "issue-1,issue-2,issue-3"
```

### Sequential Sub-Tasks
```bash
# Issue #2 (Rendering) has internal sequence
npx claude-flow@alpha task orchestrate \
  --workflow "issue-2-rendering" \
  --sequential \
  --tasks "cosmic-text-integration,atlas-creation,mesh-generation"
```

---

## 📈 Performance Monitoring

### Real-Time Metrics
```bash
# Agent metrics
npx claude-flow@alpha agent metrics --session scarab-phase1

# Bottleneck analysis
npx claude-flow@alpha benchmark run --session scarab-phase1

# Neural patterns (learn from success)
npx claude-flow@alpha neural patterns --session scarab-phase1
```

### Export Session Data
```bash
npx claude-flow@alpha hooks session-end \
  --export-metrics true \
  --output "scarab-phase1-metrics.json"
```

---

## 🔧 Conflict Resolution

### File Conflicts
If two agents modify the same file:

1. **Prevention**: Assign non-overlapping files
2. **Detection**: Git pre-commit hooks detect conflicts
3. **Resolution**: Queen agent (orchestrator) merges changes

### Example:
```bash
# Agent A working on daemon/src/main.rs (VTE)
# Agent B working on daemon/src/ipc.rs (IPC)
# No conflict - different files

# But if both touch daemon/src/main.rs:
npx claude-flow@alpha swarm resolve-conflict \
  --agent-a vte-parser \
  --agent-b ipc-control \
  --strategy "merge"  # or "rebase" or "manual"
```

---

## 🎓 Best Practices

### 1. Clear Interfaces
Define contracts between workstreams upfront:
- SharedState layout (already done)
- ControlMessage enum (already done)
- Plugin trait signatures

### 2. Incremental Commits
Each agent commits incrementally:
```bash
git commit -m "feat(vte): parse basic ANSI sequences

- Implement SGR (color) parsing
- Handle cursor movement
- Update SharedState.cells

Issue: #1"
```

### 3. Continuous Integration
Run tests after each commit:
```bash
cargo test --workspace
cargo clippy --all-targets
cargo fmt --check
```

### 4. Communication
Use memory system for coordination:
```bash
# Agent A notifies completion
npx claude-flow@alpha hooks notify \
  --message "VTE parser complete, grid updates working"

# Agent B checks dependencies
npx claude-flow@alpha memory usage --session scarab-phase1
```

---

## 🚨 Emergency Procedures

### Agent Stuck
```bash
# Check agent status
npx claude-flow@alpha agent status --id <agent-id>

# Kill stuck agent
npx claude-flow@alpha agent kill --id <agent-id>

# Respawn with updated context
npx claude-flow@alpha agent spawn --type coder --task "Resume issue #1"
```

### Swarm Deadlock
```bash
# Analyze bottleneck
npx claude-flow@alpha benchmark run --detect-deadlock

# Reset swarm
npx claude-flow@alpha swarm reset --session scarab-phase1

# Restart with different topology
npx claude-flow@alpha swarm init --topology mesh
```

---

## 📊 Example Workflow

### Full Phase 1 Parallel Execution

```bash
# 1. Initialize
npx claude-flow@alpha swarm init --topology mesh --session scarab-phase1

# 2. Spawn all Phase 1 agents in parallel
npx claude-flow@alpha task orchestrate \
  --workflow "phase1" \
  --parallel \
  --tasks "issue-1,issue-2,issue-3" \
  --session scarab-phase1

# 3. Monitor progress
watch -n 5 'npx claude-flow@alpha swarm status --session scarab-phase1'

# 4. Integration testing (after all complete)
npx claude-flow@alpha agent spawn \
  --type tester \
  --task "Run end-to-end tests for Phase 1" \
  --depends-on "issue-1,issue-2,issue-3"

# 5. Export metrics
npx claude-flow@alpha hooks session-end \
  --export-metrics true \
  --session scarab-phase1
```

---

## 🎯 Success Metrics

- **Velocity**: 3x speedup via parallelization
- **Quality**: 80%+ test coverage maintained
- **Conflicts**: <5% file conflicts
- **Completion**: All Phase 1 issues in 1-2 weeks

---

## 📚 Resources

- [Claude-Flow Documentation](https://github.com/ruvnet/claude-flow)
- [Swarm Patterns](https://github.com/ruvnet/claude-flow/blob/main/docs/swarm-patterns.md)
- [Memory System](https://github.com/ruvnet/claude-flow/blob/main/docs/memory.md)

---

**Last Updated**: 2025-11-21
**Version**: 1.0.0
**Status**: 🟢 Ready for Execution
