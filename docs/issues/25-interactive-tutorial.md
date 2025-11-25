# Issue #25: Interactive Tutorial and Onboarding

## 🎯 Goal
Create comprehensive interactive tutorials and visual documentation to onboard new users to Scarab Terminal.

## 🐛 Problem
Current documentation gaps:
- ❌ No "Your First 5 Minutes" walkthrough
- ❌ No step-by-step plugin creation tutorial
- ❌ No video/screencast demos
- ❌ No animated GIFs showing features in action
- ⚠️ Assumes users are familiar with terminal concepts

New users struggle to:
- Understand what makes Scarab unique
- Get started quickly
- Create their first plugin
- Discover advanced features

## 💡 Proposed Solution

Create a multi-layered onboarding experience:

### 1. Interactive In-Terminal Tutorial
**Location**: `crates/scarab-client/src/tutorial/mod.rs` (new module)

A guided tour that launches on first run:
- Introduces key features with real examples
- Interactive steps (user must complete each step)
- Progress tracking
- Can be replayed anytime with `scarab --tutorial`

### 2. Video Screencasts
**Location**: `docs/videos/`

Three essential videos:
1. **"Scarab in 2 Minutes"** - Quick demo of key features
2. **"Your First Plugin"** - Step-by-step plugin creation
3. **"Advanced Workflows"** - Power user features

### 3. Animated GIF Demos
**Location**: `docs/assets/demos/`

Create GIFs for:
- Link hints feature
- Plugin installation
- Command palette
- Theme switching
- Split panes

### 4. Step-by-Step Written Tutorials
**Location**: `docs/tutorials/`

Comprehensive guides:
- Getting Started (0 to productive in 5 minutes)
- Plugin Development (hello world to complex plugin)
- Customization (themes, keybindings, config)
- Integration (Git workflows, Docker, SSH)

## 📋 Implementation Tasks

### Phase 1: Interactive Tutorial System (2 days)

#### Create Tutorial Framework
**File**: `crates/scarab-client/src/tutorial/mod.rs`

```rust
pub struct TutorialSystem {
    current_step: usize,
    steps: Vec<TutorialStep>,
    state: TutorialState,
}

pub struct TutorialStep {
    id: String,
    title: String,
    description: String,
    instruction: String,
    validation: Box<dyn Fn(&TerminalState) -> bool>,
    hint: Option<String>,
}

pub enum TutorialState {
    NotStarted,
    InProgress { step: usize },
    Completed,
    Skipped,
}
```

#### Tutorial Steps
1. **Welcome**: Show splash screen, explain Scarab
2. **Basic Navigation**: Type a command, press Enter
3. **Scrollback**: Use mouse wheel to scroll
4. **Link Hints**: Show URL, trigger link hints (Ctrl+Shift+O)
5. **Command Palette**: Open palette (Ctrl+Shift+P)
6. **Plugins**: Show plugin list, explain plugin system
7. **Configuration**: Show config file location
8. **Completion**: Summary + next steps

#### Visual Design
```
┌─────────────────────────────────────────────────────┐
│                   SCARAB TUTORIAL                   │
│                   Step 2 of 8                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  BASIC NAVIGATION                                   │
│                                                     │
│  Try typing a command and pressing Enter:          │
│                                                     │
│  $ ls -la                                           │
│    ▌                                                │
│                                                     │
│  Hint: Just type any command to continue            │
│                                                     │
│  [Skip Tutorial]              [Next: Scrollback →] │
└─────────────────────────────────────────────────────┘
```

### Phase 2: Video Production (1 day)

#### Recording Setup
- Tool: `asciinema` for terminal recording
- Editor: `asciinema-edit` for trimming
- Export: Convert to GIF with `agg` or MP4 with `asciicast2mp4`

#### Video 1: "Scarab in 2 Minutes"
**Script**:
1. Launch Scarab (0:00-0:05)
2. Run some commands, show GPU rendering (0:05-0:20)
3. Trigger link hints on URL (0:20-0:35)
4. Open command palette (0:35-0:50)
5. Show plugin in action (0:50-1:10)
6. Quick theme change (1:10-1:25)
7. Show split panes (1:25-1:45)
8. Closing + call to action (1:45-2:00)

#### Video 2: "Your First Plugin" (5 minutes)
**Script**:
1. Create plugin directory
2. Write simple `.fsx` script
3. Compile to `.fzb`
4. Load in Scarab
5. Test plugin functionality
6. Iterate and improve

#### Video 3: "Advanced Workflows" (5 minutes)
**Script**:
1. Git integration workflow
2. Docker container management
3. SSH session handling
4. Custom keybindings
5. Theme customization

### Phase 3: Animated GIFs (half day)

Create GIFs for README and docs:

1. **`link-hints-demo.gif`**: URL highlighting + opening
2. **`command-palette.gif`**: Opening palette, fuzzy search
3. **`plugin-install.gif`**: Installing and loading plugin
4. **`theme-switch.gif`**: Switching themes in real-time
5. **`split-panes.gif`**: Creating and navigating splits

Tools:
- Record: `asciinema record demo.cast`
- Convert: `agg demo.cast demo.gif --theme monokai`

### Phase 4: Written Tutorials (1 day)

#### Tutorial 1: Getting Started
**File**: `docs/tutorials/01-getting-started.md`

Sections:
1. Installation (3 methods)
2. First launch
3. Basic usage
4. Configuration basics
5. Next steps

#### Tutorial 2: Customization Guide
**File**: `docs/tutorials/02-customization.md`

Sections:
1. Configuration file structure
2. Changing themes
3. Custom keybindings
4. Font configuration
5. GPU settings
6. Plugin configuration

#### Tutorial 3: Workflow Integration
**File**: `docs/tutorials/03-workflows.md`

Real-world examples:
1. **Git Workflow**: Show git status plugin, commit helpers
2. **Docker Development**: Container management, log viewing
3. **SSH Sessions**: Connection management, tmux alternative
4. **Remote Development**: VS Code integration tips

## 🎨 Visual Assets Needed

### Screenshots
- `assets/screenshots/scarab-main.png` - Clean terminal view
- `assets/screenshots/link-hints.png` - Link hints in action
- `assets/screenshots/command-palette.png` - Palette open
- `assets/screenshots/themes.png` - Theme comparison grid

### Diagrams
- `assets/diagrams/architecture.svg` - System architecture
- `assets/diagrams/plugin-flow.svg` - Plugin execution flow
- `assets/diagrams/ipc-layout.svg` - IPC communication

### Animated GIFs
- `assets/demos/quick-demo.gif` - 30-second feature showcase
- `assets/demos/plugin-demo.gif` - Plugin creation process
- `assets/demos/link-hints.gif` - Link hints feature
- `assets/demos/palette.gif` - Command palette usage

## 🧪 Testing

### Tutorial System Tests
```rust
#[test]
fn test_tutorial_progression() {
    let mut tutorial = TutorialSystem::new();
    assert_eq!(tutorial.current_step(), 0);

    tutorial.complete_step(0);
    assert_eq!(tutorial.current_step(), 1);

    tutorial.skip_tutorial();
    assert!(tutorial.is_skipped());
}

#[test]
fn test_first_launch_detection() {
    let config = Config::load();
    assert!(config.is_first_launch());

    config.mark_tutorial_seen();
    assert!(!config.is_first_launch());
}
```

### Manual Testing
- [ ] Launch tutorial on fresh install
- [ ] Complete each step successfully
- [ ] Test skip functionality
- [ ] Verify tutorial can be replayed
- [ ] Check all GIFs load in README

## 📊 Success Criteria

- [ ] Interactive tutorial completes in < 5 minutes
- [ ] All 3 videos produced and hosted
- [ ] 5+ animated GIFs showing features
- [ ] 3+ written tutorials published
- [ ] README.md updated with visual demos
- [ ] New users report easier onboarding (survey)

## 🔗 Related Issues

- Issue #26: Complete Reference Documentation (complementary)
- Issue #27: Plugin Development Documentation (plugin tutorial component)

---

**Priority**: 🟡 HIGH
**Effort**: 3 days
**Assignee**: Visual Storyteller + Frontend Developer
