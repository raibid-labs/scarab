//! Focusable entity detection system for navigation
//!
//! This module scans terminal content and emits focusable entities for the navigation system.
//! It detects URLs, file paths, emails, and integrates with prompt markers to create
//! navigation anchors that can be focused via hint mode.
//!
//! ## Architecture
//!
//! The focusable system operates in several phases:
//! 1. **Scanning Phase**: When entering hint mode, scan terminal content with regex patterns
//! 2. **Entity Spawning**: Create FocusableRegion entities for each detected item
//! 3. **Coordinate Conversion**: Transform grid coordinates to world space for rendering
//! 4. **Zone Filtering**: Filter focusables to current prompt zone when active
//! 5. **Cleanup Phase**: Despawn all focusables when exiting hint mode
//!
//! ## Integration Points
//!
//! - **Terminal Content**: Reads from SharedMemoryReader to scan terminal grid
//! - **Prompt Markers**: Queries NavAnchor entities to create focusables for prompts
//! - **Navigation System**: Emits FocusableRegion entities for NavHint generation
//! - **Zone Filtering**: Listens for PromptZoneFocusedEvent to filter regions

use bevy::prelude::*;
use regex::Regex;
use scarab_protocol::TerminalMetrics;

use crate::integration::SharedMemoryReader;
use crate::prompt_markers::{NavAnchor, PromptZoneFocusedEvent};

use super::{EnterHintModeEvent, ExitHintModeEvent, NavSystemSet};

// ==================== Components ====================

/// Focusable region component representing a detectable navigation target
///
/// Each FocusableRegion represents a specific area of the terminal that can
/// be navigated to and interacted with. This includes URLs, file paths, emails,
/// prompt markers, and UI widgets.
///
/// # Example
/// ```rust,ignore
/// commands.spawn(FocusableRegion {
///     region_type: FocusableType::Url,
///     grid_start: (10, 5),
///     grid_end: (30, 5),
///     content: "https://example.com".to_string(),
///     source: FocusableSource::Terminal,
///     screen_position: None, // Will be calculated by bounds_to_world_coords
///     pane_id: Some(1), // Track which pane owns this focusable
///     generation: 0, // Generation number for stale detection
/// });
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
pub struct FocusableRegion {
    /// Type of focusable element
    pub region_type: FocusableType,

    /// Starting position in terminal grid (column, row)
    pub grid_start: (u16, u16),

    /// Ending position in terminal grid (column, row) - exclusive
    pub grid_end: (u16, u16),

    /// Actual text content (URL, path, email, etc.)
    pub content: String,

    /// Source system that detected this focusable
    pub source: FocusableSource,

    /// World space position for rendering (calculated from grid coords)
    pub screen_position: Option<Vec2>,

    /// Track which pane owns this focusable (for cleanup on pane close)
    pub pane_id: Option<u64>,

    /// Generation number for stale detection (increments on chunk refresh)
    pub generation: u64,
}

/// Type of focusable element
///
/// Determines how the navigation system interprets and acts upon this focusable.
/// Different types may have different rendering styles and activation behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusableType {
    /// HTTP/HTTPS URL or www.* link
    Url,

    /// File system path (absolute or relative)
    FilePath,

    /// Email address (mailto: compatible)
    Email,

    /// Shell prompt marker (from OSC 133 sequences)
    PromptMarker,

    /// UI widget or interactive element (ratatui, etc.)
    Widget,
}

/// Source system that detected this focusable
///
/// Tracks the origin of the focusable for debugging and filtering purposes.
/// Different sources may have different priority or visibility rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusableSource {
    /// Detected by scanning terminal text content
    Terminal,

    /// Detected from Ratatui UI overlay
    Ratatui,

    /// Derived from prompt marker system (OSC 133)
    PromptMarker,
}

// ==================== Resources ====================

/// Configuration resource for focusable scanning
///
/// Controls the behavior of the focusable detection system, including
/// regex patterns for detection and performance limits.
#[derive(Resource, Clone)]
pub struct FocusableScanConfig {
    /// Regex pattern for URL detection
    pub url_regex: String,

    /// Regex pattern for file path detection
    pub filepath_regex: String,

    /// Regex pattern for email detection
    pub email_regex: String,

    /// Whether to scan on every frame (true) or only when entering hint mode (false)
    /// Default: false - only scan when hint mode activates for better performance
    pub scan_on_frame: bool,

    /// Maximum number of focusables to detect (prevents performance issues)
    pub max_focusables: usize,
}

impl Default for FocusableScanConfig {
    fn default() -> Self {
        Self {
            // Match HTTP(S) URLs and www.* patterns
            url_regex: r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+".to_string(),

            // Match absolute and relative file paths
            // More restrictive than link_hints to reduce false positives
            filepath_regex:
                r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+\.[a-zA-Z]{2,5}"
                    .to_string(),

            // Match email addresses
            email_regex: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),

            scan_on_frame: false,
            max_focusables: 500, // Reasonable limit for typical terminal usage
        }
    }
}

/// Internal resource for compiled regex patterns
///
/// Caches compiled regex patterns to avoid recompilation on every scan.
#[derive(Resource)]
pub(crate) struct FocusableDetector {
    pub(crate) url_regex: Regex,
    pub(crate) filepath_regex: Regex,
    pub(crate) email_regex: Regex,
}

/// Resource for tracking focusable generation numbers
///
/// Generation numbers are incremented when terminal content is refreshed
/// (e.g., on chunk updates). Focusables with mismatched generation numbers
/// are considered stale and should be cleaned up.
#[derive(Resource, Default, Debug, Clone)]
pub struct FocusableGeneration {
    /// Global generation counter
    pub generation: u64,

    /// Per-pane generation numbers
    pub pane_generations: std::collections::HashMap<u64, u64>,
}

impl FocusableGeneration {
    /// Create a new generation tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the generation for a specific pane
    ///
    /// Call this when the pane's content is refreshed to mark all
    /// existing focusables as stale.
    pub fn increment_pane(&mut self, pane_id: u64) -> u64 {
        let entry = self.pane_generations.entry(pane_id).or_insert(0);
        *entry += 1;
        self.generation += 1;
        *entry
    }

    /// Get the current generation for a pane
    pub fn get_pane_generation(&self, pane_id: u64) -> u64 {
        self.pane_generations.get(&pane_id).copied().unwrap_or(0)
    }

    /// Remove a pane's generation tracking (on pane close)
    pub fn remove_pane(&mut self, pane_id: u64) {
        self.pane_generations.remove(&pane_id);
    }
}

impl FocusableDetector {
    pub(crate) fn new(config: &FocusableScanConfig) -> Self {
        Self {
            url_regex: Regex::new(&config.url_regex)
                .expect("Invalid URL regex in FocusableScanConfig"),
            filepath_regex: Regex::new(&config.filepath_regex)
                .expect("Invalid filepath regex in FocusableScanConfig"),
            email_regex: Regex::new(&config.email_regex)
                .expect("Invalid email regex in FocusableScanConfig"),
        }
    }

    /// Detect all focusables in terminal text content
    ///
    /// Returns a vector of (content, type, start_col, start_row, end_col, end_row)
    pub(crate) fn detect_all(
        &self,
        text: &str,
        max_focusables: usize,
    ) -> Vec<(String, FocusableType, u16, u16, u16, u16)> {
        let mut focusables = Vec::new();

        // Split text into lines and track row positions
        for (row, line) in text.lines().enumerate() {
            let row = row as u16;

            // Detect URLs
            for m in self.url_regex.find_iter(line) {
                if focusables.len() >= max_focusables {
                    break;
                }
                focusables.push((
                    m.as_str().to_string(),
                    FocusableType::Url,
                    m.start() as u16,
                    row,
                    m.end() as u16,
                    row,
                ));
            }

            // Detect file paths
            for m in self.filepath_regex.find_iter(line) {
                if focusables.len() >= max_focusables {
                    break;
                }
                let path = m.as_str();
                // Additional validation: must have reasonable length
                if path.len() > 3 {
                    focusables.push((
                        path.to_string(),
                        FocusableType::FilePath,
                        m.start() as u16,
                        row,
                        m.end() as u16,
                        row,
                    ));
                }
            }

            // Detect emails
            for m in self.email_regex.find_iter(line) {
                if focusables.len() >= max_focusables {
                    break;
                }
                focusables.push((
                    m.as_str().to_string(),
                    FocusableType::Email,
                    m.start() as u16,
                    row,
                    m.end() as u16,
                    row,
                ));
            }

            if focusables.len() >= max_focusables {
                break;
            }
        }

        focusables
    }
}

// ==================== Systems ====================

/// System: Initialize focusable detector from config
///
/// This system runs once at startup to compile regex patterns from config.
fn initialize_focusable_detector(mut commands: Commands, config: Res<FocusableScanConfig>) {
    let detector = FocusableDetector::new(&config);
    commands.insert_resource(detector);
    info!("Focusable detector initialized with compiled regex patterns");
}

/// System: Scan terminal content and spawn focusable entities
///
/// This system runs when entering hint mode. It:
/// 1. Reads terminal content from SharedMemoryReader
/// 2. Detects URLs, file paths, and emails using regex
/// 3. Queries NavAnchor entities from prompt markers
/// 4. Spawns FocusableRegion entities for each detection
///
/// Runs in NavSystemSet::Input phase.
fn scan_terminal_focusables(
    mut commands: Commands,
    mut enter_hint_events: EventReader<EnterHintModeEvent>,
    state_reader: Res<SharedMemoryReader>,
    detector: Res<FocusableDetector>,
    config: Res<FocusableScanConfig>,
    nav_anchors: Query<&NavAnchor>,
    nav_registry: Res<crate::NavStateRegistry>,
    generation: Res<FocusableGeneration>,
) {
    // Only scan when entering hint mode
    if enter_hint_events.is_empty() {
        return;
    }
    enter_hint_events.clear();

    // Get the active pane ID
    let active_pane_id = nav_registry.active_pane();

    // Get the current generation for this pane
    let current_generation = active_pane_id
        .map(|id| generation.get_pane_generation(id))
        .unwrap_or(0);

    // Extract terminal text content
    let safe_state = state_reader.get_safe_state();
    let terminal_text = crate::integration::extract_grid_text(&safe_state);

    // Detect all focusables in terminal content
    let detected = detector.detect_all(&terminal_text, config.max_focusables);
    info!("Detected {} focusables in terminal content", detected.len());

    // Spawn FocusableRegion entities for detected items
    for (content, region_type, start_col, start_row, end_col, end_row) in detected {
        commands.spawn(FocusableRegion {
            region_type,
            grid_start: (start_col, start_row),
            grid_end: (end_col, end_row),
            content,
            source: FocusableSource::Terminal,
            screen_position: None, // Will be calculated by bounds_to_world_coords
            pane_id: active_pane_id,
            generation: current_generation,
        });
    }

    // Also create FocusableRegion entities from NavAnchor prompt markers
    let mut prompt_focusables = 0;
    for anchor in nav_anchors.iter() {
        // Only create focusables for prompt start markers
        // (command finished markers are less useful for navigation)
        if anchor.anchor_type != crate::prompt_markers::PromptAnchorType::PromptStart {
            continue;
        }

        // Create a focusable region for the prompt marker
        // Position it at the start of the line where the prompt marker is
        let content = anchor
            .command_text
            .clone()
            .unwrap_or_else(|| format!("Prompt at line {}", anchor.line));

        commands.spawn(FocusableRegion {
            region_type: FocusableType::PromptMarker,
            grid_start: (0, anchor.line as u16),
            grid_end: (10, anchor.line as u16), // Arbitrary width for marker
            content,
            source: FocusableSource::PromptMarker,
            screen_position: None,
            pane_id: active_pane_id,
            generation: current_generation,
        });

        prompt_focusables += 1;
    }

    info!(
        "Created {} prompt marker focusables from NavAnchor entities",
        prompt_focusables
    );
}

/// System: Convert grid coordinates to world coordinates
///
/// Transforms FocusableRegion grid positions to screen space using TerminalMetrics.
/// This enables accurate rendering of hint labels and focus indicators.
///
/// Runs in NavSystemSet::Update phase.
fn bounds_to_world_coords(
    mut focusables: Query<&mut FocusableRegion, Changed<FocusableRegion>>,
    metrics: Res<TerminalMetrics>,
) {
    for mut region in focusables.iter_mut() {
        // Calculate world position from grid coordinates
        // Use the start position as the anchor point for the focusable
        let world_x = region.grid_start.0 as f32 * metrics.cell_width;
        let world_y = -(region.grid_start.1 as f32 * metrics.cell_height);

        region.screen_position = Some(Vec2::new(world_x, world_y));
    }
}

/// System: Filter focusables to current prompt zone
///
/// Listens for PromptZoneFocusedEvent and despawns FocusableRegion entities
/// that fall outside the specified zone boundaries.
///
/// This enables context-aware navigation where only relevant focusables
/// in the current command output are visible.
///
/// Runs in NavSystemSet::Update phase.
fn filter_focusables_by_zone(
    mut commands: Commands,
    mut zone_events: EventReader<PromptZoneFocusedEvent>,
    focusables: Query<(Entity, &FocusableRegion)>,
) {
    for event in zone_events.read() {
        info!(
            "Filtering focusables to prompt zone: lines {}-{}",
            event.start_line, event.end_line
        );

        let mut removed_count = 0;

        // Despawn focusables outside the zone
        for (entity, region) in focusables.iter() {
            let region_row = region.grid_start.1 as u32;

            // Check if focusable is outside the zone boundaries
            if region_row < event.start_line || region_row >= event.end_line {
                commands.entity(entity).despawn();
                removed_count += 1;
            }
        }

        info!("Removed {} focusables outside prompt zone", removed_count);
    }
}

/// System: Cleanup focusables when exiting hint mode
///
/// Despawns all FocusableRegion entities when exiting hint mode to free
/// memory and prevent stale focusables from persisting.
///
/// Runs in NavSystemSet::Update phase.
fn cleanup_focusables(
    mut commands: Commands,
    mut exit_hint_events: EventReader<ExitHintModeEvent>,
    focusables: Query<Entity, With<FocusableRegion>>,
) {
    if exit_hint_events.is_empty() {
        return;
    }
    exit_hint_events.clear();

    let mut count = 0;
    for entity in focusables.iter() {
        commands.entity(entity).despawn();
        count += 1;
    }

    info!("Cleaned up {} focusable entities on hint mode exit", count);
}

/// System: Detect and remove stale focusables
///
/// Scans all FocusableRegion entities and despawns those with mismatched
/// generation numbers. This prevents stale focusables from persisting after
/// terminal content refreshes (chunk updates).
///
/// Runs in NavSystemSet::Update phase.
pub fn detect_stale_focusables(
    mut commands: Commands,
    generation: Res<FocusableGeneration>,
    focusables: Query<(Entity, &FocusableRegion)>,
) {
    let mut stale_count = 0;

    for (entity, region) in focusables.iter() {
        if let Some(pane_id) = region.pane_id {
            let current_gen = generation.get_pane_generation(pane_id);

            // Stale if generation doesn't match
            if region.generation != current_gen && current_gen > 0 {
                warn!(
                    "Dropping stale focusable: pane={} region_gen={} current_gen={}",
                    pane_id, region.generation, current_gen
                );
                commands.entity(entity).despawn();
                stale_count += 1;
            }
        }
    }

    if stale_count > 0 {
        info!("Removed {} stale focusables", stale_count);
    }
}

// ==================== Plugin ====================

/// Plugin for focusable entity detection system
///
/// Registers all resources, systems, and integration points for the
/// focusable detection pipeline.
///
/// ## System Ordering
///
/// 1. **Input Phase**: scan_terminal_focusables (on EnterHintModeEvent)
/// 2. **Update Phase**: bounds_to_world_coords, filter_focusables_by_zone, cleanup_focusables
/// 3. **Render Phase**: (handled by downstream navigation rendering systems)
///
/// ## Example
/// ```rust,ignore
/// App::new()
///     .add_plugins(NavigationPlugin)
///     .add_plugins(FocusablePlugin)
///     .run();
/// ```
pub struct FocusablePlugin;

impl Plugin for FocusablePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register config resource with defaults
            .init_resource::<FocusableScanConfig>()
            // Register generation tracking resource
            .init_resource::<FocusableGeneration>()
            // Register events this plugin depends on
            // (PromptMarkersPlugin may also register these, but that's harmless)
            .add_event::<PromptZoneFocusedEvent>()
            // Initialize detector at startup
            .add_systems(Startup, initialize_focusable_detector)
            // Register systems in proper phases
            .add_systems(
                Update,
                (
                    // Input phase: scan and spawn focusables
                    scan_terminal_focusables.in_set(NavSystemSet::Input),
                    // Update phase: coordinate conversion and filtering
                    (
                        bounds_to_world_coords,
                        filter_focusables_by_zone,
                        cleanup_focusables,
                        detect_stale_focusables,
                    )
                        .chain()
                        .in_set(NavSystemSet::Update),
                ),
            );

        info!("FocusablePlugin initialized");
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focusable_type_eq() {
        assert_eq!(FocusableType::Url, FocusableType::Url);
        assert_ne!(FocusableType::Url, FocusableType::FilePath);
    }

    #[test]
    fn test_focusable_source_eq() {
        assert_eq!(FocusableSource::Terminal, FocusableSource::Terminal);
        assert_ne!(FocusableSource::Terminal, FocusableSource::Ratatui);
    }

    #[test]
    fn test_focusable_region_clone() {
        let region = FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (10, 5),
            grid_end: (30, 5),
            content: "https://example.com".to_string(),
            source: FocusableSource::Terminal,
            screen_position: Some(Vec2::new(100.0, 50.0)),
            pane_id: Some(1),
            generation: 0,
        };

        let cloned = region.clone();
        assert_eq!(region, cloned);
    }

    #[test]
    fn test_focusable_detector_urls() {
        let config = FocusableScanConfig::default();
        let detector = FocusableDetector::new(&config);

        let text = "Visit https://example.com or www.github.com for more info";
        let focusables = detector.detect_all(text, 100);

        let urls: Vec<_> = focusables
            .iter()
            .filter(|(_, t, _, _, _, _)| *t == FocusableType::Url)
            .collect();

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0].0, "https://example.com");
        assert_eq!(urls[1].0, "www.github.com");
    }

    #[test]
    fn test_focusable_detector_emails() {
        let config = FocusableScanConfig::default();
        let detector = FocusableDetector::new(&config);

        let text = "Contact us at support@example.com or info@test.org";
        let focusables = detector.detect_all(text, 100);

        let emails: Vec<_> = focusables
            .iter()
            .filter(|(_, t, _, _, _, _)| *t == FocusableType::Email)
            .collect();

        assert_eq!(emails.len(), 2);
        assert_eq!(emails[0].0, "support@example.com");
        assert_eq!(emails[1].0, "info@test.org");
    }

    #[test]
    fn test_focusable_detector_file_paths() {
        let config = FocusableScanConfig::default();
        let detector = FocusableDetector::new(&config);

        let text = "Check /usr/local/bin/foo.txt or ./relative/path.rs";
        let focusables = detector.detect_all(text, 100);

        let paths: Vec<_> = focusables
            .iter()
            .filter(|(_, t, _, _, _, _)| *t == FocusableType::FilePath)
            .collect();

        assert!(paths.len() >= 2);
        assert!(paths
            .iter()
            .any(|(content, _, _, _, _, _)| content.contains("foo.txt")));
        assert!(paths
            .iter()
            .any(|(content, _, _, _, _, _)| content.contains("path.rs")));
    }

    #[test]
    fn test_focusable_detector_max_limit() {
        let config = FocusableScanConfig {
            max_focusables: 5,
            ..Default::default()
        };
        let detector = FocusableDetector::new(&config);

        // Create text with many URLs
        let mut text = String::new();
        for i in 0..20 {
            text.push_str(&format!("https://example{}.com ", i));
        }

        let focusables = detector.detect_all(&text, config.max_focusables);
        assert_eq!(focusables.len(), 5); // Should respect max_focusables
    }

    #[test]
    fn test_focusable_detector_multiline() {
        let config = FocusableScanConfig::default();
        let detector = FocusableDetector::new(&config);

        let text = "Line 1: https://example.com\nLine 2: test@email.com\nLine 3: /path/to/file.txt";
        let focusables = detector.detect_all(text, 100);

        // Check row positions
        let url = focusables
            .iter()
            .find(|(content, _, _, _, _, _)| content.contains("example.com"));
        assert!(url.is_some());
        let (_, _, _, row, _, _) = url.unwrap();
        assert_eq!(*row, 0); // First line

        let email = focusables
            .iter()
            .find(|(content, _, _, _, _, _)| content.contains("test@email.com"));
        assert!(email.is_some());
        let (_, _, _, row, _, _) = email.unwrap();
        assert_eq!(*row, 1); // Second line
    }

    #[test]
    fn test_focusable_generation_tracking() {
        let mut gen = FocusableGeneration::new();

        // Initial generation should be 0
        assert_eq!(gen.get_pane_generation(1), 0);

        // Increment pane 1
        let gen1 = gen.increment_pane(1);
        assert_eq!(gen1, 1);
        assert_eq!(gen.get_pane_generation(1), 1);

        // Increment pane 1 again
        let gen2 = gen.increment_pane(1);
        assert_eq!(gen2, 2);
        assert_eq!(gen.get_pane_generation(1), 2);

        // Increment different pane
        let gen_pane2 = gen.increment_pane(2);
        assert_eq!(gen_pane2, 1);
        assert_eq!(gen.get_pane_generation(2), 1);

        // Pane 1 should still be at 2
        assert_eq!(gen.get_pane_generation(1), 2);

        // Remove pane 1
        gen.remove_pane(1);
        assert_eq!(gen.get_pane_generation(1), 0); // Back to default
    }

    #[test]
    fn test_focusable_pane_id_tracking() {
        let region1 = FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (10, 5),
            grid_end: (30, 5),
            content: "https://example.com".to_string(),
            source: FocusableSource::Terminal,
            screen_position: None,
            pane_id: Some(1),
            generation: 0,
        };

        let region2 = FocusableRegion {
            region_type: FocusableType::Url,
            grid_start: (10, 5),
            grid_end: (30, 5),
            content: "https://example.com".to_string(),
            source: FocusableSource::Terminal,
            screen_position: None,
            pane_id: Some(2),
            generation: 0,
        };

        assert_eq!(region1.pane_id, Some(1));
        assert_eq!(region2.pane_id, Some(2));
        assert_ne!(region1.pane_id, region2.pane_id);
    }
}
