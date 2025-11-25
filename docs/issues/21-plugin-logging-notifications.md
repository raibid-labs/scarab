# Issue #21: Implement Plugin Logging and Notifications System

**Phase**: 2C - Plugin API
**Priority**: 🟢 Low
**Status**: 📝 **Pending**

## 🐛 Problem
The `scarab-plugin-api` crate contains placeholder implementations in `context.rs`:
- Line 166: "TODO: Integrate with actual logging system"
- Line 177: "TODO: Implement actual notification system"

Plugins currently cannot effectively log debug information or display notifications to users. The `log()` and `notify()` methods exist but don't actually send data to the daemon or client.

## 🎯 Goal
Implement a proper logging bridge and notification system that allows plugins to:
1. Log messages at different levels (debug, info, warn, error)
2. Send notifications to the client UI
3. Have logs routed from plugin → daemon → client logs
4. Display notifications in the client UI overlay

## 🛠 Implementation Details
- **Files**:
  - `crates/scarab-plugin-api/src/context.rs` - Implement log/notify methods
  - `crates/scarab-protocol/src/lib.rs` - Add Log and Notify message types
  - `crates/scarab-daemon/src/ipc.rs` - Handle log forwarding
  - `crates/scarab-client/src/ui/overlays.rs` - Display notifications

- **Architecture**:
  - Plugin calls `ctx.log(level, message)`
  - Message is sent via channel to daemon
  - Daemon logs to its own log and optionally forwards to client
  - Client displays notifications in UI overlay

- **Message Types**:
  ```rust
  pub enum ControlMessage {
      // ... existing variants ...
      PluginLog { plugin_name: String, level: LogLevel, message: String },
      PluginNotify { title: String, body: String, level: NotifyLevel },
  }
  ```

## ✅ Acceptance Criteria
- [ ] `PluginContext::log(level, msg)` sends logs to daemon
- [ ] `PluginContext::notify(title, body)` sends notifications to client
- [ ] Daemon logs include plugin name prefix
- [ ] Client displays notification overlays
- [ ] Log levels are respected (debug only shown if verbose)
- [ ] Notification UI includes dismiss functionality
- [ ] Both TODO comments are resolved

## 📋 Testing
- Create test plugin that logs at different levels
- Create test plugin that sends notifications
- Verify logs appear in daemon output
- Verify notifications appear in client UI
- Test notification dismiss/timeout behavior
