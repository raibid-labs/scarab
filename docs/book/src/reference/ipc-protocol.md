# IPC Protocol

Scarab uses a hybrid IPC approach combining shared memory and Unix domain sockets.

## Architecture

```
Daemon                          Client
  |                               |
  |------ Shared Memory ---------|  (Bulk data: terminal grid)
  |        (Zero-copy)            |
  |                               |
  |------ Unix Socket ----------|  (Control messages)
  |        (Bidirectional)        |
```

## Shared Memory Layout

Defined in `crates/scarab-protocol/src/lib.rs`.

### SharedState Structure

```rust
#[repr(C)]
pub struct SharedState {
    pub sequence_number: AtomicU64,  // Incremented on each update
    pub width: u32,                  // Terminal width (columns)
    pub height: u32,                 // Terminal height (rows)
    pub cursor_x: u32,               // Cursor X position
    pub cursor_y: u32,               // Cursor Y position
    pub cells: [Cell; 200 * 100],   // Terminal grid
}
```

### Cell Structure

```rust
#[repr(C)]
#[derive(Pod, Zeroable)]
pub struct Cell {
    pub ch: u32,           // Unicode codepoint
    pub fg: u32,           // Foreground color (RGBA)
    pub bg: u32,           // Background color (RGBA)
    pub flags: u8,         // Style flags (bold, italic, etc.)
}
```

## Synchronization Protocol

### Lock-Free Updates

1. **Daemon writes**:
   ```rust
   // Update cells
   state.cells[idx] = new_cell;

   // Increment sequence (release ordering)
   state.sequence_number.fetch_add(1, Ordering::Release);
   ```

2. **Client reads**:
   ```rust
   // Load sequence (acquire ordering)
   let seq = state.sequence_number.load(Ordering::Acquire);

   if seq != last_seq {
       // Read cells (guaranteed consistent)
       let cells = &state.cells;
       // Render...
       last_seq = seq;
   }
   ```

### Memory Ordering Guarantees

- **Release** (daemon): All cell writes visible before sequence increment
- **Acquire** (client): All cell reads see writes before sequence load
- **No locks**: Client never blocks daemon updates

## Unix Socket Protocol

### Socket Location

Default: `/tmp/scarab.sock`

### Message Format

```rust
pub enum ControlMessage {
    // Input events
    KeyPress { key: Key, modifiers: Modifiers },
    MouseEvent { x: u32, y: u32, button: u8 },

    // Window management
    CreateTab,
    CloseTab { id: TabId },
    SwitchTab { id: TabId },

    // Pane management
    CreatePane { parent: PaneId, direction: SplitDirection },
    ClosePane { id: PaneId },
    FocusPane { id: PaneId },

    // Configuration
    Resize { width: u32, height: u32 },
    SetOption { key: String, value: String },

    // Responses
    Ack,
    Error { message: String },
}
```

### Message Encoding

Messages are serialized using `bincode`:

```rust
// Send
let msg = ControlMessage::CreateTab;
let bytes = bincode::serialize(&msg)?;
socket.write_all(&bytes)?;

// Receive
let bytes = read_message(&mut socket)?;
let msg: ControlMessage = bincode::deserialize(&bytes)?;
```

## Critical Constraints

### Memory Safety

1. **`#[repr(C)]`**: All shared structs must have C layout
2. **`Pod + Zeroable`**: Safe for zero-copy transmutation
3. **No pointers**: Shared memory contains only plain data
4. **Fixed size**: No dynamic allocations in shared region

### Performance

1. **Zero-copy**: Client reads directly from shared memory
2. **Lock-free**: Atomic operations only
3. **No syscalls**: Shared memory access is pure memory read
4. **Batching**: Socket messages are batched when possible

## Error Handling

### Shared Memory Errors

- **Creation failure**: Daemon exits, client retries
- **Size mismatch**: Version incompatibility, client refuses to connect
- **Corruption**: Detected via checksum (future enhancement)

### Socket Errors

- **Connection refused**: Daemon not running
- **Broken pipe**: Daemon crashed, client attempts reconnect
- **Timeout**: Client retries with exponential backoff

## Version Compatibility

Shared memory path includes version: `/scarab_shm_v1`

On breaking changes:
1. Increment version number
2. Client checks version before mapping
3. Old clients fail gracefully with clear error

## Security Considerations

### Shared Memory

- Permissions: `0600` (owner only)
- Cleanup: Unlinked on daemon exit
- No sensitive data: Terminal content only

### Socket

- Permissions: `0600` (owner only)
- Authentication: UID-based (Unix domain)
- No network exposure: Local only

## Debugging

### Inspect Shared Memory

```bash
# View shared memory segments
ls -la /dev/shm/scarab_*

# Dump content (binary)
xxd /dev/shm/scarab_shm_v1
```

### Monitor Socket Traffic

```bash
# Test socket connection
nc -U /tmp/scarab.sock

# Monitor with strace
strace -e trace=network scarab-client
```

## Implementation Status

- **Phase 1**: Shared memory layout (Complete)
- **Phase 2**: Atomic synchronization (Complete)
- **Phase 3**: Socket protocol (In Progress)
- **Phase 4**: Error recovery (Planned)

## Related Documentation

- [Architecture Guide](../developer-guide/architecture.md)
- [Testing Guide](../developer-guide/testing.md)
- [Configuration Schema](./config-schema.md)
