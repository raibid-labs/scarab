//! Navigation Performance Metrics
//!
//! This module provides instrumentation for tracking navigation system performance.
//! Metrics are collected using lock-free atomic operations to minimize overhead.
//!
//! ## Metrics Tracked
//!
//! - **hints_generated**: Total number of focusable hints generated
//! - **hint_generation_time**: Time samples for hint generation operations
//! - **focus_switches**: Number of focus changes between navigation targets
//! - **nav_input_latency**: Input-to-action processing latency samples
//! - **nav_errors**: Count of navigation errors encountered
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use scarab_client::navigation::NavMetricsPlugin;
//!
//! App::new()
//!     .add_plugins(NavigationPlugin)
//!     .add_plugins(NavMetricsPlugin::default())
//!     .run();
//! ```

use bevy::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ==================== Constants ====================

/// Maximum number of timing samples to keep in memory
const MAX_TIMING_SAMPLES: usize = 100;

/// Default reporting interval in seconds (debug mode)
const DEFAULT_REPORT_INTERVAL_SECS: u64 = 60;

// ==================== Navigation Metrics Resource ====================

/// Navigation performance metrics resource
///
/// Tracks key performance indicators for the navigation system using
/// lock-free atomic operations. Timing samples are stored in bounded
/// circular buffers to prevent unbounded memory growth.
///
/// All atomic operations use `Ordering::Relaxed` for best performance,
/// as exact ordering is not critical for telemetry data.
#[derive(Resource)]
pub struct NavMetrics {
    /// Total number of focusable hints generated
    pub hints_generated: AtomicU64,

    /// Number of focus switches between navigation targets
    pub focus_switches: AtomicU64,

    /// Count of navigation errors encountered
    pub nav_errors: AtomicU64,

    /// Timing samples for hint generation (milliseconds)
    hint_gen_times: Mutex<TimingSamples>,

    /// Timing samples for navigation input latency (microseconds)
    nav_input_latency: Mutex<TimingSamples>,

    // Pane switch metrics
    /// Total number of pane switches
    pub pane_switches: AtomicU64,

    /// Timing samples for pane switch operations (microseconds)
    pane_switch_times: Mutex<TimingSamples>,

    /// Number of focusables restored during pane switches
    pub focusables_restored: AtomicU64,

    /// Number of focusables dropped during pane switches
    pub focusables_dropped: AtomicU64,

    // Plugin navigation metrics
    /// Number of plugin navigation actions accepted
    pub plugin_actions_accepted: AtomicU64,

    /// Number of plugin navigation actions rejected
    pub plugin_actions_rejected: AtomicU64,

    /// Number of plugin focusables registered
    pub plugin_focusables_registered: AtomicU64,
            pane_switches: AtomicU64::new(0),
            pane_switch_times: Mutex::new(TimingSamples::new()),
            focusables_restored: AtomicU64::new(0),
            focusables_dropped: AtomicU64::new(0),
            plugin_actions_accepted: AtomicU64::new(0),
            plugin_actions_rejected: AtomicU64::new(0),
            plugin_focusables_registered: AtomicU64::new(0),
            plugin_rate_limit_hits: AtomicU64::new(0),

    /// Number of rate limit hits (from any source)
    pub plugin_rate_limit_hits: AtomicU64,

    /// Timestamp of last metrics report
    last_report: Mutex<Instant>,

    /// Reporting interval in seconds
    report_interval_secs: u64,
}

impl Default for NavMetrics {
    fn default() -> Self {
        Self::new(DEFAULT_REPORT_INTERVAL_SECS)
    }
}

impl NavMetrics {
    /// Create a new NavMetrics instance with specified report interval
    pub fn new(report_interval_secs: u64) -> Self {
        Self {
            hints_generated: AtomicU64::new(0),
            focus_switches: AtomicU64::new(0),
            nav_errors: AtomicU64::new(0),
            hint_gen_times: Mutex::new(TimingSamples::new()),
            nav_input_latency: Mutex::new(TimingSamples::new()),
            last_report: Mutex::new(Instant::now()),
            report_interval_secs,
        }
    }

    /// Record a hint generation operation
    ///
    /// Increments the hints_generated counter and records timing if provided.
    pub fn record_hint_generation(&self, count: u64, duration: Option<Duration>) {
        self.hints_generated.fetch_add(count, Ordering::Relaxed);

        if let Some(dur) = duration {
            let millis = dur.as_millis() as f64;
            if let Ok(mut samples) = self.hint_gen_times.lock() {
                samples.add_sample(millis);
            }
        }
    }

    /// Record a focus switch operation
    pub fn record_focus_switch(&self) {
        self.focus_switches.fetch_add(1, Ordering::Relaxed);
    }

    /// Record navigation input latency
    ///
    /// Measures the time from input reception to action emission.
    pub fn record_nav_input_latency(&self, duration: Duration) {
        let micros = duration.as_micros() as f64;
        if let Ok(mut samples) = self.nav_input_latency.lock() {
            samples.add_sample(micros);
        }
    }

    /// Record a navigation error
    pub fn record_error(&self) {
        self.nav_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if it's time to report metrics
    ///
    /// Returns true if report_interval_secs has elapsed since last report.
    pub fn should_report(&self) -> bool {
        if let Ok(last) = self.last_report.lock() {
            last.elapsed().as_secs() >= self.report_interval_secs
        } else {
            false
        }
    }

    /// Update last report timestamp
    pub fn mark_reported(&self) {
        if let Ok(mut last) = self.last_report.lock() {
            *last = Instant::now();
        }
    }

    /// Generate a structured metrics report
    ///
    /// Returns all current metrics as a formatted report structure.
    pub fn report(&self) -> NavMetricsReport {
        let hints_generated = self.hints_generated.load(Ordering::Relaxed);
        let focus_switches = self.focus_switches.load(Ordering::Relaxed);
        let nav_errors = self.nav_errors.load(Ordering::Relaxed);

        let hint_gen_stats = self
            .hint_gen_times
            .lock()
            .map(|s| s.compute_stats())
            .unwrap_or_default();
        let input_latency_stats = self
            .nav_input_latency
            .lock()
            .map(|s| s.compute_stats())
            .unwrap_or_default();

        NavMetricsReport {
            hints_generated,
            focus_switches,
            nav_errors,
            hint_generation_ms: hint_gen_stats,
            input_latency_us: input_latency_stats,
        }
    }

    /// Reset all metrics to zero
    ///
    /// Useful for benchmarking or testing scenarios.
    pub fn reset(&self) {
        self.hints_generated.store(0, Ordering::Relaxed);
        self.focus_switches.store(0, Ordering::Relaxed);
        self.nav_errors.store(0, Ordering::Relaxed);
        if let Ok(mut samples) = self.hint_gen_times.lock() {
            samples.clear();
        }
        if let Ok(mut samples) = self.nav_input_latency.lock() {
            samples.clear();
        }
    }
}

// ==================== Timing Samples ====================

/// Bounded circular buffer for timing samples
struct TimingSamples {
    samples: Vec<f64>,
    next_index: usize,
}

impl TimingSamples {
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(MAX_TIMING_SAMPLES),
            next_index: 0,
        }
    }

    fn add_sample(&mut self, value: f64) {
        if self.samples.len() < MAX_TIMING_SAMPLES {
            self.samples.push(value);
        } else {
            self.samples[self.next_index] = value;
            self.next_index = (self.next_index + 1) % MAX_TIMING_SAMPLES;
        }
    }

    fn compute_stats(&self) -> TimingStats {
        if self.samples.is_empty() {
            return TimingStats::default();
        }

        let sum: f64 = self.samples.iter().sum();
        let count = self.samples.len();
        let mean = sum / count as f64;

        let min = self.samples.iter().copied().fold(f64::INFINITY, f64::min);
        let max = self.samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Compute p50, p95, p99
        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = sorted[count / 2];
        let p95 = sorted[(count * 95) / 100];
        let p99 = sorted[(count * 99) / 100];

        TimingStats {
            count,
            mean,
            min,
            max,
            p50,
            p95,
            p99,
        }
    }

    fn clear(&mut self) {
        self.samples.clear();
        self.next_index = 0;
    }
}

// ==================== Report Structures ====================

/// Structured metrics report
#[derive(Debug, Clone)]
pub struct NavMetricsReport {
    /// Total hints generated
    pub hints_generated: u64,

    /// Total focus switches
    pub focus_switches: u64,

    /// Total navigation errors
    pub nav_errors: u64,

    /// Hint generation timing statistics (milliseconds)
    pub hint_generation_ms: TimingStats,

    /// Navigation input latency statistics (microseconds)
    pub input_latency_us: TimingStats,
}

impl NavMetricsReport {
    /// Format as a debug log string
    pub fn format_log(&self) -> String {
        format!(
            "[DEBUG] nav: hints_generated={} focus_switches={} nav_errors={} \
             hint_gen_ms=(mean={:.2} p50={:.2} p95={:.2} p99={:.2}) \
             input_latency_us=(mean={:.2} p50={:.2} p95={:.2} p99={:.2})",
            self.hints_generated,
            self.focus_switches,
            self.nav_errors,
            self.hint_generation_ms.mean,
            self.hint_generation_ms.p50,
            self.hint_generation_ms.p95,
            self.hint_generation_ms.p99,
            self.input_latency_us.mean,
            self.input_latency_us.p50,
            self.input_latency_us.p95,
            self.input_latency_us.p99,
        )
    }
}

/// Timing statistics for a metric
#[derive(Debug, Clone, Default)]
pub struct TimingStats {
    /// Number of samples
    pub count: usize,

    /// Mean value
    pub mean: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,

    /// 50th percentile (median)
    pub p50: f64,

    /// 95th percentile
    pub p95: f64,

    /// 99th percentile
    pub p99: f64,
}

// ==================== Systems ====================

/// System: Periodic metrics reporting
///
/// Runs every frame but only logs when the report interval has elapsed.
/// Uses debug log level to avoid spam in production.
fn report_nav_metrics(metrics: Res<NavMetrics>) {
    if metrics.should_report() {
        let report = metrics.report();
        debug!("{}", report.format_log());
        metrics.mark_reported();
    }
}

// ==================== Plugin ====================

/// Navigation metrics plugin
///
/// Registers the NavMetrics resource and periodic reporting system.
/// Reporting interval can be configured via the resource.
///
/// # Example
/// ```rust,ignore
/// use bevy::prelude::*;
/// use scarab_client::navigation::{NavigationPlugin, NavMetricsPlugin};
///
/// App::new()
///     .add_plugins(NavigationPlugin)
///     .add_plugins(NavMetricsPlugin::with_interval(30)) // Report every 30 seconds
///     .run();
/// ```
pub struct NavMetricsPlugin {
    /// Reporting interval in seconds (0 = disabled)
    report_interval_secs: u64,
}

impl Default for NavMetricsPlugin {
    fn default() -> Self {
        Self {
            report_interval_secs: DEFAULT_REPORT_INTERVAL_SECS,
        }
    }
}

impl NavMetricsPlugin {
    /// Create a plugin with a custom reporting interval
    pub fn with_interval(secs: u64) -> Self {
        Self {
            report_interval_secs: secs,
        }
    }

    /// Create a plugin with reporting disabled
    pub fn disabled() -> Self {
        Self {
            report_interval_secs: 0,
        }
    }
}

impl Plugin for NavMetricsPlugin {
    fn build(&self, app: &mut App) {
        // Register metrics resource
        app.insert_resource(NavMetrics::new(self.report_interval_secs));

        // Add reporting system only if interval > 0
        if self.report_interval_secs > 0 {
            app.add_systems(Update, report_nav_metrics);
            info!(
                "NavMetricsPlugin initialized with {}s reporting interval",
                self.report_interval_secs
            );
        } else {
            info!("NavMetricsPlugin initialized with reporting disabled");
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = NavMetrics::default();
        assert_eq!(metrics.hints_generated.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.focus_switches.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.nav_errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_hint_generation() {
        let metrics = NavMetrics::default();
        metrics.record_hint_generation(5, Some(Duration::from_millis(12)));

        assert_eq!(metrics.hints_generated.load(Ordering::Relaxed), 5);

        metrics.record_hint_generation(3, None);
        assert_eq!(metrics.hints_generated.load(Ordering::Relaxed), 8);
    }

    #[test]
    fn test_record_focus_switch() {
        let metrics = NavMetrics::default();
        metrics.record_focus_switch();
        metrics.record_focus_switch();

        assert_eq!(metrics.focus_switches.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_nav_input_latency() {
        let metrics = NavMetrics::default();
        metrics.record_nav_input_latency(Duration::from_micros(150));

        let report = metrics.report();
        assert_eq!(report.input_latency_us.count, 1);
        assert_eq!(report.input_latency_us.mean, 150.0);
    }

    #[test]
    fn test_record_error() {
        let metrics = NavMetrics::default();
        metrics.record_error();
        metrics.record_error();
        metrics.record_error();

        assert_eq!(metrics.nav_errors.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_metrics_report() {
        let metrics = NavMetrics::default();
        metrics.record_hint_generation(10, Some(Duration::from_millis(15)));
        metrics.record_focus_switch();
        metrics.record_error();

        let report = metrics.report();
        assert_eq!(report.hints_generated, 10);
        assert_eq!(report.focus_switches, 1);
        assert_eq!(report.nav_errors, 1);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = NavMetrics::default();
        metrics.record_hint_generation(5, Some(Duration::from_millis(10)));
        metrics.record_focus_switch();
        metrics.record_error();

        metrics.reset();

        assert_eq!(metrics.hints_generated.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.focus_switches.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.nav_errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_timing_samples_bounded() {
        let mut samples = TimingSamples::new();

        // Add more samples than the max
        for i in 0..150 {
            samples.add_sample(i as f64);
        }

        // Should only keep MAX_TIMING_SAMPLES
        assert_eq!(samples.samples.len(), MAX_TIMING_SAMPLES);
    }

    #[test]
    fn test_timing_stats_computation() {
        let mut samples = TimingSamples::new();
        samples.add_sample(10.0);
        samples.add_sample(20.0);
        samples.add_sample(30.0);
        samples.add_sample(40.0);
        samples.add_sample(50.0);

        let stats = samples.compute_stats();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 30.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 50.0);
    }

    #[test]
    fn test_report_formatting() {
        let metrics = NavMetrics::default();
        metrics.record_hint_generation(42, Some(Duration::from_millis(12)));
        metrics.record_focus_switch();

        let report = metrics.report();
        let log_string = report.format_log();

        assert!(log_string.contains("hints_generated=42"));
        assert!(log_string.contains("focus_switches=1"));
        assert!(log_string.contains("nav_errors=0"));
    }

    #[test]
    fn test_plugin_default_interval() {
        let plugin = NavMetricsPlugin::default();
        assert_eq!(plugin.report_interval_secs, DEFAULT_REPORT_INTERVAL_SECS);
    }

    #[test]
    fn test_plugin_custom_interval() {
        let plugin = NavMetricsPlugin::with_interval(30);
        assert_eq!(plugin.report_interval_secs, 30);
    }

    #[test]
    fn test_plugin_disabled() {
        let plugin = NavMetricsPlugin::disabled();
        assert_eq!(plugin.report_interval_secs, 0);
    }
}
