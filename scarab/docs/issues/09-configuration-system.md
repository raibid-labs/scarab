# Issue #9: Configuration System

**Phase**: 3C - Advanced Features
**Priority**: 🟢 Medium
**Workstream**: Configuration Management
**Estimated Effort**: 1 week
**Assignee**: Config Management Specialist Agent

---

## 🎯 Objective

Implement a comprehensive configuration system with TOML files, hot-reload, per-shell/per-directory configs, sensible defaults, and validation.

---

## 📋 Background

Users need to customize:
- Font settings
- Color themes
- Key bindings
- Plugin configurations
- Session preferences
- UI behavior

The system should support:
- Global config (~/.config/scarab/config.toml)
- Per-directory overrides (.scarab.toml)
- Environment-specific configs
- Hot-reload without restart

---

## ✅ Acceptance Criteria

- [ ] TOML configuration format
- [ ] Config file discovery (global + local)
- [ ] Hot-reload on file change (<100ms)
- [ ] Per-shell/per-directory configs
- [ ] Sensible defaults (zero-config startup)
- [ ] Config validation with helpful errors
- [ ] Config migration for version updates
- [ ] Documentation of all options
- [ ] Example configs for common use cases
- [ ] Config schema for IDE autocomplete

---

## 🔧 Technical Approach

### Step 1: Config Structure
```toml
# ~/.config/scarab/config.toml

[terminal]
default_shell = "zsh"
scrollback_lines = 10000
alt_screen = true

[font]
family = "JetBrains Mono"
size = 14.0
line_height = 1.2
fallback = ["Fira Code", "DejaVu Sans Mono"]

[colors]
theme = "dracula"
# Or custom:
foreground = "#f8f8f2"
background = "#282a36"
cursor = "#f8f8f2"

[colors.palette]
black = "#21222c"
red = "#ff5555"
# ... 16 colors

[keybindings]
leader_key = "Space"
copy_mode = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"

[ui]
link_hints = true
command_palette = true
animations = true

[plugins]
enabled = ["auto-notify", "git-status"]

[[plugins.config]]
name = "auto-notify"
keywords = ["ERROR", "FAIL"]
```

### Step 2: Config Loading
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ScarabConfig {
    #[serde(default)]
    terminal: TerminalConfig,
    #[serde(default)]
    font: FontConfig,
    #[serde(default)]
    colors: ColorConfig,
    #[serde(default)]
    keybindings: KeyBindings,
    #[serde(default)]
    ui: UiConfig,
    #[serde(default)]
    plugins: PluginConfig,
}

impl ScarabConfig {
    pub fn load() -> Result<Self> {
        let global = Self::load_global()?;
        let local = Self::load_local()?;
        Ok(global.merge(local))
    }

    fn load_global() -> Result<Self> {
        let path = dirs::config_dir()?
            .join("scarab")
            .join("config.toml");
        Self::from_file(&path)
    }

    fn load_local() -> Result<Option<Self>> {
        // Walk up from cwd looking for .scarab.toml
        let mut current = env::current_dir()?;
        loop {
            let config = current.join(".scarab.toml");
            if config.exists() {
                return Ok(Some(Self::from_file(&config)?));
            }
            if !current.pop() { break; }
        }
        Ok(None)
    }
}
```

### Step 3: Hot-Reload
```rust
use notify::{Watcher, RecursiveMode};

pub struct ConfigWatcher {
    watcher: RecommendedWatcher,
    config: Arc<RwLock<ScarabConfig>>,
}

impl ConfigWatcher {
    pub fn new(config: Arc<RwLock<ScarabConfig>>) -> Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;

        watcher.watch(
            dirs::config_dir()?.join("scarab"),
            RecursiveMode::NonRecursive,
        )?;

        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if let Ok(new_config) = ScarabConfig::load() {
                    *config.write().unwrap() = new_config;
                    println!("Config reloaded");
                }
            }
        });

        Ok(Self { watcher, config })
    }
}
```

### Step 4: Validation
```rust
impl ScarabConfig {
    pub fn validate(&self) -> Result<()> {
        if self.font.size < 6.0 || self.font.size > 72.0 {
            return Err(ConfigError::InvalidFontSize(self.font.size));
        }

        if self.terminal.scrollback_lines > 100_000 {
            return Err(ConfigError::ScrollbackTooLarge);
        }

        // Validate color hex codes
        for color in self.colors.palette.iter() {
            if !color.starts_with('#') || color.len() != 7 {
                return Err(ConfigError::InvalidColor(color.clone()));
            }
        }

        Ok(())
    }
}
```

---

## 📦 Deliverables

1. **Code**: `crates/scarab-config/` crate
2. **Schema**: JSON Schema for IDE autocomplete
3. **Defaults**: Complete default config
4. **Examples**: 5+ example configs (themes, layouts, etc.)
5. **Documentation**: Config reference guide
6. **Migration**: Version migration system

---

## 🔗 Dependencies

- **Depends On**: Issue #6 (Plugin API) - for plugin configs
- **Integrates With**: All other components
- **Blocks**: None

---

## 📚 Resources

- [TOML Spec](https://toml.io/)
- [serde TOML](https://docs.rs/toml/)
- [notify File Watcher](https://docs.rs/notify/)
- [Alacritty Config](https://github.com/alacritty/alacritty/blob/master/alacritty.yml)

---

## 🎯 Success Metrics

- ✅ Zero-config startup works
- ✅ Hot-reload <100ms
- ✅ Helpful validation errors
- ✅ Config docs comprehensive
- ✅ Schema enables autocomplete

---

**Created**: 2025-11-21
**Labels**: `phase-3`, `medium-priority`, `configuration`, `toml`
