//! Gaussian blur post-processing shader implementation

use bevy::prelude::*;
use bevy::render::render_resource::{BindGroupLayout, CachedRenderPipelineId, Shader, ShaderType};
use bevy::render::renderer::RenderDevice;

/// Blur shader settings and state
#[derive(Component, Clone, Copy)]
pub struct BlurSettings {
    /// Blur radius in pixels (kernel size = 2*radius + 1)
    pub radius: f32,
    /// Intensity multiplier (0.0 = no blur, 1.0 = full blur)
    pub intensity: f32,
    /// Enable blur effect
    pub enabled: bool,
}

impl Default for BlurSettings {
    fn default() -> Self {
        Self {
            radius: 4.0,
            intensity: 0.8,
            enabled: true,
        }
    }
}

/// Uniform data passed to blur shader
#[derive(Clone, Copy, ShaderType)]
struct BlurUniforms {
    /// Direction: (1, 0) for horizontal, (0, 1) for vertical
    direction: Vec2,
    /// Blur radius in pixels
    radius: f32,
    /// Intensity multiplier
    intensity: f32,
}

/// Blur render node for custom render graph
pub struct BlurShaderNode {
    // Render state will be stored here
}

impl BlurShaderNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BlurShaderNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource to track blur state across frames
#[derive(Resource)]
pub struct BlurRenderState {
    pub pipeline_id: Option<CachedRenderPipelineId>,
    pub bind_group_layout: Option<BindGroupLayout>,
}

impl Default for BlurRenderState {
    fn default() -> Self {
        Self {
            pipeline_id: None,
            bind_group_layout: None,
        }
    }
}

/// System to update blur settings from config
pub fn update_blur_settings(
    config: Res<scarab_config::ScarabConfig>,
    mut blur_query: Query<&mut BlurSettings>,
) {
    for mut blur in blur_query.iter_mut() {
        blur.radius = config.effects.overlay_blur_radius;
        blur.intensity = config.effects.overlay_blur_intensity;
        blur.enabled = config.effects.should_render_blur();
    }
}

/// Initialize blur rendering pipeline
pub fn setup_blur_pipeline(
    mut commands: Commands,
    _render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
) {
    // Load blur shader
    let _blur_shader: Handle<Shader> = asset_server.load("shaders/blur.wgsl");

    commands.insert_resource(BlurRenderState {
        pipeline_id: None,
        bind_group_layout: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_settings_default() {
        let settings = BlurSettings::default();
        assert_eq!(settings.radius, 4.0);
        assert_eq!(settings.intensity, 0.8);
        assert!(settings.enabled);
    }

    #[test]
    fn test_blur_uniforms_size() {
        // BlurUniforms contains Vec2 (2x f32) + 2 f32s
        // Vec2 in glam has 4-byte alignment (f32), not 8-byte
        // The ShaderType derive handles proper GPU padding during serialization
        assert_eq!(std::mem::align_of::<BlurUniforms>(), 4);
        // Total size: Vec2 (8 bytes) + f32 (4 bytes) + f32 (4 bytes) = 16 bytes
        assert_eq!(std::mem::size_of::<BlurUniforms>(), 16);
    }
}
