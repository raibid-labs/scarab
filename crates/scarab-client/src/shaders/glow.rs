//! Border glow post-processing shader implementation
//!
//! Provides dynamic glow effects for:
//! - Active panes
//! - Mode indicators (different colors per mode)
//! - Notifications (pulse effect)

use bevy::prelude::*;
use bevy::render::render_resource::{BindGroupLayout, CachedRenderPipelineId, Shader, ShaderType};
use bevy::render::renderer::RenderDevice;

/// Glow shader settings for focused elements
#[derive(Component, Clone, Copy)]
pub struct GlowSettings {
    /// Glow radius in pixels
    pub radius: f32,
    /// Glow color (RGB)
    pub color: Vec3,
    /// Glow intensity (0.0 = no glow, 1.0 = full)
    pub intensity: f32,
    /// Edge detection threshold
    pub edge_threshold: f32,
    /// Falloff power (higher = sharper edges)
    pub falloff_power: f32,
    /// Enable glow effect
    pub enabled: bool,
    /// Glow type for specialized effects
    pub glow_type: GlowType,
}

impl Default for GlowSettings {
    fn default() -> Self {
        Self {
            radius: 6.0,
            color: Vec3::new(0.545, 0.914, 0.992), // Dracula cyan
            intensity: 0.7,
            edge_threshold: 0.1,
            falloff_power: 2.0,
            enabled: true,
            glow_type: GlowType::Static,
        }
    }
}

/// Type of glow effect
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlowType {
    /// Static glow (constant intensity)
    Static,
    /// Pulsing glow (for notifications)
    Pulse { speed: u32 }, // Speed as u32 for simplicity, represents cycles per second
    /// Breathing glow (slow fade in/out)
    Breathing { speed: u32 },
    /// Mode indicator glow (color changes based on mode)
    Mode,
}

impl Default for GlowType {
    fn default() -> Self {
        Self::Static
    }
}

/// Mode-specific glow colors
#[derive(Resource, Clone)]
pub struct ModeGlowColors {
    pub normal: Vec3,
    pub copy: Vec3,
    pub search: Vec3,
    pub window: Vec3,
    pub font: Vec3,
    pub pick: Vec3,
    pub hint: Vec3,
}

impl Default for ModeGlowColors {
    fn default() -> Self {
        Self {
            normal: Vec3::new(0.659, 0.875, 0.353),  // Slime green
            copy: Vec3::new(0.545, 0.914, 0.992),    // Cyan
            search: Vec3::new(1.0, 0.847, 0.0),      // Gold
            window: Vec3::new(0.741, 0.576, 0.976),  // Purple
            font: Vec3::new(1.0, 0.333, 0.333),      // Red
            pick: Vec3::new(0.298, 0.686, 0.314),    // Green
            hint: Vec3::new(1.0, 0.596, 0.0),        // Orange
        }
    }
}

/// Pulse animation state
#[derive(Component, Clone, Copy)]
pub struct GlowPulse {
    /// Time accumulator for pulse animation
    pub time: f32,
    /// Pulse speed (cycles per second)
    pub speed: f32,
    /// Pulse intensity range (min, max)
    pub intensity_range: (f32, f32),
}

impl Default for GlowPulse {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 2.0, // 2 pulses per second
            intensity_range: (0.3, 1.0),
        }
    }
}

impl GlowPulse {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.intensity_range = (min, max);
        self
    }

    /// Calculate current intensity based on sine wave
    pub fn current_intensity(&self) -> f32 {
        let (min, max) = self.intensity_range;
        let range = max - min;
        let sine = (self.time * self.speed * std::f32::consts::TAU).sin();
        min + range * (sine * 0.5 + 0.5)
    }
}

/// Uniform data passed to glow shader
#[derive(Clone, Copy, ShaderType)]
#[allow(dead_code)]
struct GlowUniforms {
    /// Glow color (RGB + intensity in alpha)
    glow_color: Vec4,
    /// Glow radius in pixels
    glow_radius: f32,
    /// Edge detection threshold
    edge_threshold: f32,
    /// Glow falloff power
    falloff_power: f32,
    /// Padding for alignment
    _padding: f32,
}

/// Glow render node for custom render graph
pub struct GlowShaderNode {
    // Render state will be stored here
}

impl GlowShaderNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GlowShaderNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource to track glow state across frames
#[derive(Resource)]
pub struct GlowRenderState {
    pub pipeline_id: Option<CachedRenderPipelineId>,
    pub bind_group_layout: Option<BindGroupLayout>,
}

impl Default for GlowRenderState {
    fn default() -> Self {
        Self {
            pipeline_id: None,
            bind_group_layout: None,
        }
    }
}

/// System to update glow settings from config
pub fn update_glow_settings(
    config: Res<scarab_config::ScarabConfig>,
    mut glow_query: Query<&mut GlowSettings>,
) {
    for mut glow in glow_query.iter_mut() {
        glow.radius = config.effects.overlay_glow_radius;
        glow.intensity = config.effects.overlay_glow_intensity;
        glow.enabled = config.effects.should_render_glow();

        // Parse color from config (if not mode-based)
        if glow.glow_type != GlowType::Mode {
            let (r, g, b) = config.effects.glow_color_rgb();
            glow.color = Vec3::new(r, g, b);
        }
    }
}

/// System to update pulse animations
pub fn update_glow_pulse(
    time: Res<Time>,
    mut query: Query<(&mut GlowPulse, &mut GlowSettings)>,
) {
    for (mut pulse, mut settings) in query.iter_mut() {
        pulse.time += time.delta_secs();
        settings.intensity = pulse.current_intensity();
    }
}

/// System to update mode-based glow colors
pub fn update_mode_glow(
    mode_colors: Res<ModeGlowColors>,
    // This would need the actual mode state from the terminal
    // For now, just a placeholder
    mut query: Query<&mut GlowSettings>,
) {
    for mut glow in query.iter_mut() {
        if glow.glow_type == GlowType::Mode {
            // Default to normal mode color
            // In actual implementation, this would check the current mode
            glow.color = mode_colors.normal;
        }
    }
}

/// Initialize glow rendering pipeline
pub fn setup_glow_pipeline(
    mut commands: Commands,
    _render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
) {
    // Load glow shader
    let _glow_shader: Handle<Shader> = asset_server.load("shaders/glow.wgsl");

    commands.insert_resource(GlowRenderState {
        pipeline_id: None,
        bind_group_layout: None,
    });

    commands.insert_resource(ModeGlowColors::default());
}

/// Helper to create a static glow
pub fn create_static_glow(color: Vec3, intensity: f32, radius: f32) -> GlowSettings {
    GlowSettings {
        color,
        intensity,
        radius,
        glow_type: GlowType::Static,
        ..Default::default()
    }
}

/// Helper to create a pulsing glow (for notifications)
pub fn create_pulse_glow(color: Vec3, speed: f32, radius: f32) -> (GlowSettings, GlowPulse) {
    let settings = GlowSettings {
        color,
        radius,
        glow_type: GlowType::Pulse { speed: speed as u32 },
        ..Default::default()
    };
    let pulse = GlowPulse::new(speed);
    (settings, pulse)
}

/// Helper to create a mode indicator glow
pub fn create_mode_glow(radius: f32) -> GlowSettings {
    GlowSettings {
        radius,
        glow_type: GlowType::Mode,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glow_settings_default() {
        let settings = GlowSettings::default();
        assert_eq!(settings.radius, 6.0);
        assert_eq!(settings.intensity, 0.7);
        assert_eq!(settings.falloff_power, 2.0);
        assert!(settings.enabled);
    }

    #[test]
    fn test_glow_uniforms_size() {
        // Ensure proper alignment for GPU (should be 16-byte aligned)
        assert_eq!(std::mem::align_of::<GlowUniforms>(), 16);
    }

    #[test]
    fn test_glow_color_from_config() {
        let config = scarab_config::EffectsConfig {
            overlay_glow_color: "#ff00ff".to_string(),
            ..Default::default()
        };

        let (r, g, b) = config.glow_color_rgb();
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 1.0);
    }

    #[test]
    fn test_pulse_intensity() {
        let mut pulse = GlowPulse::default();

        // At time 0, should be at minimum
        let intensity = pulse.current_intensity();
        assert!(intensity >= 0.3 && intensity <= 1.0);

        // Advance time
        pulse.time = std::f32::consts::PI / (2.0 * pulse.speed * std::f32::consts::TAU);
        let intensity = pulse.current_intensity();
        assert!(intensity >= 0.3 && intensity <= 1.0);
    }

    #[test]
    fn test_mode_glow_colors() {
        let colors = ModeGlowColors::default();

        // Check that all colors are valid (between 0 and 1)
        for color in [
            colors.normal,
            colors.copy,
            colors.search,
            colors.window,
            colors.font,
            colors.pick,
            colors.hint,
        ] {
            assert!(color.x >= 0.0 && color.x <= 1.0);
            assert!(color.y >= 0.0 && color.y <= 1.0);
            assert!(color.z >= 0.0 && color.z <= 1.0);
        }
    }

    #[test]
    fn test_create_helpers() {
        let static_glow = create_static_glow(Vec3::ONE, 0.5, 8.0);
        assert_eq!(static_glow.glow_type, GlowType::Static);
        assert_eq!(static_glow.intensity, 0.5);

        let (pulse_glow, pulse) = create_pulse_glow(Vec3::ONE, 3.0, 10.0);
        assert!(matches!(pulse_glow.glow_type, GlowType::Pulse { .. }));
        assert_eq!(pulse.speed, 3.0);

        let mode_glow = create_mode_glow(12.0);
        assert_eq!(mode_glow.glow_type, GlowType::Mode);
        assert_eq!(mode_glow.radius, 12.0);
    }
}
