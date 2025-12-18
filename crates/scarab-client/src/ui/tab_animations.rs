//! Tab transition animations for smooth switching and effects
//!
//! Provides smooth animations when switching between tabs, adding/removing tabs,
//! and hover effects. Uses Bevy 0.15's animation capabilities and custom easing.

use bevy::prelude::*;
use super::animations::easing;

/// Plugin for tab animations
pub struct TabAnimationsPlugin;

impl Plugin for TabAnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_tab_transitions_system,
                update_tab_fade_system,
                update_tab_hover_system,
                cleanup_finished_tab_animations_system,
            ),
        );
    }
}

/// Tab transition animation when switching tabs
#[derive(Component, Clone, Debug)]
pub struct TabTransition {
    /// Index of the tab we're transitioning from
    pub from_index: usize,
    /// Index of the tab we're transitioning to
    pub to_index: usize,
    /// Progress of the transition (0.0 to 1.0)
    pub progress: f32,
    /// Total duration of the transition in seconds
    pub duration: f32,
    /// Easing function to use
    pub easing: TabEasingFunction,
}

impl TabTransition {
    pub fn new(from_index: usize, to_index: usize, duration: f32) -> Self {
        Self {
            from_index,
            to_index,
            progress: 0.0,
            duration,
            easing: TabEasingFunction::EaseOutCubic,
        }
    }

    pub fn with_easing(mut self, easing: TabEasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Get the current interpolated position
    pub fn current_position(&self) -> f32 {
        let eased_progress = self.easing.apply(self.progress);

        // Slide effect: interpolate from source to destination
        let direction = (self.to_index as f32 - self.from_index as f32).signum();
        direction * eased_progress
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
}

/// Easing functions for tab animations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabEasingFunction {
    Linear,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
}

impl TabEasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseInCubic => easing::ease_in_cubic(t),
            Self::EaseOutCubic => easing::ease_out_cubic(t),
            Self::EaseInOutCubic => easing::ease_in_out_cubic(t),
            Self::EaseInQuad => easing::ease_in_quad(t),
            Self::EaseOutQuad => easing::ease_out_quad(t),
            Self::EaseInOutQuad => easing::ease_in_out_quad(t),
            Self::EaseInSine => easing::ease_in_sine(t),
            Self::EaseOutSine => easing::ease_out_sine(t),
            Self::EaseInOutSine => easing::ease_in_out_sine(t),
        }
    }
}

/// Fade animation for tabs being added or removed
#[derive(Component, Clone, Debug)]
pub struct TabFade {
    /// Whether this is a fade in or fade out
    pub fade_in: bool,
    /// Duration of the fade in seconds
    pub duration: f32,
    /// Elapsed time
    pub elapsed: f32,
}

impl TabFade {
    pub fn fade_in(duration: f32) -> Self {
        Self {
            fade_in: true,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn fade_out(duration: f32) -> Self {
        Self {
            fade_in: false,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn alpha(&self) -> f32 {
        let progress = self.progress();
        if self.fade_in {
            easing::ease_out_cubic(progress)
        } else {
            1.0 - easing::ease_in_cubic(progress)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Hover animation for tabs
#[derive(Component, Clone, Debug)]
pub struct TabHover {
    /// Whether the tab is currently hovered
    pub is_hovered: bool,
    /// Animation progress (0.0 = not hovered, 1.0 = fully hovered)
    pub progress: f32,
    /// Animation speed (higher = faster)
    pub speed: f32,
}

impl Default for TabHover {
    fn default() -> Self {
        Self {
            is_hovered: false,
            progress: 0.0,
            speed: 5.0, // Complete transition in 0.2 seconds
        }
    }
}

impl TabHover {
    pub fn scale(&self) -> f32 {
        // Subtle scale effect on hover (1.0 to 1.05)
        1.0 + 0.05 * easing::ease_out_cubic(self.progress)
    }

    pub fn glow_intensity(&self) -> f32 {
        // Glow intensity on hover (0.0 to 1.0)
        easing::ease_out_quad(self.progress)
    }
}

/// Marker component for animation completion
#[derive(Component)]
pub struct TabAnimationComplete;

/// System to update tab transition animations
fn update_tab_transitions_system(
    time: Res<Time>,
    mut query: Query<(&mut TabTransition, &mut Transform), Without<TabAnimationComplete>>,
) {
    for (mut transition, mut transform) in query.iter_mut() {
        transition.progress += time.delta_secs() / transition.duration;
        transition.progress = transition.progress.clamp(0.0, 1.0);

        // Apply slide effect
        let offset = transition.current_position();

        // Horizontal slide animation (assuming tabs are arranged horizontally)
        // Offset is multiplied by a base distance (e.g., 100 pixels)
        let slide_distance = 100.0;
        transform.translation.x = offset * slide_distance;
    }
}

/// System to update tab fade animations
fn update_tab_fade_system(
    time: Res<Time>,
    mut query: Query<(&mut TabFade, &mut BackgroundColor), Without<TabAnimationComplete>>,
) {
    for (mut fade, mut bg_color) in query.iter_mut() {
        fade.elapsed += time.delta_secs();

        let alpha = fade.alpha();

        // Update background color alpha
        let mut color = bg_color.0;
        color.set_alpha(alpha);
        bg_color.0 = color;
    }
}

/// System to update tab hover animations
fn update_tab_hover_system(
    time: Res<Time>,
    mut query: Query<(&mut TabHover, &mut Transform)>,
) {
    for (mut hover, mut transform) in query.iter_mut() {
        let delta = time.delta_secs() * hover.speed;

        if hover.is_hovered {
            // Animate towards hovered state
            hover.progress = (hover.progress + delta).min(1.0);
        } else {
            // Animate away from hovered state
            hover.progress = (hover.progress - delta).max(0.0);
        }

        // Apply scale effect
        let scale = hover.scale();
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
}

/// System to clean up finished animations
fn cleanup_finished_tab_animations_system(
    mut commands: Commands,
    transition_query: Query<(Entity, &TabTransition), Without<TabAnimationComplete>>,
    fade_query: Query<(Entity, &TabFade), Without<TabAnimationComplete>>,
) {
    // Mark completed transitions
    for (entity, transition) in transition_query.iter() {
        if transition.is_complete() {
            commands.entity(entity).insert(TabAnimationComplete);
        }
    }

    // Mark completed fades (and potentially remove fade-out entities)
    for (entity, fade) in fade_query.iter() {
        if fade.is_complete() {
            commands.entity(entity).insert(TabAnimationComplete);

            // If this was a fade-out, despawn the entity
            if !fade.fade_in {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Configuration for tab animations
#[derive(Resource, Clone)]
pub struct TabAnimationConfig {
    /// Duration of tab switch transitions in seconds
    pub switch_duration: f32,
    /// Duration of tab fade in/out in seconds
    pub fade_duration: f32,
    /// Enable tab animations
    pub enabled: bool,
    /// Enable hover effects
    pub hover_enabled: bool,
}

impl Default for TabAnimationConfig {
    fn default() -> Self {
        Self {
            switch_duration: 0.25,
            fade_duration: 0.2,
            enabled: true,
            hover_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_transition_progress() {
        let mut transition = TabTransition::new(0, 1, 1.0);
        assert_eq!(transition.progress, 0.0);
        assert!(!transition.is_complete());

        transition.progress = 0.5;
        assert!(!transition.is_complete());

        transition.progress = 1.0;
        assert!(transition.is_complete());
    }

    #[test]
    fn test_tab_fade_alpha() {
        let mut fade = TabFade::fade_in(1.0);
        assert_eq!(fade.alpha(), 0.0);

        fade.elapsed = 1.0;
        assert_eq!(fade.alpha(), 1.0);
        assert!(fade.is_complete());
    }

    #[test]
    fn test_tab_fade_out_alpha() {
        let mut fade = TabFade::fade_out(1.0);
        assert_eq!(fade.alpha(), 1.0);

        fade.elapsed = 1.0;
        assert_eq!(fade.alpha(), 0.0);
    }

    #[test]
    fn test_tab_hover_scale() {
        let mut hover = TabHover::default();
        assert_eq!(hover.scale(), 1.0);

        hover.is_hovered = true;
        hover.progress = 1.0;
        assert!(hover.scale() > 1.0);
        assert!(hover.scale() <= 1.05);
    }

    #[test]
    fn test_easing_functions() {
        for easing in [
            TabEasingFunction::Linear,
            TabEasingFunction::EaseInCubic,
            TabEasingFunction::EaseOutCubic,
            TabEasingFunction::EaseInOutCubic,
        ] {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);
        }
    }
}
