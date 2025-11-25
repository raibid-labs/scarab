# Issue #20: Implement Fusabi VM Hook Functions

**Phase**: 2B - Plugin Runtime
**Priority**: 🟡 Medium
**Status**: 📝 **Pending**

## 🐛 Problem
The Fusabi VM integration in `scarab-daemon/src/plugin_manager/fusabi_adapter.rs` contains multiple TODO comments indicating that the VM hook functions are not yet implemented:
- Line 93: "TODO: Pass context to VM via host functions"
- Line 101: "TODO: Call VM hook function" (on_output)
- Line 111: "TODO: Call VM hook function" (on_input)
- Line 121: "TODO: Call VM hook function" (on_resize)
- Line 133: "TODO: Execute VM cleanup code"

Currently, plugins can be loaded but their hooks are never actually invoked, making them non-functional.

## 🎯 Goal
Implement the actual Fusabi VM function calls for all plugin lifecycle hooks:
1. Pass `PluginContext` to VM via host functions
2. Implement `on_output` hook invocation
3. Implement `on_input` hook invocation
4. Implement `on_resize` hook invocation
5. Implement `shutdown` cleanup logic

## 🛠 Implementation Details
- **Files**: `crates/scarab-daemon/src/plugin_manager/fusabi_adapter.rs`
- **Dependencies**: `fusabi-vm` crate (already integrated at v0.5.0)
- **Key Changes**:
  - Use VM's `call_function()` method to invoke hooks
  - Serialize PluginContext data and pass via VM stack
  - Handle VM execution errors gracefully
  - Implement proper cleanup in Drop trait

## ✅ Acceptance Criteria
- [ ] `on_output` hook is called when terminal output occurs
- [ ] `on_input` hook is called when user input is received
- [ ] `on_resize` hook is called when terminal is resized
- [ ] `shutdown` is called during plugin unload
- [ ] PluginContext data is accessible within VM
- [ ] Errors are logged and don't crash the daemon
- [ ] All 5 TODO comments are resolved

## 📋 Testing
- Load a test plugin with hook implementations
- Verify hooks are triggered by corresponding events
- Check log output for VM execution traces
- Ensure daemon remains stable with misbehaving plugins
