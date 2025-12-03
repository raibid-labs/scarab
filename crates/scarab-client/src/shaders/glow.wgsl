// Border glow shader for focused overlay elements in Scarab terminal
//
// This shader creates a soft glow effect around focused elements by:
// 1. Detecting edges using alpha channel
// 2. Applying radial distance-based glow
// 3. Combining with original color

struct GlowUniforms {
    // Glow color (RGB + intensity in alpha)
    glow_color: vec4<f32>,
    // Glow radius in pixels
    glow_radius: f32,
    // Edge detection threshold (alpha values below this are considered edges)
    edge_threshold: f32,
    // Glow falloff power (higher = sharper edges)
    falloff_power: f32,
    // Padding for alignment
    _padding: f32,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: GlowUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

// Calculate glow contribution at a given distance from edge
fn glow_intensity(distance: f32, radius: f32, power: f32) -> f32 {
    if (distance >= radius) {
        return 0.0;
    }

    // Smooth falloff using power curve
    let normalized_dist = distance / radius;
    return pow(1.0 - normalized_dist, power);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;

    // Sample original pixel
    let original = textureSample(input_texture, input_sampler, input.uv);

    // Early exit if pixel is fully opaque (not near an edge)
    if (original.a >= 1.0 - uniforms.edge_threshold) {
        return original;
    }

    // Search for nearest edge within glow radius
    let search_radius = i32(uniforms.glow_radius);
    var min_distance = uniforms.glow_radius;
    var found_edge = false;

    // Spiral search pattern for edge detection
    for (var y = -search_radius; y <= search_radius; y++) {
        for (var x = -search_radius; x <= search_radius; x++) {
            let offset = vec2<f32>(f32(x), f32(y));
            let distance = length(offset);

            // Skip if outside search radius
            if (distance > uniforms.glow_radius) {
                continue;
            }

            // Sample neighbor
            let sample_uv = input.uv + offset * texel_size;
            let neighbor = textureSample(input_texture, input_sampler, sample_uv);

            // Check if this is an edge (significant alpha difference)
            if (neighbor.a >= uniforms.edge_threshold) {
                found_edge = true;
                min_distance = min(min_distance, distance);
            }
        }
    }

    // Calculate glow contribution
    var glow = vec4<f32>(0.0);
    if (found_edge) {
        let intensity = glow_intensity(
            min_distance,
            uniforms.glow_radius,
            uniforms.falloff_power
        );

        // Apply glow color with calculated intensity
        glow = uniforms.glow_color * intensity * uniforms.glow_color.a;
    }

    // Composite: original over glow (alpha blending)
    let result_color = original.rgb + glow.rgb * (1.0 - original.a);
    let result_alpha = original.a + glow.a * (1.0 - original.a);

    return vec4<f32>(result_color, result_alpha);
}
