//! Rate limiting for plugin navigation actions
//!
//! Prevents plugins from overwhelming the navigation system with too many
//! actions in a short time period. Uses a sliding window approach to track
//! action counts.

use std::time::Instant;
use thiserror::Error;

/// Error returned when a plugin exceeds its rate limit
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RateLimitError {
    /// Plugin exceeded its maximum actions per second
    #[error("Rate limit exceeded: {current} actions in last second (max: {limit})")]
    ExceededLimit { current: u32, limit: u32 },
}

/// Rate limiter for plugin navigation actions
///
/// Uses a simple sliding window approach: tracks the number of actions
/// within the current time window. When the window expires, the counter
/// resets.
///
/// # Examples
///
/// ```
/// use scarab_client::plugin_host::PluginNavRateLimiter;
///
/// let mut limiter = PluginNavRateLimiter::new(10); // 10 actions per second
///
/// // First 10 actions should succeed
/// for _ in 0..10 {
///     assert!(limiter.check_action().is_ok());
/// }
///
/// // 11th action should fail
/// assert!(limiter.check_action().is_err());
/// ```
#[derive(Debug, Clone)]
pub struct PluginNavRateLimiter {
    /// Maximum actions allowed per second
    actions_per_second: u32,
    /// Start of the current time window
    window_start: Instant,
    /// Number of actions in the current window
    action_count: u32,
}

impl PluginNavRateLimiter {
    /// Create a new rate limiter with the specified actions per second limit
    ///
    /// # Arguments
    ///
    /// * `actions_per_second` - Maximum number of actions allowed per second
    ///
    /// # Examples
    ///
    /// ```
    /// use scarab_client::plugin_host::PluginNavRateLimiter;
    ///
    /// let limiter = PluginNavRateLimiter::new(10);
    /// ```
    pub fn new(actions_per_second: u32) -> Self {
        Self {
            actions_per_second,
            window_start: Instant::now(),
            action_count: 0,
        }
    }

    /// Create a rate limiter with default settings (10 actions per second)
    ///
    /// # Examples
    ///
    /// ```
    /// use scarab_client::plugin_host::PluginNavRateLimiter;
    ///
    /// let limiter = PluginNavRateLimiter::default();
    /// ```
    pub fn default() -> Self {
        Self::new(10)
    }

    /// Check if an action is allowed and increment the counter
    ///
    /// Returns `Ok(())` if the action is allowed, or `Err(RateLimitError)`
    /// if the rate limit would be exceeded.
    ///
    /// # Examples
    ///
    /// ```
    /// use scarab_client::plugin_host::PluginNavRateLimiter;
    ///
    /// let mut limiter = PluginNavRateLimiter::new(5);
    ///
    /// // First 5 actions succeed
    /// for _ in 0..5 {
    ///     assert!(limiter.check_action().is_ok());
    /// }
    ///
    /// // 6th action fails
    /// assert!(limiter.check_action().is_err());
    /// ```
    pub fn check_action(&mut self) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start);

        // If we've passed the 1-second window, reset the counter
        if elapsed.as_secs() >= 1 {
            self.window_start = now;
            self.action_count = 0;
        }

        // Check if we're under the limit
        if self.action_count >= self.actions_per_second {
            return Err(RateLimitError::ExceededLimit {
                current: self.action_count,
                limit: self.actions_per_second,
            });
        }

        // Increment the counter
        self.action_count += 1;
        Ok(())
    }

    /// Get the current action count in the window
    ///
    /// Useful for monitoring and debugging.
    pub fn current_count(&self) -> u32 {
        self.action_count
    }

    /// Get the maximum actions per second limit
    pub fn limit(&self) -> u32 {
        self.actions_per_second
    }

    /// Reset the rate limiter
    ///
    /// Clears the action count and starts a new time window.
    pub fn reset(&mut self) {
        self.window_start = Instant::now();
        self.action_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = PluginNavRateLimiter::new(10);
        assert_eq!(limiter.limit(), 10);
        assert_eq!(limiter.current_count(), 0);
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = PluginNavRateLimiter::default();
        assert_eq!(limiter.limit(), 10);
    }

    #[test]
    fn test_rate_limiter_within_limit() {
        let mut limiter = PluginNavRateLimiter::new(5);

        // First 5 actions should succeed
        for i in 0..5 {
            assert!(
                limiter.check_action().is_ok(),
                "Action {} should succeed",
                i
            );
        }

        assert_eq!(limiter.current_count(), 5);
    }

    #[test]
    fn test_rate_limiter_exceeds_limit() {
        let mut limiter = PluginNavRateLimiter::new(3);

        // First 3 actions succeed
        for _ in 0..3 {
            assert!(limiter.check_action().is_ok());
        }

        // 4th action should fail
        let result = limiter.check_action();
        assert!(result.is_err());

        match result.unwrap_err() {
            RateLimitError::ExceededLimit { current, limit } => {
                assert_eq!(current, 3);
                assert_eq!(limit, 3);
            }
        }
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let mut limiter = PluginNavRateLimiter::new(2);

        // Use up the limit
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_err());

        // Wait for window to expire
        thread::sleep(Duration::from_millis(1100));

        // Should be able to act again
        assert!(limiter.check_action().is_ok());
        assert_eq!(limiter.current_count(), 1);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = PluginNavRateLimiter::new(2);

        // Use up the limit
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_err());

        // Manual reset
        limiter.reset();

        // Should be able to act again
        assert!(limiter.check_action().is_ok());
        assert_eq!(limiter.current_count(), 1);
    }

    #[test]
    fn test_rate_limiter_multiple_windows() {
        let mut limiter = PluginNavRateLimiter::new(2);

        // First window
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_err());

        // Wait and try again
        thread::sleep(Duration::from_millis(1100));

        // Second window
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_ok());
        assert!(limiter.check_action().is_err());
    }

    #[test]
    fn test_rate_limit_error_display() {
        let err = RateLimitError::ExceededLimit {
            current: 10,
            limit: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("10"));
        assert!(msg.contains("5"));
        assert!(msg.contains("Rate limit exceeded"));
    }
}
