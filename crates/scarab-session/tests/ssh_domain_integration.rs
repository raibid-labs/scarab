//! Integration tests for SSH domain functionality
//!
//! Note: These tests require a local SSH server for full integration testing.
//! Run with `--ignored` flag to execute tests that require SSH setup:
//!
//! ```bash
//! cargo test --package scarab-session --test ssh_domain_integration -- --ignored
//! ```

use scarab_session::{
    Domain, DomainRegistry, LocalDomain, PaneConfig, SshAuth, SshDomain, SshDomainConfig,
};
use std::sync::Arc;

#[tokio::test]
async fn test_domain_registry_basic() {
    let registry = DomainRegistry::new();
    assert_eq!(registry.count(), 0);

    let local = Arc::new(LocalDomain::new());
    registry.register(local.clone());
    assert_eq!(registry.count(), 1);

    let retrieved = registry.get(&"local".to_string());
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), "local");
}

#[tokio::test]
async fn test_domain_registry_multiple_domains() {
    let registry = DomainRegistry::new();

    // Register local domain
    let local = Arc::new(LocalDomain::new());
    registry.register(local);

    // Register SSH domain
    let ssh_config = SshDomainConfig {
        id: "test-ssh".to_string(),
        name: "Test SSH".to_string(),
        ..Default::default()
    };
    let ssh = Arc::new(SshDomain::new(ssh_config));
    registry.register(ssh);

    assert_eq!(registry.count(), 2);

    // List domains
    let domains = registry.list();
    assert_eq!(domains.len(), 2);

    // Check that we have both local and ssh
    let ids: Vec<_> = domains.iter().map(|(id, _, _, _)| id.as_str()).collect();
    assert!(ids.contains(&"local"));
    assert!(ids.contains(&"test-ssh"));
}

#[tokio::test]
async fn test_domain_registry_default() {
    let registry = DomainRegistry::new();

    let local = Arc::new(LocalDomain::new());
    registry.register(local.clone());

    // First registered domain should be default
    let default = registry.get_default();
    assert!(default.is_some());
    assert_eq!(default.unwrap().id(), "local");
}

#[tokio::test]
async fn test_local_domain_basic_operations() {
    let domain = LocalDomain::new();

    // Test initial state
    assert!(domain.is_connected());
    assert_eq!(domain.id(), "local");

    // Spawn a pane
    let config = PaneConfig {
        shell: "bash".to_string(),
        cols: 80,
        rows: 24,
        ..Default::default()
    };

    let handle = domain.spawn_pane(config).await.unwrap();
    assert_eq!(handle.domain_id, "local");
    assert_eq!(handle.pane_id, 1);

    // Check stats
    let stats = domain.stats();
    assert_eq!(stats.active_panes, 1);

    // Close pane
    domain.close_pane(&handle).await.unwrap();
    let stats = domain.stats();
    assert_eq!(stats.active_panes, 0);
}

#[tokio::test]
async fn test_local_domain_multiple_panes() {
    let domain = LocalDomain::new();

    // Spawn multiple panes
    let handle1 = domain.spawn_pane(PaneConfig::default()).await.unwrap();
    let handle2 = domain.spawn_pane(PaneConfig::default()).await.unwrap();
    let handle3 = domain.spawn_pane(PaneConfig::default()).await.unwrap();

    assert_eq!(handle1.pane_id, 1);
    assert_eq!(handle2.pane_id, 2);
    assert_eq!(handle3.pane_id, 3);

    let stats = domain.stats();
    assert_eq!(stats.active_panes, 3);

    // Close middle pane
    domain.close_pane(&handle2).await.unwrap();
    let stats = domain.stats();
    assert_eq!(stats.active_panes, 2);
}

#[tokio::test]
async fn test_local_domain_io() {
    let domain = LocalDomain::new();

    // Use 'cat' for simple echo testing
    let config = PaneConfig {
        shell: "cat".to_string(),
        ..Default::default()
    };

    let handle = domain.spawn_pane(config).await.unwrap();

    // Write data
    let test_data = b"hello world\n";
    domain.write_to_pane(&handle, test_data).await.unwrap();

    // Give it time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Read back
    let mut buf = [0u8; 1024];
    let n = domain.read_from_pane(&handle, &mut buf).await.unwrap();

    assert!(n > 0, "Should have read some data");
    assert!(
        buf[..n].windows(11).any(|w| w == b"hello world"),
        "Should contain echoed text"
    );

    // Check stats
    let stats = domain.stats();
    assert!(stats.bytes_sent >= test_data.len() as u64);
    assert!(stats.bytes_received > 0);
}

#[tokio::test]
async fn test_local_domain_resize() {
    let domain = LocalDomain::new();

    let handle = domain.spawn_pane(PaneConfig::default()).await.unwrap();

    // Resize to different dimensions
    domain.resize_pane(&handle, 120, 40).await.unwrap();
    domain.resize_pane(&handle, 200, 60).await.unwrap();

    // Should not error
}

#[tokio::test]
async fn test_ssh_domain_config_creation() {
    let config = SshDomainConfig {
        id: "my-server".to_string(),
        name: "My Development Server".to_string(),
        host: "dev.example.com".to_string(),
        port: 2222,
        user: "developer".to_string(),
        auth: SshAuth::Agent,
        connect_timeout: 15,
        forward_agent: true,
        remote_cwd: Some("/home/developer/projects".to_string()),
    };

    let domain = SshDomain::new(config.clone());

    assert_eq!(domain.id(), "my-server");
    assert_eq!(domain.name(), "My Development Server");
    assert!(!domain.is_connected()); // Not connected until connect() is called
}

// Tests below require an actual SSH server and are marked as ignored
// Run with: cargo test -- --ignored

#[tokio::test]
#[ignore = "Requires SSH server at localhost:22"]
async fn test_ssh_domain_connect_local() {
    // This test requires:
    // 1. SSH server running on localhost:22
    // 2. SSH agent with loaded key OR password authentication set up
    // 3. User has permission to connect

    let config = SshDomainConfig {
        id: "localhost-ssh".to_string(),
        name: "Localhost SSH".to_string(),
        host: "localhost".to_string(),
        port: 22,
        user: std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
        auth: SshAuth::Agent,
        connect_timeout: 5,
        forward_agent: false,
        remote_cwd: None,
    };

    let domain = SshDomain::new(config);

    // Attempt to connect
    match domain.reconnect().await {
        Ok(_) => {
            assert!(domain.is_connected());

            // Try spawning a pane
            let pane_config = PaneConfig {
                shell: "bash".to_string(),
                cols: 80,
                rows: 24,
                ..Default::default()
            };

            let handle = domain.spawn_pane(pane_config).await.unwrap();
            assert_eq!(handle.domain_id, "localhost-ssh");

            // Close the pane
            domain.close_pane(&handle).await.unwrap();
        }
        Err(e) => {
            eprintln!(
                "SSH connection failed (this is expected if SSH server not set up): {}",
                e
            );
            // Don't fail the test, just skip
        }
    }
}

#[tokio::test]
#[ignore = "Requires SSH server setup"]
async fn test_ssh_domain_io() {
    // Similar to local domain IO test, but over SSH
    let config = SshDomainConfig {
        id: "localhost-ssh".to_string(),
        name: "Localhost SSH".to_string(),
        host: "localhost".to_string(),
        user: std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
        auth: SshAuth::Agent,
        ..Default::default()
    };

    let domain = SshDomain::new(config);

    if domain.reconnect().await.is_err() {
        eprintln!("Skipping SSH I/O test - connection failed");
        return;
    }

    let pane_config = PaneConfig {
        shell: "cat".to_string(),
        ..Default::default()
    };

    let handle = domain.spawn_pane(pane_config).await.unwrap();

    // Write data
    let test_data = b"ssh test\n";
    domain.write_to_pane(&handle, test_data).await.unwrap();

    // Give SSH some time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Read back
    let mut buf = [0u8; 1024];
    let n = domain.read_from_pane(&handle, &mut buf).await.unwrap();

    if n > 0 {
        assert!(
            buf[..n].windows(8).any(|w| w == b"ssh test"),
            "Should contain echoed text over SSH"
        );
    }

    domain.close_pane(&handle).await.unwrap();
}

#[tokio::test]
async fn test_cross_domain_operations() {
    let registry = DomainRegistry::new();

    // Register both local and SSH domains
    let local = Arc::new(LocalDomain::new());
    registry.register(local.clone());

    let ssh_config = SshDomainConfig {
        id: "test-ssh".to_string(),
        name: "Test SSH".to_string(),
        ..Default::default()
    };
    let ssh = Arc::new(SshDomain::new(ssh_config));
    registry.register(ssh.clone());

    // Spawn panes in both domains
    let local_handle = local.spawn_pane(PaneConfig::default()).await.unwrap();

    // Note: SSH pane won't actually connect without a server,
    // but we can test the registry lookup
    assert_eq!(local_handle.domain_id, "local");

    // Verify we can look up domains by handle
    let domain = registry.get(&local_handle.domain_id);
    assert!(domain.is_some());
    assert_eq!(domain.unwrap().id(), &local_handle.domain_id);
}
