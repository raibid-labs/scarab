# CLI Bootstrap Pattern

This document describes the standardized CLI bootstrap pattern used across Scarab binaries.

## Overview

All Scarab binaries (daemon, client, plugin-compiler) follow a consistent bootstrap pattern:

1. **Parse CLI Arguments** - Use clap for consistent argument parsing
2. **Initialize Logging** - Set up env_logger or similar
3. **Load Configuration** - Read config files with fallbacks
4. **Validate Environment** - Check dependencies and permissions
5. **Initialize Runtime** - Set up tokio/bevy runtime
6. **Run Application** - Execute main application logic
7. **Graceful Shutdown** - Clean up resources

## Standard Bootstrap Flow

### 1. CLI Argument Parsing

All binaries use `clap` with derive macros:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "scarab-daemon")]
#[command(about = "Headless daemon server for Scarab terminal emulator")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Enable profiling
    #[arg(long)]
    profile: bool,
}
```

### 2. Logging Initialization

Standard env_logger setup:

```rust
fn init_logging(log_level: &str) -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(log_level)
    )
    .format_timestamp_millis()
    .init();

    log::info!("Logging initialized at level: {}", log_level);
    Ok(())
}
```

### 3. Configuration Loading

Standardized config loading with fallbacks:

```rust
fn load_config(config_path: Option<PathBuf>) -> Result<Config> {
    let config = match config_path {
        Some(path) => {
            log::info!("Loading config from: {}", path.display());
            Config::from_file(&path)?
        }
        None => {
            let default_path = Config::default_path()?;
            if default_path.exists() {
                log::info!("Loading default config from: {}", default_path.display());
                Config::from_file(&default_path)?
            } else {
                log::info!("Using default configuration");
                Config::default()
            }
        }
    };

    log::debug!("Configuration loaded: {:?}", config);
    Ok(config)
}
```

### 4. Environment Validation

Check prerequisites before starting:

```rust
fn validate_environment() -> Result<()> {
    // Check required directories
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not determine config directory"))?;

    if !config_dir.exists() {
        log::warn!("Config directory does not exist: {}", config_dir.display());
    }

    // Check permissions
    // Check dependencies
    // etc.

    Ok(())
}
```

### 5. Runtime Initialization

For async binaries (daemon):

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logging(&cli.log_level)?;
    let config = load_config(cli.config)?;
    validate_environment()?;

    // Run daemon
    run_daemon(config).await
}
```

For Bevy binaries (client):

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logging(&cli.log_level)?;
    let config = load_config(cli.config)?;
    validate_environment()?;

    // Build Bevy app
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .run();

    Ok(())
}
```

### 6. Graceful Shutdown

Handle SIGTERM/SIGINT:

```rust
use tokio::signal;

async fn run_daemon(config: Config) -> Result<()> {
    // Set up shutdown signal
    let shutdown = signal::ctrl_c();

    tokio::select! {
        result = daemon_main_loop(config) => {
            log::info!("Daemon exited: {:?}", result);
            result
        }
        _ = shutdown => {
            log::info!("Received shutdown signal");
            // Clean up
            Ok(())
        }
    }
}
```

## Bootstrap Checklist

For each new binary, ensure:

- [ ] Uses clap for CLI argument parsing
- [ ] Initializes logging early in main()
- [ ] Loads configuration with fallbacks
- [ ] Validates environment before running
- [ ] Handles graceful shutdown
- [ ] Returns proper exit codes
- [ ] Logs important events
- [ ] Documents CLI arguments

## Performance Considerations

### Startup Time

- Keep bootstrap fast (<100ms for daemons, <500ms for UI)
- Defer heavy initialization until after startup
- Use lazy loading where possible
- Profile startup with `--profile` flag

### Resource Usage

- Check available memory before allocating large buffers
- Validate disk space before writing
- Close file handles promptly
- Clean up on shutdown

## Example: scarab-daemon Bootstrap

```rust
use anyhow::Result;
use clap::Parser;
use log::{info, error};

#[derive(Parser)]
#[command(name = "scarab-daemon")]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI
    let cli = Cli::parse();

    // 2. Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&cli.log_level)
    ).init();

    info!("Scarab daemon starting...");

    // 3. Load configuration
    let config = load_config(cli.config)?;

    // 4. Validate environment
    validate_environment()?;

    // 5. Initialize runtime
    let runtime = init_daemon_runtime(config).await?;

    // 6. Run application
    info!("Daemon initialized, running main loop");
    run_daemon_loop(runtime).await?;

    // 7. Graceful shutdown
    info!("Daemon shutting down gracefully");
    Ok(())
}
```

## Common Patterns

### Config File Resolution

```rust
fn resolve_config_path(explicit: Option<PathBuf>) -> PathBuf {
    explicit
        .or_else(|| std::env::var("SCARAB_CONFIG").ok().map(PathBuf::from))
        .or_else(|| {
            dirs::config_dir().map(|d| d.join("scarab/config.toml"))
        })
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}
```

### Feature Flags

```rust
#[cfg(feature = "profiling")]
fn init_profiling() -> Result<()> {
    tracy_client::Client::start();
    Ok(())
}

#[cfg(not(feature = "profiling"))]
fn init_profiling() -> Result<()> {
    Ok(())
}
```

## Testing Bootstrap

Unit tests for bootstrap logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = load_config(None).unwrap();
        assert!(config.is_valid());
    }

    #[test]
    fn test_environment_validation() {
        assert!(validate_environment().is_ok());
    }
}
```

## Related Documentation

- [Configuration System](../crates/scarab-config/README.md)
- [Logging Guidelines](./LOGGING.md) (TBD)
- [Testing Guide](../TESTING.md)
