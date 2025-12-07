//! Integration tests for GPG signature verification workflow
//!
//! These tests verify the complete end-to-end workflow of plugin verification including:
//! - GPG signature verification with sequoia-openpgp
//! - Checksum verification
//! - Unsigned plugin warnings
//! - Invalid signature blocking
//! - Verification status tracking

#[cfg(test)]
#[cfg(feature = "registry")]
mod integration_tests {
    use crate::registry::{
        security::PluginVerifier,
        types::{PluginEntry, PluginStats, PluginVersion, SecurityConfig, VerificationStatus},
    };
    use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
    use sequoia_openpgp::cert::CertBuilder;
    use sequoia_openpgp::policy::StandardPolicy;
    use sequoia_openpgp::serialize::stream::{Armorer, Message, Signer};
    use sequoia_openpgp::serialize::SerializeInto;
    use std::io::Write;

    fn create_test_cert() -> (sequoia_openpgp::Cert, sequoia_openpgp::Fingerprint) {
        use sequoia_openpgp::types::KeyFlags;

        let (cert, _) = CertBuilder::new()
            .add_userid("Test Author <test@example.com>")
            .add_signing_subkey()
            .set_primary_key_flags(KeyFlags::empty().set_certification())
            .generate()
            .expect("Failed to generate test certificate");
        let fp = cert.fingerprint();
        (cert, fp)
    }

    fn sign_content(cert: &sequoia_openpgp::Cert, content: &[u8]) -> String {
        use sequoia_openpgp::serialize::stream::LiteralWriter;

        let policy = StandardPolicy::new();
        let signing_keypair = cert
            .keys()
            .secret()
            .with_policy(&policy, None)
            .for_signing()
            .next()
            .unwrap()
            .key()
            .clone()
            .into_keypair()
            .unwrap();

        let mut signature_bytes = Vec::new();
        let message = Message::new(&mut signature_bytes);
        let message = Armorer::new(message).build().unwrap();
        let message = Signer::new(message, signing_keypair)
            .detached()
            .build()
            .unwrap();
        let mut literal = LiteralWriter::new(message).build().unwrap();
        literal.write_all(content).unwrap();
        literal.finalize().unwrap();

        BASE64_STANDARD.encode(&signature_bytes)
    }

    fn create_test_plugin_entry(checksum: &str, signature: Option<String>) -> PluginEntry {
        PluginEntry {
            name: "test-plugin".to_string(),
            description: "Test plugin".to_string(),
            readme: None,
            author: "Test Author".to_string(),
            author_email: Some("test@example.com".to_string()),
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![PluginVersion {
                version: "1.0.0".to_string(),
                download_url: "https://example.com/test-plugin-1.0.0.fzb".to_string(),
                checksum: checksum.to_string(),
                signature,
                changelog: None,
                api_version: "0.1.0".to_string(),
                min_scarab_version: "0.1.0".to_string(),
                size: 1024,
                released_at: 0,
                prerelease: false,
            }],
            tags: vec![],
            stats: PluginStats::default(),
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn test_full_gpg_verification_workflow() {
        // Generate test certificate
        let (cert, fingerprint) = create_test_cert();
        let fingerprint_str = fingerprint.to_hex();

        // Create test content
        let content = b"FZB\x00test plugin bytecode content";
        let checksum = PluginVerifier::compute_checksum(content);

        // Sign the content
        let signature = sign_content(&cert, content);

        // Create plugin entry with signature
        let entry = create_test_plugin_entry(&checksum, Some(signature));

        // Export cert to keyring
        let keyring_bytes = cert.armored().to_vec().unwrap();
        let keyring_path = std::env::temp_dir().join("test_keyring.asc");
        std::fs::write(&keyring_path, keyring_bytes).unwrap();

        // Configure verifier with trusted key
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: true,
            trusted_keys: vec![fingerprint_str.clone()],
            allow_unsigned: false,
            keyring_path: Some(keyring_path.clone()),
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);

        // Verify the plugin
        let result = verifier.verify(content, &entry, "1.0.0");

        // Cleanup
        let _ = std::fs::remove_file(keyring_path);

        // Assert verification succeeded with correct status
        assert!(result.is_ok(), "Verification failed: {:?}", result.err());
        let status = result.unwrap();
        match status {
            VerificationStatus::Verified {
                key_fingerprint,
                signature_timestamp,
            } => {
                assert!(
                    key_fingerprint
                        .to_lowercase()
                        .contains(&fingerprint_str.to_lowercase())
                        || fingerprint_str
                            .to_lowercase()
                            .contains(&key_fingerprint.to_lowercase()),
                    "Fingerprint mismatch: expected {}, got {}",
                    fingerprint_str,
                    key_fingerprint
                );
                assert!(signature_timestamp > 0, "Signature timestamp should be set");
            }
            _ => panic!("Expected Verified status, got {:?}", status),
        }
    }

    #[test]
    fn test_unsigned_plugin_warning() {
        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);
        let entry = create_test_plugin_entry(&checksum, None);

        // Configure to require signature but allow unsigned
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: true,
            trusted_keys: vec![],
            allow_unsigned: true,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.verify(content, &entry, "1.0.0");

        assert!(result.is_ok());
        match result.unwrap() {
            VerificationStatus::Unverified { warning } => {
                assert!(
                    warning.contains("unsigned"),
                    "Warning should mention unsigned: {}",
                    warning
                );
            }
            _ => panic!("Expected Unverified status with warning"),
        }
    }

    #[test]
    fn test_unsigned_plugin_blocks_install() {
        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);
        let entry = create_test_plugin_entry(&checksum, None);

        // Configure to require signature and disallow unsigned
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: true,
            trusted_keys: vec![],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.verify(content, &entry, "1.0.0");

        assert!(result.is_err(), "Unsigned plugin should be rejected");
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("signature required"),
            "Error should mention signature requirement: {}",
            error
        );
    }

    #[test]
    fn test_invalid_signature_blocks_install() {
        let (cert, fingerprint) = create_test_cert();
        let fingerprint_str = fingerprint.to_hex();

        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);

        // Sign different content
        let wrong_content = b"different content";
        let signature = sign_content(&cert, wrong_content);

        let entry = create_test_plugin_entry(&checksum, Some(signature));

        // Export cert to keyring
        let keyring_bytes = cert.armored().to_vec().unwrap();
        let keyring_path = std::env::temp_dir().join("test_keyring_invalid.asc");
        std::fs::write(&keyring_path, keyring_bytes).unwrap();

        let config = SecurityConfig {
            require_checksum: true,
            require_signature: true,
            trusted_keys: vec![fingerprint_str],
            allow_unsigned: false,
            keyring_path: Some(keyring_path.clone()),
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.verify(content, &entry, "1.0.0");

        // Cleanup
        let _ = std::fs::remove_file(keyring_path);

        assert!(result.is_err(), "Invalid signature should be rejected");
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("verification failed") || error.contains("Invalid signature"),
            "Error should mention verification failure: {}",
            error
        );
    }

    #[test]
    fn test_checksum_only_verification() {
        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);
        let entry = create_test_plugin_entry(&checksum, None);

        // Configure to only require checksum
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: false,
            trusted_keys: vec![],
            allow_unsigned: true,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.verify(content, &entry, "1.0.0");

        assert!(result.is_ok());
        match result.unwrap() {
            VerificationStatus::ChecksumOnly { checksum: cs } => {
                assert_eq!(cs, checksum, "Checksum should match");
            }
            _ => panic!("Expected ChecksumOnly status"),
        }
    }

    #[test]
    fn test_tampered_content_detected() {
        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);
        let entry = create_test_plugin_entry(&checksum, None);

        let config = SecurityConfig {
            require_checksum: true,
            require_signature: false,
            trusted_keys: vec![],
            allow_unsigned: true,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);

        // Try to verify with tampered content
        let tampered_content = b"tampered plugin content";
        let result = verifier.verify(tampered_content, &entry, "1.0.0");

        assert!(result.is_err(), "Tampered content should be detected");
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("Checksum mismatch"),
            "Error should mention checksum mismatch: {}",
            error
        );
    }

    #[test]
    fn test_trusted_key_management() {
        let (cert, fingerprint) = create_test_cert();
        let (untrusted_cert, _untrusted_fp) = create_test_cert();

        let fingerprint_str = fingerprint.to_hex();

        let content = b"test plugin content";
        let checksum = PluginVerifier::compute_checksum(content);

        // Sign with untrusted cert
        let signature = sign_content(&untrusted_cert, content);
        let entry = create_test_plugin_entry(&checksum, Some(signature));

        // Create keyring with both certs
        let mut keyring_bytes = Vec::new();
        keyring_bytes.extend_from_slice(&cert.armored().to_vec().unwrap());
        keyring_bytes.extend_from_slice(&untrusted_cert.armored().to_vec().unwrap());
        let keyring_path = std::env::temp_dir().join("test_keyring_multi.asc");
        std::fs::write(&keyring_path, keyring_bytes).unwrap();

        // Configure with only trusted key
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: true,
            trusted_keys: vec![fingerprint_str],
            allow_unsigned: false,
            keyring_path: Some(keyring_path.clone()),
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.verify(content, &entry, "1.0.0");

        // Cleanup
        let _ = std::fs::remove_file(keyring_path);

        assert!(
            result.is_err(),
            "Signature from untrusted key should be rejected"
        );
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("untrusted") || error.contains("verification failed"),
            "Error should mention trust issue: {}",
            error
        );
    }
}
