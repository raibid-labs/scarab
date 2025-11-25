# Scarab Video Screencasts

Professional video tutorials for Scarab Terminal.

## Video List

1. **Scarab in 2 Minutes** (`scarab-2min-demo.mp4`) - Quick feature showcase
2. **Your First Plugin** (`first-plugin-tutorial.mp4`) - Plugin creation walkthrough
3. **Advanced Workflows** (`advanced-workflows.mp4`) - Power user features

## Recording Setup

### Prerequisites

```bash
# Install asciinema
pip install asciinema

# Install asciicast2mp4 (for video conversion)
npm install -g asciicast2mp4

# Or install agg for simpler conversion
cargo install agg

# Install ffmpeg (for post-processing)
sudo apt install ffmpeg  # Ubuntu/Debian
brew install ffmpeg      # macOS
```

### Recording Guidelines

**Technical Specs:**
- Resolution: 120x30 columns (or 1920x1080px equivalent)
- Frame rate: 30 FPS
- Font: JetBrains Mono 14pt
- Theme: Dracula (consistent across videos)
- Duration: 2-5 minutes per video

**Content Guidelines:**
- Clear, deliberate actions (not too fast)
- Pause 1-2 seconds between actions
- Use on-screen text annotations for clarity
- No mistakes - edit or re-record if needed
- Include intro/outro cards

## Video Scripts

### 1. Scarab in 2 Minutes

**Duration:** 2:00
**File:** `scarab-2min-demo.mp4`

**Script:**

```bash
#!/bin/bash
# record-2min-demo.sh

asciinema rec 2min-demo.cast -c "bash -l" << 'EOF'
# === INTRO (0:00-0:10) ===
clear
cat << 'INTRO'
╔════════════════════════════════════════╗
║                                        ║
║    SCARAB TERMINAL                     ║
║    Next-Gen GPU-Accelerated Terminal   ║
║                                        ║
║    github.com/raibid-labs/scarab       ║
║                                        ║
╚════════════════════════════════════════╝
INTRO
sleep 4
clear

# === GPU RENDERING (0:10-0:30) ===
sleep 1
echo "1. GPU-Accelerated Rendering"
echo "=============================="
sleep 1
echo ""
echo "Scarab uses Bevy game engine for 60+ FPS rendering"
sleep 2

# Run some commands to show smooth rendering
for i in {1..20}; do
    echo "Line $i: Smooth scrolling with GPU acceleration"
    sleep 0.1
done
sleep 2
clear

# === LINK HINTS (0:30-0:50) ===
sleep 1
echo "2. Link Hints"
echo "============="
sleep 1
echo ""
echo "Keyboard-driven URL opening:"
sleep 1
echo "  Documentation: https://github.com/raibid-labs/scarab/docs"
sleep 1
echo "  Issues: https://github.com/raibid-labs/scarab/issues"
sleep 1
echo "  Website: https://scarab.dev"
sleep 1
echo ""
echo "Press Ctrl+Shift+O to activate link hints"
sleep 2
echo "[a] [b] [c] - Press letter to open link"
sleep 3
clear

# === COMMAND PALETTE (0:50-1:10) ===
sleep 1
echo "3. Command Palette"
echo "=================="
sleep 1
echo ""
echo "Press Ctrl+Shift+P for quick command access"
sleep 2
cat << 'PALETTE'

┌───────────────────────────────────┐
│ Command Palette          Ctrl+P   │
├───────────────────────────────────┤
│ >                                 │
│                                   │
│  session new                      │
│  theme switch                     │
│  plugin reload                    │
│  config edit                      │
└───────────────────────────────────┘
PALETTE
sleep 3
clear

# === PLUGINS (1:10-1:35) ===
sleep 1
echo "4. F# Plugin System"
echo "==================="
sleep 1
echo ""
echo "Write plugins in Fusabi (F# for Rust):"
sleep 1
cat << 'PLUGIN'

// hello-plugin.fsx
open Scarab.PluginApi

let on_load ctx = async {
    ctx.Log(Info, "Hello from F#!")
    return Ok ()
}

Plugin.Register {
    Metadata = { Name = "hello"; Version = "1.0.0" }
    OnLoad = Some on_load
}
PLUGIN
sleep 4
clear

# === ARCHITECTURE (1:35-1:50) ===
sleep 1
echo "5. Split-Process Architecture"
echo "=============================="
sleep 1
echo ""
echo "Daemon (server)  + Client (GUI)"
sleep 1
echo "     │                  │"
sleep 1
echo "     └── Zero-Copy IPC ──┘"
sleep 1
echo "         (Shared Memory)"
sleep 2
echo ""
echo "→ Session persistence"
echo "→ Client crashes don't lose state"
echo "→ Multiple clients can attach"
sleep 3
clear

# === OUTRO (1:50-2:00) ===
sleep 1
cat << 'OUTRO'
╔════════════════════════════════════════╗
║                                        ║
║    Get Started:                        ║
║                                        ║
║    git clone github.com/raibid-labs/  ║
║              scarab                    ║
║    cargo build --release               ║
║                                        ║
║    Star us on GitHub!                  ║
║                                        ║
╚════════════════════════════════════════╝
OUTRO
sleep 4
EOF

# Convert to MP4
asciicast2mp4 -t dracula -s 2 2min-demo.cast 2min-demo.mp4
```

### 2. Your First Plugin (5 minutes)

**Duration:** 5:00
**File:** `first-plugin-tutorial.mp4`

**Script:**

```bash
#!/bin/bash
# record-first-plugin.sh

asciinema rec first-plugin.cast -c "bash -l" << 'EOF'
# === INTRO (0:00-0:15) ===
clear
cat << 'INTRO'
╔════════════════════════════════════════╗
║                                        ║
║    YOUR FIRST SCARAB PLUGIN            ║
║    F# Plugin Development Tutorial      ║
║                                        ║
╚════════════════════════════════════════╝
INTRO
sleep 3
clear

# === SETUP (0:15-0:45) ===
sleep 1
echo "Step 1: Create Plugin Directory"
echo "================================"
sleep 1
echo ""
echo "$ mkdir -p ~/.config/scarab/plugins"
mkdir -p ~/.config/scarab/plugins
sleep 2
echo ""
echo "$ cd ~/.config/scarab/plugins"
cd ~/.config/scarab/plugins
sleep 1
echo ""
echo "$ ls"
ls
sleep 2
clear

# === CREATE PLUGIN (0:45-2:30) ===
sleep 1
echo "Step 2: Create Plugin File"
echo "=========================="
sleep 1
echo ""
echo "$ vim hello-world.fsx"
sleep 2
echo ""
echo "Writing plugin code..."
sleep 1

cat > hello-world.fsx << 'PLUGIN'
(*
 * hello-world.fsx - My First Scarab Plugin
 * A simple plugin that demonstrates the basics
 *)

open Scarab.PluginApi

// Plugin metadata
let metadata = {
    Name = "hello-world"
    Version = "1.0.0"
    Description = "My first Scarab plugin"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Called when plugin loads
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Hello, Scarab!")

        // Get terminal size
        let (cols, rows) = ctx.GetSize()
        ctx.Log(LogLevel.Info, sprintf "Terminal: %dx%d" cols rows)

        return Ok ()
    }

// Called when plugin unloads
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Goodbye!")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = Some on_load
    OnUnload = Some on_unload
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
PLUGIN

echo ""
echo "Plugin created!"
sleep 2

echo ""
echo "$ cat hello-world.fsx"
sleep 1
cat hello-world.fsx
sleep 5
clear

# === LOAD PLUGIN (2:30-3:30) ===
sleep 1
echo "Step 3: Load Plugin"
echo "==================="
sleep 1
echo ""
echo "Restart the Scarab daemon to load plugins"
sleep 2
echo ""
echo "$ pkill scarab-daemon"
sleep 1
echo "$ scarab-daemon &"
sleep 2
echo ""
echo "Daemon starting..."
sleep 1
echo "Loading plugins from: ~/.config/scarab/plugins/"
sleep 2
echo ""
echo "[INFO] Plugin 'hello-world' loaded successfully"
echo "[INFO] Hello, Scarab!"
echo "[INFO] Terminal: 120x30"
sleep 3
clear

# === ADVANCED (3:30-4:30) ===
sleep 1
echo "Step 4: Add Functionality"
echo "========================="
sleep 1
echo ""
echo "Let's add a command that shows current time"
sleep 2

cat > hello-world.fsx << 'PLUGIN2'
open Scarab.PluginApi
open System

let metadata = { (* same as before *) }

// Get current time command
let get_current_time (ctx: PluginContext) : Command =
    {
        Name = "current-time"
        Description = "Show current time"
        Handler = fun () -> async {
            let now = DateTime.Now
            let time = now.ToString("HH:mm:ss")
            ctx.Log(LogLevel.Info, sprintf "Current time: %s" time)
            return Ok ()
        }
    }

Plugin.Register {
    (* ... *)
    GetCommands = fun () -> [get_current_time ctx]
}
PLUGIN2

echo ""
echo "Updated plugin!"
sleep 2
echo ""
echo "Now you can run: Ctrl+Shift+P > current-time"
sleep 3
clear

# === OUTRO (4:30-5:00) ===
sleep 1
cat << 'OUTRO'
╔════════════════════════════════════════╗
║                                        ║
║    Plugin Created! 🎉                  ║
║                                        ║
║    Next Steps:                         ║
║    - Explore hook types (OnOutput,     ║
║      OnInput, OnResize, etc.)          ║
║    - Check examples/plugins/           ║
║    - Read plugin API docs              ║
║                                        ║
║    Happy Hacking!                      ║
║                                        ║
╚════════════════════════════════════════╝
OUTRO
sleep 5
EOF

asciicast2mp4 -t dracula -s 2 first-plugin.cast first-plugin-tutorial.mp4
```

### 3. Advanced Workflows (5 minutes)

**Duration:** 5:00
**File:** `advanced-workflows.mp4`

**Script:** See full script in `record-advanced-workflows.sh`

## Post-Processing

### Add Title Cards

Use ffmpeg to add intro/outro:

```bash
# Create title card (1920x1080, 3 seconds)
ffmpeg -f lavfi -i color=c=black:s=1920x1080:d=3 \
       -vf "drawtext=text='Scarab Terminal':fontsize=72:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2" \
       intro.mp4

# Concatenate
ffmpeg -f concat -i <(for f in intro.mp4 main.mp4 outro.mp4; do echo "file '$f'"; done) \
       -c copy final.mp4
```

### Optimize File Size

```bash
# Compress video (target 20MB for GitHub)
ffmpeg -i input.mp4 -vcodec libx264 -crf 28 output.mp4

# Check file size
ls -lh output.mp4
```

### Add Captions

Create an SRT file and embed:

```srt
1
00:00:00,000 --> 00:00:03,000
Scarab Terminal - GPU Accelerated

2
00:00:03,000 --> 00:00:06,000
Built with Rust and Bevy
```

```bash
ffmpeg -i video.mp4 -i subtitles.srt -c copy -c:s mov_text output.mp4
```

## Hosting

### YouTube

Upload to Scarab Terminal YouTube channel:
- Title: "Scarab Terminal: [Topic]"
- Description: Include GitHub link and timestamps
- Tags: terminal, rust, bevy, gpu, f#, fusabi

### GitHub

Embed in README:
```markdown
[![Scarab Demo](https://img.youtube.com/vi/VIDEO_ID/maxresdefault.jpg)](https://www.youtube.com/watch?v=VIDEO_ID)
```

### Self-Hosting

Host on GitHub Releases or project website:
```bash
# Create release with videos
gh release create v0.1.0-alpha \
    --title "Scarab v0.1.0 Alpha" \
    --notes "First alpha release" \
    docs/videos/*.mp4
```

## File Size Limits

- **GitHub:** 100MB per file (use Git LFS for larger)
- **YouTube:** No limit, but keep under 100MB for faster uploads
- **Recommended:** 10-30MB per video

If videos exceed limits:
1. Reduce resolution
2. Increase compression (higher CRF)
3. Shorten duration
4. Split into parts

## Credits

All videos created with:
- `asciinema` - Terminal recording
- `asciicast2mp4` - Video conversion
- `ffmpeg` - Post-processing
- Scarab Terminal - The star of the show!
