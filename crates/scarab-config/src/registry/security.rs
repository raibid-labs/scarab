//! Security verification for plugin downloads

use super::types::{PluginEntry, SecurityConfig};
use crate::error::{ConfigError, Result};
use sha2::{Digest, Sha256};

#[cfg(feature = "registry")]
use sequoia_openpgp as openpgp;
#[cfg(feature = "registry")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "registry")]
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};

/// Plugin security verifier
pub struct PluginVerifier {
    config: SecurityConfig,
}

impl PluginVerifier {
    /// Create new verifier with configuration
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Verify plugin content against entry metadata
    pub fn verify(&self, content: &[u8], entry: &PluginEntry, version: &str) -> Result<()> {
        // Find version metadata
        let version_meta = entry
            .versions
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| {
                ConfigError::ValidationError(format!("Version {} not found", version))
            })?;

        // Verify checksum if required
        if self.config.require_checksum {
            self.verify_checksum(content, &version_meta.checksum)?;
        }

        // Verify signature if required
        if self.config.require_signature {
            if let Some(signature) = &version_meta.signature {
                self.verify_signature(content, signature)?;
            } else if !self.config.allow_unsigned {
                return Err(ConfigError::SecurityError(
                    "Plugin signature required but not provided".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Verify SHA256 checksum
    fn verify_checksum(&self, content: &[u8], expected: &str) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let computed = format!("{:x}", hasher.finalize());

        if computed != expected {
            return Err(ConfigError::SecurityError(format!(
                "Checksum mismatch: expected {}, got {}",
                expected, computed
            )));
        }

        Ok(())
    }

    /// Verify GPG signature
    ///
    /// Verifies a detached GPG signature against the provided content using sequoia-openpgp.
    ///
    /// The signature should be base64-encoded ASCII-armored OpenPGP signature.
    ///
    /// This implementation:
    /// 1. Decodes the base64 signature
    /// 2. Parses the OpenPGP signature
    /// 3. Verifies the signature against the content
    /// 4. Checks the signing key against trusted_keys (if require_key_match is true)
    /// 5. Validates signature age
    /// 6. Checks key expiration and revocation
    #[cfg(feature = "registry")]
    fn verify_signature(&self, content: &[u8], signature_b64: &str) -> Result<()> {
        use openpgp::policy::StandardPolicy;

        // Check if we have trusted keys configured
        if self.config.trusted_keys.is_empty() && self.config.keyring_path.is_none() {
            return Err(ConfigError::SecurityError(
                "GPG signature verification enabled but no trusted keys or keyring configured".to_string(),
            ));
        }

        // Decode base64 signature
        let signature_bytes = BASE64_STANDARD.decode(signature_b64).map_err(|e| {
            ConfigError::SecurityError(format!("Invalid base64 signature: {}", e))
        })?;

        // Load trusted certificates
        let certs = self.load_trusted_certs()?;

        if certs.is_empty() {
            return Err(ConfigError::SecurityError(
                "No valid trusted certificates found for verification".to_string(),
            ));
        }

        // Use the standard policy (RFC 4880)
        let policy = StandardPolicy::new();

        // Verify the signature using sequoia's verification API
        let helper = VerificationHelperImpl {
            certs,
            config: &self.config,
            good_signatures: Vec::new(),
        };

        match Self::verify_detached_signature(helper, &policy, content, &signature_bytes) {
            Ok(fingerprints) => {
                tracing::info!("Successfully verified signature(s) from key(s): {:?}", fingerprints);
                Ok(())
            }
            Err(e) => Err(ConfigError::SecurityError(format!(
                "Signature verification failed: {}",
                e
            ))),
        }
    }

    /// Fallback when registry feature is not enabled
    #[cfg(not(feature = "registry"))]
    fn verify_signature(&self, _content: &[u8], _signature: &str) -> Result<()> {
        Err(ConfigError::SecurityError(
            "GPG signature verification requires 'registry' feature to be enabled".to_string(),
        ))
    }

    /// Filter certificates by matching fingerprints
    #[cfg(feature = "registry")]
    fn filter_certs_by_fingerprints(&self, certs: &[openpgp::Cert]) -> Result<Vec<openpgp::Cert>> {
        use openpgp::Fingerprint;

        let mut matched_certs = Vec::new();

        // Parse all trusted fingerprints
        let trusted_fps: Vec<Fingerprint> = self
            .config
            .trusted_keys
            .iter()
            .filter_map(|fp_str| {
                let normalized = fp_str.trim().replace(" ", "").to_uppercase();
                match normalized.parse::<Fingerprint>() {
                    Ok(fp) => Some(fp),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse fingerprint '{}': {}",
                            fp_str,
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        if trusted_fps.is_empty() {
            return Err(ConfigError::SecurityError(
                "No valid fingerprints could be parsed from trusted_keys".to_string(),
            ));
        }

        // Match certificates against trusted fingerprints
        for cert in certs {
            let cert_fp = cert.fingerprint();

            if trusted_fps.iter().any(|trusted_fp| trusted_fp == &cert_fp) {
                tracing::info!(
                    "Matched certificate with fingerprint: {}",
                    cert_fp.to_hex()
                );
                matched_certs.push(cert.clone());
            }
        }

        Ok(matched_certs)
    }

    /// Load trusted certificates from configuration
    #[cfg(feature = "registry")]
    fn load_trusted_certs(&self) -> Result<Vec<openpgp::Cert>> {
        use openpgp::parse::Parse;
        use std::fs;

        let mut certs = Vec::new();

        // Load from keyring file if specified
        if let Some(keyring_path) = &self.config.keyring_path {
            if keyring_path.exists() {
                let keyring_data = fs::read(keyring_path).map_err(|e| {
                    ConfigError::SecurityError(format!(
                        "Failed to read keyring file {}: {}",
                        keyring_path.display(),
                        e
                    ))
                })?;

                // Parse all certificates from keyring
                let parsed_certs = openpgp::Cert::from_bytes(&keyring_data).map_err(|e| {
                    ConfigError::SecurityError(format!(
                        "Failed to parse keyring file: {}",
                        e
                    ))
                })?;

                certs.push(parsed_certs);
            } else {
                tracing::warn!(
                    "Keyring file not found: {}",
                    keyring_path.display()
                );
            }
        }

        // Filter certificates by trusted fingerprints if specified
        // This allows loading specific keys from the keyring rather than trusting all keys
        if !self.config.trusted_keys.is_empty() {
            let filtered_certs = self.filter_certs_by_fingerprints(&certs)?;

            if filtered_certs.is_empty() {
                return Err(ConfigError::SecurityError(format!(
                    "No certificates found matching trusted fingerprints: {:?}. \
                     Ensure keyring_path contains the required public keys.",
                    self.config.trusted_keys
                )));
            }

            tracing::debug!(
                "Filtered {} certificate(s) from keyring matching trusted fingerprints",
                filtered_certs.len()
            );

            return Ok(filtered_certs);
        }

        // If no trusted_keys filter, but we have a keyring, warn about using all keys
        if !certs.is_empty() && self.config.trusted_keys.is_empty() {
            tracing::warn!(
                "Using all certificates from keyring without fingerprint filtering. \
                 Consider specifying 'trusted_keys' fingerprints for better security."
            );
        }

        Ok(certs)
    }

    /// Verify detached signature using sequoia-openpgp
    #[cfg(feature = "registry")]
    fn verify_detached_signature(
        helper: VerificationHelperImpl,
        policy: &openpgp::policy::StandardPolicy,
        content: &[u8],
        signature_bytes: &[u8],
    ) -> std::result::Result<Vec<String>, String> {
        use openpgp::parse::Parse;
        use openpgp::parse::stream::DetachedVerifierBuilder;

        // Create a detached verifier with the helper
        let mut verifier = DetachedVerifierBuilder::from_bytes(signature_bytes)
            .map_err(|e| format!("Failed to parse signature: {}", e))?
            .with_policy(policy, None, helper)
            .map_err(|e| format!("Failed to initialize verifier: {}", e))?;

        // Verify the content
        verifier
            .verify_bytes(content)
            .map_err(|e| format!("Failed to verify content: {}", e))?;

        // Get the helper back and check results
        let helper_ref = verifier.into_helper();

        // Return the collected good signatures
        if !helper_ref.good_signatures.is_empty() {
            return Ok(helper_ref.good_signatures.clone());
        }

        Err("No valid signatures found".to_string())
    }

    /// Compute SHA256 checksum of content
    pub fn compute_checksum(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Validate plugin file format
    pub fn validate_plugin_format(content: &[u8], filename: &str) -> Result<PluginFormat> {
        // Check for Fusabi bytecode magic number (.fzb)
        if content.starts_with(b"FZB\x00") {
            return Ok(PluginFormat::FusabiBytecode);
        }

        // Check for Fusabi source by extension and content
        if filename.ends_with(".fsx") {
            // Basic validation: check if it looks like F# source
            if let Ok(text) = std::str::from_utf8(content) {
                if text.contains("module") || text.contains("let") || text.contains("type") {
                    return Ok(PluginFormat::FusabiSource);
                }
            }
        }

        Err(ConfigError::ValidationError(
            "Invalid plugin format: expected .fzb or .fsx file".to_string(),
        ))
    }
}

/// Plugin file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginFormat {
    /// Compiled Fusabi bytecode (.fzb)
    FusabiBytecode,
    /// Fusabi source code (.fsx)
    FusabiSource,
}

/// Verification helper implementation for sequoia-openpgp
#[cfg(feature = "registry")]
struct VerificationHelperImpl<'a> {
    certs: Vec<openpgp::Cert>,
    config: &'a SecurityConfig,
    good_signatures: Vec<String>,
}

#[cfg(feature = "registry")]
impl<'a> openpgp::parse::stream::VerificationHelper for VerificationHelperImpl<'a> {
    fn get_certs(
        &mut self,
        _ids: &[openpgp::KeyHandle],
    ) -> openpgp::Result<Vec<openpgp::Cert>> {
        // Return all our trusted certificates
        Ok(self.certs.clone())
    }

    fn check(
        &mut self,
        structure: openpgp::parse::stream::MessageStructure,
    ) -> openpgp::Result<()> {
        use openpgp::parse::stream::MessageLayer;

        let mut good_sigs = Vec::new();

        // Iterate through all layers in the message structure
        for layer in structure.into_iter() {
            match layer {
                MessageLayer::SignatureGroup { results } => {
                    // Check each signature result
                    for result in results {
                        match result {
                            Ok(good_checksum) => {
                                // Get the signing key fingerprint
                                let ka = good_checksum.ka;
                                let cert_fp = ka.cert().fingerprint().to_hex();

                                // Validate signature age if configured
                                if self.config.max_signature_age_days > 0 {
                                    let sig = good_checksum.sig;
                                    if let Some(sig_time) = sig.signature_creation_time() {
                                        let now = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map_err(|e| anyhow::anyhow!("System time error: {}", e))?
                                            .as_secs();

                                        let sig_timestamp = sig_time.duration_since(UNIX_EPOCH)
                                            .map_err(|e| anyhow::anyhow!("Signature time error: {}", e))?
                                            .as_secs();

                                        let age_days = (now.saturating_sub(sig_timestamp)) / 86400;

                                        if age_days > self.config.max_signature_age_days {
                                            return Err(anyhow::anyhow!(
                                                "Signature is too old: {} days (max: {} days)",
                                                age_days,
                                                self.config.max_signature_age_days
                                            ));
                                        }
                                    }
                                }

                                // Check if the key is in our trusted list (if required)
                                if self.config.require_key_match && !self.config.trusted_keys.is_empty() {
                                    let is_trusted = self.config.trusted_keys.iter().any(|trusted_fp| {
                                        cert_fp.to_lowercase() == trusted_fp.to_lowercase()
                                            || cert_fp.to_lowercase().ends_with(&trusted_fp.to_lowercase())
                                    });

                                    if !is_trusted {
                                        return Err(anyhow::anyhow!(
                                            "Signature from untrusted key: {}",
                                            cert_fp
                                        ));
                                    }
                                }

                                tracing::debug!("Valid signature from: {}", cert_fp);
                                good_sigs.push(cert_fp);
                            }
                            Err(e) => {
                                tracing::warn!("Bad signature: {}", e);
                                return Err(anyhow::anyhow!("Invalid signature: {}", e));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if good_sigs.is_empty() {
            return Err(anyhow::anyhow!("No valid signatures found"));
        }

        self.good_signatures = good_sigs;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::types::{PluginStats, PluginVersion};

    #[test]
    fn test_compute_checksum() {
        let content = b"hello world";
        let checksum = PluginVerifier::compute_checksum(content);
        // Known SHA256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_verify_checksum_success() {
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
        let content = b"test content";
        let checksum = PluginVerifier::compute_checksum(content);

        let entry = create_test_entry(&checksum);
        assert!(verifier.verify(content, &entry, "1.0.0").is_ok());
    }

    #[test]
    fn test_verify_checksum_failure() {
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
        let content = b"test content";
        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        let entry = create_test_entry(wrong_checksum);
        assert!(verifier.verify(content, &entry, "1.0.0").is_err());
    }

    #[test]
    fn test_signature_required_but_missing() {
        let config = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec![],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let content = b"test content";
        let checksum = PluginVerifier::compute_checksum(content);

        let entry = create_test_entry(&checksum);
        let result = verifier.verify(content, &entry, "1.0.0");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("signature required"));
    }

    #[test]
    fn test_unsigned_allowed() {
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
        let content = b"test content";
        let checksum = PluginVerifier::compute_checksum(content);

        let entry = create_test_entry(&checksum);
        // Should succeed because allow_unsigned is true
        assert!(verifier.verify(content, &entry, "1.0.0").is_ok());
    }

    #[cfg(feature = "registry")]
    #[test]
    fn test_gpg_verification_no_trusted_keys() {
        let config = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec![],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let content = b"test content";

        // Any signature should fail if no trusted keys are configured
        let fake_signature = BASE64_STANDARD.encode(b"fake signature data");
        let result = verifier.verify_signature(content, &fake_signature);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no trusted keys"));
    }

    #[test]
    fn test_validate_plugin_format() {
        // Test .fsx source format
        let fsx_content = b"module MyPlugin\nlet foo = 42";
        let result = PluginVerifier::validate_plugin_format(fsx_content, "test.fsx");
        assert!(matches!(result, Ok(PluginFormat::FusabiSource)));

        // Test invalid format
        let invalid_content = b"random content";
        let result = PluginVerifier::validate_plugin_format(invalid_content, "test.txt");
        assert!(result.is_err());
    }

    #[cfg(feature = "registry")]
    #[test]
    fn test_fingerprint_filtering() {
        use openpgp::cert::CertBuilder;

        // Generate a test certificate
        let (cert, _) = CertBuilder::new()
            .add_userid("Test User <test@example.com>")
            .generate()
            .expect("Failed to generate test certificate");

        let fingerprint = cert.fingerprint().to_hex();
        tracing::info!("Generated test cert with fingerprint: {}", fingerprint);

        // Test with matching fingerprint
        let config = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec![fingerprint.clone()],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let filtered = verifier.filter_certs_by_fingerprints(&[cert.clone()]);
        assert!(filtered.is_ok());
        assert_eq!(filtered.unwrap().len(), 1);

        // Test with non-matching fingerprint
        let config_no_match = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec!["0000000000000000000000000000000000000000".to_string()],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier_no_match = PluginVerifier::new(config_no_match);
        let filtered_empty = verifier_no_match.filter_certs_by_fingerprints(&[cert.clone()]);
        assert!(filtered_empty.is_ok());
        assert_eq!(filtered_empty.unwrap().len(), 0);

        // Test with spaces in fingerprint (should normalize)
        let fingerprint_spaced = format!(
            "{} {} {} {} {}",
            &fingerprint[0..8],
            &fingerprint[8..16],
            &fingerprint[16..24],
            &fingerprint[24..32],
            &fingerprint[32..40]
        );

        let config_spaced = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec![fingerprint_spaced],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier_spaced = PluginVerifier::new(config_spaced);
        let filtered_spaced = verifier_spaced.filter_certs_by_fingerprints(&[cert]);
        assert!(filtered_spaced.is_ok());
        assert_eq!(filtered_spaced.unwrap().len(), 1);
    }

    #[cfg(feature = "registry")]
    #[test]
    fn test_invalid_fingerprint_parsing() {
        let config = SecurityConfig {
            require_checksum: false,
            require_signature: true,
            trusted_keys: vec!["invalid_fingerprint".to_string()],
            allow_unsigned: false,
            keyring_path: None,
            require_key_match: true,
            max_signature_age_days: 365,
        };

        let verifier = PluginVerifier::new(config);
        let result = verifier.filter_certs_by_fingerprints(&[]);

        // Should error because no valid fingerprints could be parsed
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid fingerprints"));
    }

    fn create_test_entry(checksum: &str) -> PluginEntry {
        PluginEntry {
            name: "test-plugin".to_string(),
            description: "Test".to_string(),
            readme: None,
            author: "Test".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![PluginVersion {
                version: "1.0.0".to_string(),
                download_url: "https://example.com/plugin.fzb".to_string(),
                checksum: checksum.to_string(),
                signature: None,
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
}
