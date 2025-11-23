# Issue #16: Implement Plugin API Logging & Notifications

**Phase**: 2C - Plugin API
**Priority**: 🟢 Low
**Status**: 📝 **Pending**

## 🐛 Problem
The `scarab-plugin-api` crate contains placeholders in `context.rs`:
- `// TODO: Integrate with actual logging system`
- `// TODO: Implement actual notification system`

Plugins currently cannot log debug info or show user notifications effectively.

## 🎯 Goal
Implement a proper logging bridge and notification system.
1. Define a `Log` trait or use the standard `log` crate facade, but ensure it routes back to the host daemon.
2. Define a `Notification` struct (title, body, level).
3. Implement the IPC/FFI calls to send these events from Plugin -> Daemon -> Client (UI).

## 🛠 Implementation Details
- **Files**: `crates/scarab-plugin-api/src/context.rs`
- **Dependencies**: `scarab-protocol` (may need new message types for Log/Notify).

## ✅ Acceptance Criteria
- [ ] `PluginContext::log(level, msg)` works.
- [ ] `PluginContext::notify(title, body)` works.
- [ ] Events are received by the Daemon/Client.
