# Troubleshooting Guide

Solutions to common issues when using Scarab Terminal.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Performance Issues](#performance-issues)
- [Display Issues](#display-issues)
- [Plugin Issues](#plugin-issues)
- [IPC & Connection Issues](#ipc--connection-issues)
- [Configuration Issues](#configuration-issues)
- [Platform-Specific Issues](#platform-specific-issues)

---

## Installation Issues

### Command not found after install

**Symptom**: Running `scarab-daemon` or `scarab-client` gives "command not found"

**Solutions**:

1. **Add Cargo bin to PATH**:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"

   # Reload shell
   source ~/.bashrc  # or source ~/.zshrc
   ```

2. **Install to system location**:
   ```bash
   cargo install --path . --root /usr/local
   # Or with sudo
   sudo cargo install --path . --root /usr
   ```

3. **Use full path**:
   ```bash
   # Find binary location
   find target/release -name "scarab-*" -type f

   # Run with full path
   ./target/release/scarab-daemon
   ./target/release/scarab-client
   ```

4. **Create symlinks**:
   ```bash
   sudo ln -s $(pwd)/target/release/scarab-daemon /usr/local/bin/
   sudo ln -s $(pwd)/target/release/scarab-client /usr/local/bin/
   ```

---

### Permission denied errors

**Symptom**: `Permission denied` when running binaries or accessing files

**Solutions**:

1. **Shared memory permissions**:
   ```bash
   # Check shared memory location
   ls -la /dev/shm/scarab_*

   # Fix ownership
   sudo chown $USER:$USER /dev/shm/scarab_*

   # Or remove and restart
   rm -f /dev/shm/scarab_*
   ```

2. **Socket permissions**:
   ```bash
   # Check socket
   ls -la /tmp/scarab*.sock

   # Fix permissions
   chmod 600 /tmp/scarab*.sock

   # Or remove and restart
   rm -f /tmp/scarab*.sock
   ```

3. **Config directory**:
   ```bash
   # Create config directory
   mkdir -p ~/.config/scarab
   chmod 755 ~/.config/scarab
   ```

4. **PTY permissions** (Linux):
   ```bash
   # Add user to tty group
   sudo usermod -a -G tty $USER

   # Log out and back in for group change to take effect
   ```

---

### Missing dependencies

**Symptom**: Build fails with "cannot find -lfontconfig" or similar

**Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libfontconfig-dev \
    libfreetype6-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev
```

**Fedora/RHEL**:
```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    fontconfig-devel \
    freetype-devel \
    libxcb-devel \
    libxkbcommon-devel \
    openssl-devel
```

**Arch Linux**:
```bash
sudo pacman -S \
    base-devel \
    fontconfig \
    freetype2 \
    libxcb \
    libxkbcommon \
    openssl
```

**Wayland support**:
```bash
# Ubuntu/Debian
sudo apt install libwayland-dev

# Fedora
sudo dnf install wayland-devel

# Arch
sudo pacman -S wayland
```

---

## Performance Issues

### High CPU usage

**Symptom**: Scarab uses excessive CPU (>50% on one core)

**Diagnosis**:
```bash
# Check CPU usage
top -p $(pgrep scarab)

# Profile daemon
RUST_LOG=debug scarab-daemon 2>&1 | grep "frame time"

# Profile client
RUST_LOG=debug scarab-client 2>&1 | grep "render"
```

**Solutions**:

1. **Disable animations**:
   ```toml
   # ~/.config/scarab/config.toml
   [ui]
   animations = false
   smooth_scroll = false
   ```

2. **Reduce frame rate**:
   ```toml
   # Target 30 FPS instead of 60
   [ui]
   target_fps = 30
   ```

3. **Disable cursor blinking**:
   ```toml
   [ui]
   cursor_blink = false
   ```

4. **Reduce scrollback buffer**:
   ```toml
   [terminal]
   scrollback_lines = 5000  # Default: 10000
   ```

5. **Disable problematic plugins**:
   ```toml
   [plugins]
   enabled = []  # Disable all plugins temporarily
   ```

6. **Check for runaway processes**:
   ```bash
   # List plugins
   scarab-daemon --list-plugins

   # Disable specific plugin
   scarab-daemon --disable-plugin git-status
   ```

---

### Stuttering / lag / frame drops

**Symptom**: Rendering is choppy or input feels delayed

**Solutions**:

1. **Enable VSync**:
   ```toml
   [ui]
   vsync = true
   ```

2. **Force GPU backend** (Linux):
   ```bash
   # Try Vulkan
   WGPU_BACKEND=vulkan scarab-client

   # Or OpenGL
   WGPU_BACKEND=gl scarab-client
   ```

3. **Disable compositor** (X11):
   ```bash
   # KDE
   qdbus org.kde.KWin /Compositor suspend

   # Or use picom with --no-vsync
   ```

4. **Increase priority**:
   ```bash
   nice -n -10 scarab-client
   ```

5. **Check GPU drivers**:
   ```bash
   # Verify GPU acceleration
   glxinfo | grep "direct rendering"

   # Should show: direct rendering: Yes

   # Update drivers (Ubuntu)
   sudo ubuntu-drivers autoinstall

   # Update drivers (Arch)
   sudo pacman -S mesa vulkan-radeon  # AMD
   sudo pacman -S nvidia nvidia-utils  # NVIDIA
   ```

6. **Reduce texture atlas size**:
   ```toml
   [font]
   # Use smaller font to reduce atlas
   size = 12.0

   # Reduce fallback fonts
   fallback = ["DejaVu Sans Mono"]
   ```

---

### Memory leaks

**Symptom**: Scarab memory usage grows over time

**Diagnosis**:
```bash
# Monitor memory usage
watch -n 1 'ps aux | grep scarab'

# Check for leaks with valgrind
valgrind --leak-check=full scarab-daemon

# Or use heaptrack
heaptrack scarab-client
heaptrack_gui heaptrack.scarab-client.*
```

**Solutions**:

1. **Limit scrollback**:
   ```toml
   [terminal]
   scrollback_lines = 5000
   ```

2. **Don't save scrollback in sessions**:
   ```toml
   [sessions]
   save_scrollback = false
   ```

3. **Restart daemon periodically**:
   ```bash
   # Add to crontab
   0 */6 * * * systemctl --user restart scarab-daemon
   ```

4. **Clear cache**:
   ```bash
   rm -rf ~/.cache/scarab/
   ```

5. **Update to latest version**:
   ```bash
   cd scarab
   git pull
   cargo build --release
   ```

6. **Report bug** if issue persists:
   ```bash
   # Collect debug info
   RUST_LOG=trace scarab-daemon > daemon.log 2>&1 &
   RUST_LOG=trace scarab-client > client.log 2>&1 &

   # Wait for memory to grow, then:
   kill -SIGUSR1 $(pgrep scarab-daemon)  # Dumps heap

   # Submit logs and heap dump to GitHub
   ```

---

### GPU not being used

**Symptom**: High CPU usage, `nvidia-smi` shows 0% GPU utilization

**Diagnosis**:
```bash
# Check which backend is used
RUST_LOG=wgpu=debug scarab-client 2>&1 | grep backend

# List available adapters
RUST_LOG=wgpu=info scarab-client 2>&1 | grep adapter
```

**Solutions**:

1. **Force GPU backend**:
   ```bash
   # NVIDIA
   WGPU_BACKEND=vulkan scarab-client

   # AMD
   WGPU_BACKEND=vulkan scarab-client

   # Intel
   WGPU_BACKEND=vulkan scarab-client
   ```

2. **Check driver installation**:
   ```bash
   # NVIDIA
   nvidia-smi

   # Should show GPU info, not error

   # Install proprietary drivers (Ubuntu)
   sudo ubuntu-drivers install

   # Arch
   sudo pacman -S nvidia nvidia-utils
   ```

3. **Verify Vulkan support**:
   ```bash
   vulkaninfo | grep deviceName

   # Install Vulkan (Ubuntu)
   sudo apt install vulkan-tools libvulkan1

   # Arch
   sudo pacman -S vulkan-tools vulkan-icd-loader
   ```

4. **Check power management**:
   ```bash
   # NVIDIA - disable power saving
   nvidia-settings -a "[gpu:0]/GPUPowerMizerMode=1"

   # Add to config
   [ui]
   force_discrete_gpu = true
   ```

5. **Wayland issues**:
   ```bash
   # Use X11 session instead
   # Or set environment
   export GBM_BACKEND=nvidia-drm
   export __GLX_VENDOR_LIBRARY_NAME=nvidia
   ```

---

## Display Issues

### Fonts not rendering correctly

**Symptom**: Characters appear blocky, missing, or incorrect

**Solutions**:

1. **Verify font installation**:
   ```bash
   # List installed fonts
   fc-list | grep -i "JetBrains"

   # Install JetBrains Mono (Ubuntu)
   sudo apt install fonts-jetbrains-mono

   # Arch
   sudo pacman -S ttf-jetbrains-mono

   # Manual install
   mkdir -p ~/.local/share/fonts
   cd ~/.local/share/fonts
   wget https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip
   unzip JetBrainsMono-2.304.zip
   fc-cache -f -v
   ```

2. **Use exact font name**:
   ```bash
   # Get exact font name
   fc-list | grep -i jetbrains

   # Use in config
   [font]
   family = "JetBrains Mono"  # Exact name from fc-list
   ```

3. **Add fallback fonts**:
   ```toml
   [font]
   family = "JetBrains Mono"
   fallback = [
       "DejaVu Sans Mono",
       "Noto Sans Mono",
       "Liberation Mono",
       "Noto Color Emoji"  # For emoji
   ]
   ```

4. **Rebuild font cache**:
   ```bash
   fc-cache -f -v
   ```

5. **Check font rendering settings**:
   ```toml
   [font]
   use_thin_strokes = false  # Try toggling
   line_height = 1.2         # Adjust if characters overlap
   ```

---

### Colors appear wrong

**Symptom**: Colors don't match theme, appear washed out, or too dark

**Solutions**:

1. **Verify theme**:
   ```toml
   [colors]
   theme = "dracula"  # Check spelling
   ```

2. **Check color depth**:
   ```bash
   # Verify 24-bit color support
   echo $COLORTERM
   # Should show: truecolor or 24bit

   # Test colors
   curl -s https://gist.githubusercontent.com/lifepillar/09a44b8cf0f9397465614e622979107f/raw/24-bit-color.sh | bash
   ```

3. **Disable transparency**:
   ```toml
   [colors]
   opacity = 1.0  # Fully opaque
   ```

4. **Force color mode**:
   ```bash
   export COLORTERM=truecolor
   scarab-client
   ```

5. **Test with custom colors**:
   ```toml
   [colors]
   theme = null  # Disable theme
   foreground = "#ffffff"
   background = "#000000"

   [colors.palette]
   red = "#ff0000"
   green = "#00ff00"
   blue = "#0000ff"
   # ... etc
   ```

6. **Check compositor** (Linux):
   ```bash
   # Some compositors affect colors
   # Try without compositor
   killall picom  # or compton
   ```

---

### Ligatures not working

**Symptom**: Font ligatures (e.g., `->`, `=>`, `!=`) don't combine

**Solutions**:

1. **Use font with ligature support**:
   ```toml
   [font]
   # Fonts with ligatures:
   family = "JetBrains Mono"  # ✅
   # family = "Fira Code"     # ✅
   # family = "Cascadia Code" # ✅
   # family = "Monospace"     # ❌ No ligatures
   ```

2. **Verify ligatures in font**:
   ```bash
   # Check if font has ligatures
   fc-query "$(fc-match "JetBrains Mono" file | cut -d: -f2)" | grep liga
   ```

3. **Enable ligatures** (should be default):
   ```toml
   [font]
   enable_ligatures = true
   ```

4. **Check HarfBuzz version**:
   ```bash
   # cosmic-text requires HarfBuzz 2.6.4+
   pkg-config --modversion harfbuzz

   # Update if needed (Ubuntu)
   sudo apt update && sudo apt upgrade libharfbuzz-dev
   ```

5. **Known limitation**: Ligatures may not work in all contexts. This is a cosmic-text limitation being actively worked on.

---

### Emoji not displaying

**Symptom**: Emoji show as boxes or missing characters

**Solutions**:

1. **Install emoji font**:
   ```bash
   # Ubuntu/Debian
   sudo apt install fonts-noto-color-emoji

   # Fedora
   sudo dnf install google-noto-emoji-fonts

   # Arch
   sudo pacman -S noto-fonts-emoji
   ```

2. **Add to fallback**:
   ```toml
   [font]
   fallback = [
       "Noto Color Emoji",
       "Apple Color Emoji",
       "Segoe UI Emoji"
   ]
   ```

3. **Update font cache**:
   ```bash
   fc-cache -f -v
   ```

4. **Test emoji support**:
   ```bash
   echo "😀 🚀 🔥 ⚡ 💻"
   ```

5. **Check fontconfig**:
   ```bash
   # Verify emoji font is found
   fc-match "emoji"
   # Should show: NotoColorEmoji.ttf or similar
   ```

---

## Plugin Issues

### Plugins not loading

**Symptom**: Plugins in `enabled` list don't appear to run

**Diagnosis**:
```bash
# List loaded plugins
scarab-daemon --list-plugins

# Check plugin logs
tail -f ~/.local/share/scarab/scarab.log | grep plugin
```

**Solutions**:

1. **Verify plugin location**:
   ```bash
   ls -la ~/.config/scarab/plugins/

   # Should contain .fsx or .fzb files
   ```

2. **Check plugin syntax**:
   ```bash
   # Validate .fsx syntax
   fusabi-check ~/.config/scarab/plugins/my-plugin.fsx
   ```

3. **Enable plugin explicitly**:
   ```toml
   [plugins]
   enabled = ["my-plugin"]  # Without .fsx extension
   ```

4. **Check permissions**:
   ```bash
   chmod 644 ~/.config/scarab/plugins/*.fsx
   ```

5. **View plugin errors**:
   ```bash
   # Run with debug logging
   RUST_LOG=scarab_plugin_api=debug scarab-daemon
   ```

6. **Test minimal plugin**:
   ```fsharp
   // test.fsx
   open Scarab.PluginApi

   let metadata = {
       Name = "test"
       Version = "1.0.0"
       Description = "Test plugin"
       Author = "Me"
       Homepage = None
       ApiVersion = "0.1.0"
       MinScarabVersion = "0.1.0"
   }

   let on_load ctx =
       async {
           ctx.Log(LogLevel.Info, "Test plugin loaded!")
           return Ok ()
       }

   Plugin.Register {
       Metadata = metadata
       OnLoad = on_load
       OnUnload = None
       OnOutput = None
       OnInput = None
       OnResize = None
       OnAttach = None
       OnDetach = None
       OnPreCommand = None
       OnPostCommand = None
       OnRemoteCommand = None
       GetCommands = fun () -> []
   }
   ```

---

### Plugin errors in logs

**Symptom**: Plugin loads but shows errors in logs

**Common errors**:

1. **"API version mismatch"**:
   ```toml
   # Plugin requires newer Scarab version
   # Solution: Update Scarab or downgrade plugin
   ```

2. **"Symbol not found"**:
   ```fsharp
   // Using removed/renamed API function
   // Solution: Update plugin to current API
   // Check API docs: cargo doc --open -p scarab-plugin-api
   ```

3. **"Type error"**:
   ```fsharp
   // Incorrect type passed to API function
   // Solution: Check API signature
   // Example fix:
   // ctx.Log(LogLevel.Info, "message")  // Correct
   // ctx.Log("Info", "message")         // Wrong - string instead of enum
   ```

4. **"Async error"**:
   ```fsharp
   // Not using async properly
   // Solution: Wrap in async { }
   let on_load ctx =
       async {  // Required
           // ... your code
           return Ok ()
       }
   ```

---

### Fusabi compilation failures

**Symptom**: `.fsx` plugin shows compilation errors

**Solutions**:

1. **Check syntax**:
   ```bash
   fusabi-check plugin.fsx
   ```

2. **Common syntax errors**:
   ```fsharp
   // Missing async wrapper
   let on_load ctx = async { return Ok () }  // ✅
   let on_load ctx = Ok ()                   // ❌

   // Wrong return type
   return Ok ()        // ✅
   return ()           // ❌
   return Ok            // ❌

   // Incorrect Option syntax
   Homepage = Some "url"    // ✅
   Homepage = "url"         // ❌
   Homepage = None          // ✅
   Homepage = null          // ❌
   ```

3. **Update Fusabi**:
   ```bash
   cargo install fusabi-cli --force
   ```

4. **Check API imports**:
   ```fsharp
   open Scarab.PluginApi  // Required
   ```

5. **Enable detailed errors**:
   ```bash
   FUSABI_VERBOSE=1 scarab-daemon
   ```

---

## IPC & Connection Issues

### Daemon not starting

**Symptom**: `scarab-daemon` exits immediately or shows errors

**Diagnosis**:
```bash
# Run with verbose logging
RUST_LOG=debug scarab-daemon

# Check for error messages
journalctl --user -u scarab-daemon  # If using systemd
```

**Solutions**:

1. **Check shared memory**:
   ```bash
   # Remove stale shared memory
   rm -f /dev/shm/scarab_*

   # Verify permissions
   ls -la /dev/shm/
   ```

2. **Check socket**:
   ```bash
   # Remove stale socket
   rm -f /tmp/scarab*.sock

   # Ensure /tmp is writable
   ls -ld /tmp
   ```

3. **Check port conflicts**:
   ```bash
   # If using TCP socket
   netstat -tuln | grep 7800

   # Kill conflicting process
   lsof -ti:7800 | xargs kill -9
   ```

4. **Verify dependencies**:
   ```bash
   ldd target/release/scarab-daemon
   # Should not show "not found"
   ```

5. **Check disk space**:
   ```bash
   df -h ~/.local/share/scarab/
   # Need space for session database
   ```

6. **Run in foreground**:
   ```bash
   # Don't daemonize for debugging
   scarab-daemon --foreground
   ```

---

### Client can't connect to daemon

**Symptom**: `scarab-client` shows "Connection refused" or "Daemon not found"

**Solutions**:

1. **Verify daemon is running**:
   ```bash
   ps aux | grep scarab-daemon
   # Should show running process

   # Or check socket
   ls -la /tmp/scarab*.sock
   ```

2. **Check socket path**:
   ```bash
   # Daemon socket location
   RUST_LOG=debug scarab-daemon | grep socket

   # Client connecting to
   RUST_LOG=debug scarab-client | grep socket

   # Paths must match!
   ```

3. **Force socket path**:
   ```toml
   # Both daemon and client config
   [ipc]
   socket_path = "/tmp/scarab.sock"
   ```

4. **Check permissions**:
   ```bash
   ls -la /tmp/scarab*.sock
   # Should be owned by current user

   chmod 600 /tmp/scarab*.sock
   ```

5. **Restart daemon**:
   ```bash
   killall scarab-daemon
   scarab-daemon
   # Wait 2 seconds
   scarab-client
   ```

6. **Use TCP instead** (testing):
   ```toml
   [ipc]
   mode = "tcp"
   address = "127.0.0.1:7800"
   ```

---

### Shared memory errors

**Symptom**: "Failed to map shared memory" or "Shared memory full"

**Solutions**:

1. **Increase shared memory size**:
   ```bash
   # Check current size
   df -h /dev/shm

   # Increase (temporary)
   sudo mount -o remount,size=512M /dev/shm

   # Permanent (add to /etc/fstab)
   tmpfs /dev/shm tmpfs defaults,size=512M 0 0
   ```

2. **Clean stale segments**:
   ```bash
   # List shared memory
   ipcs -m

   # Remove by ID
   ipcrm -m <shmid>

   # Or remove all Scarab segments
   rm -f /dev/shm/scarab_*
   ```

3. **Reduce buffer size**:
   ```toml
   [ipc]
   shm_size = 8  # Default: 16 MB
   ```

4. **Check limits**:
   ```bash
   # View SHM limits
   sysctl kernel.shmmax
   sysctl kernel.shmall

   # Increase if needed
   sudo sysctl -w kernel.shmmax=536870912  # 512 MB
   sudo sysctl -w kernel.shmall=131072     # Pages
   ```

---

## Configuration Issues

### Config not loading

**Symptom**: Changes to `config.toml` have no effect

**Solutions**:

1. **Check file location**:
   ```bash
   # Should be in:
   ~/.config/scarab/config.toml

   # Not:
   ~/.scarab.toml
   ~/config.toml
   ~/.config/scarab.toml
   ```

2. **Validate TOML syntax**:
   ```bash
   # Install TOML validator
   cargo install taplo-cli

   # Check syntax
   taplo lint ~/.config/scarab/config.toml
   ```

3. **Check for errors**:
   ```bash
   RUST_LOG=scarab_config=debug scarab-daemon
   # Look for "Config error" messages
   ```

4. **Test minimal config**:
   ```toml
   # Rename current config
   mv ~/.config/scarab/config.toml ~/.config/scarab/config.toml.bak

   # Create minimal
   [font]
   size = 16.0

   # Test
   scarab-daemon --validate-config
   ```

5. **View active config**:
   ```bash
   scarab-daemon --print-config
   # Shows merged config from all sources
   ```

---

### Hot reload not working

**Symptom**: Config changes require restart to apply

**Solutions**:

1. **Wait for debounce**:
   ```bash
   # Changes applied after 100ms
   # Make edit, wait 1 second
   ```

2. **Check file watcher**:
   ```bash
   # Linux: Verify inotify works
   inotifywait -m ~/.config/scarab/config.toml
   # Edit file, should see event
   ```

3. **Increase inotify limits** (Linux):
   ```bash
   # Check current limit
   cat /proc/sys/fs/inotify/max_user_watches

   # Increase temporarily
   sudo sysctl fs.inotify.max_user_watches=524288

   # Permanent (add to /etc/sysctl.conf)
   fs.inotify.max_user_watches=524288
   ```

4. **Force reload**:
   ```bash
   # Send SIGHUP to daemon
   killall -HUP scarab-daemon

   # Or use command palette
   # Ctrl+Shift+P -> "Reload Configuration"
   ```

5. **Restart if needed**:
   ```bash
   # Some settings require restart (e.g., terminal size)
   # Check logs for "Restart required" message
   ```

---

## Platform-Specific Issues

### Linux: Wayland black screen

**Symptom**: Client window opens but shows black screen

**Solutions**:

1. **Force X11**:
   ```bash
   GDK_BACKEND=x11 scarab-client
   ```

2. **Update Mesa drivers**:
   ```bash
   # Ubuntu
   sudo add-apt-repository ppa:kisak/kisak-mesa
   sudo apt update && sudo apt upgrade

   # Arch
   sudo pacman -Syu mesa
   ```

3. **Enable Vulkan for Wayland**:
   ```bash
   export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json  # AMD
   # Or
   export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json  # NVIDIA
   ```

4. **Check compositor**:
   ```bash
   # GNOME: Ensure using Wayland session
   echo $XDG_SESSION_TYPE
   # Should show: wayland

   # Try Sway instead of GNOME Shell
   ```

---

### macOS: Gatekeeper blocks binary

**Symptom**: "scarab-client cannot be opened because the developer cannot be verified"

**Solutions**:

1. **Allow in System Preferences**:
   ```bash
   # Right-click binary, choose "Open"
   # Or:
   xattr -d com.apple.quarantine target/release/scarab-client
   xattr -d com.apple.quarantine target/release/scarab-daemon
   ```

2. **Sign binary** (developers):
   ```bash
   codesign --force --deep --sign - target/release/scarab-client
   ```

---

### Getting Help

If issues persist:

1. **Check logs**:
   ```bash
   tail -f ~/.local/share/scarab/scarab.log
   ```

2. **Enable verbose logging**:
   ```bash
   RUST_LOG=trace scarab-daemon > daemon.log 2>&1
   RUST_LOG=trace scarab-client > client.log 2>&1
   ```

3. **Report bug on GitHub**:
   - Include OS/distro version
   - Attach logs (daemon.log, client.log)
   - Describe steps to reproduce
   - Mention config settings

   https://github.com/raibid-labs/scarab/issues

4. **Join discussions**:
   https://github.com/raibid-labs/scarab/discussions

---

## See Also

- [Configuration Reference](./configuration.md) - All config options
- [Performance Tuning](./performance.md) - Optimization guide
- [FAQ](./faq.md) - Common questions
