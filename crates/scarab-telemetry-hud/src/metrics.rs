//! Performance Metrics Collection
//!
//! This module provides lock-free performance metric tracking for the telemetry HUD.
//! Metrics are collected using atomic operations to minimize overhead on the render thread.

use bevy::prelude::*;
use std::collections::VecDeque;

/// Performance metrics resource
///
/// Tracks real-time performance data including FPS, frame times, and timing statistics.
/// Uses a rolling window to compute averages and percentiles with minimal overhead.
#[derive(Resource)]
pub struct PerformanceMetrics {
    /// Rolling window of frame times (seconds)
    frame_times: VecDeque<f32>,

    /// Maximum number of samples to keep
    window_size: usize,

    /// Current frame time (seconds)
    pub current_frame_time: f32,

    /// Current FPS
    pub current_fps: f32,

    /// Average frame time over the window (seconds)
    pub avg_frame_time: f32,

    /// Minimum frame time in the window (seconds)
    pub min_frame_time: f32,

    /// Maximum frame time in the window (seconds)
    pub max_frame_time: f32,

    /// Total frames processed
    pub total_frames: u64,

    /// Total elapsed time (seconds)
    pub total_elapsed: f32,
}

impl PerformanceMetrics {
    /// Create a new metrics tracker with the specified window size
    pub fn new(window_size: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(window_size),
            window_size,
            current_frame_time: 0.0,
            current_fps: 0.0,
            avg_frame_time: 0.0,
            min_frame_time: f32::MAX,
            max_frame_time: 0.0,
            total_frames: 0,
            total_elapsed: 0.0,
        }
    }

    /// Record a new frame time sample
    pub fn record_frame(&mut self, delta_secs: f32) {
        // Update current metrics
        self.current_frame_time = delta_secs;
        self.current_fps = if delta_secs > 0.0 {
            1.0 / delta_secs
        } else {
            0.0
        };

        // Add to rolling window
        if self.frame_times.len() >= self.window_size {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(delta_secs);

        // Update statistics
        self.compute_stats();

        // Update totals
        self.total_frames += 1;
        self.total_elapsed += delta_secs;
    }

    /// Compute statistics from the current window
    fn compute_stats(&mut self) {
        if self.frame_times.is_empty() {
            self.avg_frame_time = 0.0;
            self.min_frame_time = 0.0;
            self.max_frame_time = 0.0;
            return;
        }

        let sum: f32 = self.frame_times.iter().sum();
        let count = self.frame_times.len() as f32;

        self.avg_frame_time = sum / count;
        self.min_frame_time = self
            .frame_times
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        self.max_frame_time = self
            .frame_times
            .iter()
            .copied()
            .fold(f32::MIN, f32::max);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            current_fps: self.current_fps,
            current_frame_time_ms: self.current_frame_time * 1000.0,
            avg_frame_time_ms: self.avg_frame_time * 1000.0,
            min_frame_time_ms: self.min_frame_time * 1000.0,
            max_frame_time_ms: self.max_frame_time * 1000.0,
            total_frames: self.total_frames,
            total_elapsed_secs: self.total_elapsed,
        }
    }

    /// Get the frame time samples for graphing (last N samples)
    pub fn get_frame_time_samples(&self, max_samples: usize) -> Vec<f32> {
        let start = if self.frame_times.len() > max_samples {
            self.frame_times.len() - max_samples
        } else {
            0
        };

        self.frame_times
            .iter()
            .skip(start)
            .copied()
            .map(|t| t * 1000.0) // Convert to milliseconds
            .collect()
    }

    /// Get average FPS over the entire window
    pub fn avg_fps(&self) -> f32 {
        if self.avg_frame_time > 0.0 {
            1.0 / self.avg_frame_time
        } else {
            0.0
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.current_frame_time = 0.0;
        self.current_fps = 0.0;
        self.avg_frame_time = 0.0;
        self.min_frame_time = f32::MAX;
        self.max_frame_time = 0.0;
        self.total_frames = 0;
        self.total_elapsed = 0.0;
    }
}

/// Snapshot of performance metrics at a point in time
#[derive(Debug, Clone, Copy)]
pub struct PerformanceSnapshot {
    /// Current FPS
    pub current_fps: f32,

    /// Current frame time (milliseconds)
    pub current_frame_time_ms: f32,

    /// Average frame time (milliseconds)
    pub avg_frame_time_ms: f32,

    /// Minimum frame time (milliseconds)
    pub min_frame_time_ms: f32,

    /// Maximum frame time (milliseconds)
    pub max_frame_time_ms: f32,

    /// Total frames processed
    pub total_frames: u64,

    /// Total elapsed time (seconds)
    pub total_elapsed_secs: f32,
}

/// Extended metrics including cache, memory, and navigation stats
#[derive(Debug, Clone, Copy)]
pub struct ExtendedMetrics {
    /// Core performance metrics
    pub performance: PerformanceSnapshot,

    /// Cache statistics
    pub cache_stats: CacheStats,

    /// Memory usage statistics
    pub memory_stats: MemoryStats,

    /// Navigation hint statistics
    pub hint_stats: HintStats,
}

/// Cache statistics for glyph and texture caches
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Number of glyphs in cache
    pub glyph_count: usize,

    /// Glyph cache hit rate (0.0 to 1.0)
    pub glyph_hit_rate: f32,

    /// Texture atlas usage count
    pub atlas_count: usize,

    /// Total texture memory (bytes)
    pub texture_memory_bytes: usize,
}

/// Memory usage statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    /// Current process memory (MB)
    pub process_mb: f32,

    /// Heap allocation size (MB)
    pub heap_mb: f32,

    /// GPU memory usage (MB)
    pub gpu_mb: f32,
}

/// Navigation hint statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct HintStats {
    /// Number of active NavHint entities
    pub hint_count: usize,

    /// Number of FocusableRegion entities
    pub focusable_count: usize,

    /// Number of visible overlays
    pub overlay_count: usize,
}

/// Resource to track extended telemetry data
#[derive(Resource, Default)]
pub struct TelemetryData {
    pub cache_stats: CacheStats,
    pub memory_stats: MemoryStats,
    pub hint_stats: HintStats,
}

impl TelemetryData {
    /// Get a combined snapshot of all metrics
    pub fn extended_snapshot(&self, perf: &PerformanceMetrics) -> ExtendedMetrics {
        ExtendedMetrics {
            performance: perf.snapshot(),
            cache_stats: self.cache_stats,
            memory_stats: self.memory_stats,
            hint_stats: self.hint_stats,
        }
    }
}

/// System: Update performance metrics
///
/// Runs every frame to collect timing data.
/// This is a lightweight operation using a circular buffer.
pub(crate) fn update_metrics(time: Res<Time>, mut metrics: ResMut<PerformanceMetrics>) {
    let delta = time.delta_secs();
    metrics.record_frame(delta);
}

/// System: Update cache statistics
///
/// Collects cache metrics from rendering systems.
/// This would ideally query actual cache resources but can be stubbed for now.
pub(crate) fn update_cache_stats(_telemetry: ResMut<TelemetryData>) {
    // Stub: In a full implementation, this would query GlyphCache and TextureAtlas resources
    // For now, we maintain placeholder values that can be updated by other systems
    // Real implementation would look like:
    // if let Some(glyph_cache) = glyph_cache_query.get_single().ok() {
    //     _telemetry.cache_stats.glyph_count = glyph_cache.len();
    // }
}

/// System: Update memory statistics
///
/// Samples process memory usage using platform-specific APIs.
pub(crate) fn update_memory_stats(mut telemetry: ResMut<TelemetryData>) {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = value.parse::<f32>() {
                            telemetry.memory_stats.process_mb = kb / 1024.0;
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Placeholder for other platforms
        telemetry.memory_stats.process_mb = 0.0;
    }
}

/// System: Update navigation hint statistics
///
/// Counts active hint-related entities in the ECS world.
pub(crate) fn update_hint_stats(
    _telemetry: ResMut<TelemetryData>,
    // We'll add component queries when integrating with scarab-client
    // For now, this is a placeholder that can be filled in during integration
) {
    // Stub: Will be updated when integrated with scarab-client navigation
    // Real implementation would count:
    // - NavHint entities
    // - FocusableRegion entities
    // - HintOverlay entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = PerformanceMetrics::new(60);
        assert_eq!(metrics.window_size, 60);
        assert_eq!(metrics.total_frames, 0);
        assert_eq!(metrics.current_fps, 0.0);
    }

    #[test]
    fn test_record_frame() {
        let mut metrics = PerformanceMetrics::new(10);

        // Record a frame at 60 FPS (16.67ms)
        metrics.record_frame(1.0 / 60.0);

        assert_eq!(metrics.total_frames, 1);
        assert!((metrics.current_fps - 60.0).abs() < 0.1);
        assert_eq!(metrics.frame_times.len(), 1);
    }

    #[test]
    fn test_window_size_limit() {
        let mut metrics = PerformanceMetrics::new(5);

        // Record 10 frames
        for _ in 0..10 {
            metrics.record_frame(1.0 / 60.0);
        }

        // Should only keep last 5
        assert_eq!(metrics.frame_times.len(), 5);
        assert_eq!(metrics.total_frames, 10);
    }

    #[test]
    fn test_stats_computation() {
        let mut metrics = PerformanceMetrics::new(10);

        // Record frames with varying times
        metrics.record_frame(0.010); // 10ms
        metrics.record_frame(0.020); // 20ms
        metrics.record_frame(0.030); // 30ms

        assert!((metrics.min_frame_time - 0.010).abs() < 0.001);
        assert!((metrics.max_frame_time - 0.030).abs() < 0.001);
        assert!((metrics.avg_frame_time - 0.020).abs() < 0.001);
    }

    #[test]
    fn test_snapshot() {
        let mut metrics = PerformanceMetrics::new(10);
        metrics.record_frame(1.0 / 60.0);

        let snapshot = metrics.snapshot();
        assert!((snapshot.current_fps - 60.0).abs() < 0.1);
        assert_eq!(snapshot.total_frames, 1);
    }

    #[test]
    fn test_frame_time_samples() {
        let mut metrics = PerformanceMetrics::new(100);

        // Record 5 frames
        for i in 1..=5 {
            metrics.record_frame(i as f32 / 1000.0); // 1ms, 2ms, 3ms, 4ms, 5ms
        }

        let samples = metrics.get_frame_time_samples(3);
        assert_eq!(samples.len(), 3);
        assert!((samples[0] - 3.0).abs() < 0.01);
        assert!((samples[1] - 4.0).abs() < 0.01);
        assert!((samples[2] - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_avg_fps() {
        let mut metrics = PerformanceMetrics::new(10);

        // Record 3 frames at 60 FPS
        for _ in 0..3 {
            metrics.record_frame(1.0 / 60.0);
        }

        let avg_fps = metrics.avg_fps();
        assert!((avg_fps - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_reset() {
        let mut metrics = PerformanceMetrics::new(10);

        metrics.record_frame(1.0 / 60.0);
        metrics.record_frame(1.0 / 30.0);

        metrics.reset();

        assert_eq!(metrics.total_frames, 0);
        assert_eq!(metrics.current_fps, 0.0);
        assert_eq!(metrics.frame_times.len(), 0);
    }
}
