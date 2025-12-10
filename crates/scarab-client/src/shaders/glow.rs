//! Border glow post-processing shader implementation

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
        }
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

        // Parse color from config
        let (r, g, b) = config.effects.glow_color_rgb();
        glow.color = Vec3::new(r, g, b);
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
}
