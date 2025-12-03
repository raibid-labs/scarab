// Two-pass Gaussian blur shader for Scarab terminal overlay effects
//
// This shader implements a separable Gaussian blur using two passes:
// 1. Horizontal pass: blur along X axis
// 2. Vertical pass: blur along Y axis
//
// Separable approach is more efficient: O(2*N) vs O(N^2) for naive implementation

struct BlurUniforms {
    // Direction: (1, 0) for horizontal, (0, 1) for vertical
    direction: vec2<f32>,
    // Blur radius in pixels (kernel size = 2*radius + 1)
    radius: f32,
    // Intensity multiplier (0.0 = no blur, 1.0 = full blur)
    intensity: f32,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: BlurUniforms;

// Gaussian weights for 5-tap kernel (radius=2)
// Pre-computed for sigma=1.0: exp(-(x^2)/(2*sigma^2))
const GAUSSIAN_WEIGHTS_5: array<f32, 3> = array<f32, 3>(
    0.38774,  // center
    0.24477,  // +/- 1
    0.06136,  // +/- 2
);

// Gaussian weights for 9-tap kernel (radius=4)
// Pre-computed for sigma=2.0
const GAUSSIAN_WEIGHTS_9: array<f32, 5> = array<f32, 5>(
    0.22702,  // center
    0.19459,  // +/- 1
    0.12162,  // +/- 2
    0.05591,  // +/- 3
    0.01894,  // +/- 4
);

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

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(input_texture));
    let texel_size = 1.0 / texture_size;

    // Use 5-tap kernel for radius <= 2, 9-tap for larger
    let use_large_kernel = uniforms.radius > 2.0;
    let kernel_radius = select(2, 4, use_large_kernel);

    var color = vec4<f32>(0.0);
    var total_weight = 0.0;

    // Sample along the blur direction
    for (var i = -kernel_radius; i <= kernel_radius; i++) {
        let offset = vec2<f32>(f32(i)) * uniforms.direction * texel_size;
        let sample_uv = input.uv + offset;

        // Determine weight based on kernel size
        var weight: f32;
        if (use_large_kernel) {
            weight = GAUSSIAN_WEIGHTS_9[abs(i)];
        } else {
            if (abs(i) < 3) {
                weight = GAUSSIAN_WEIGHTS_5[abs(i)];
            } else {
                weight = 0.0;
            }
        }

        // Sample texture and accumulate
        let sample = textureSample(input_texture, input_sampler, sample_uv);
        color += sample * weight;
        total_weight += weight;
    }

    // Normalize and blend based on intensity
    color /= total_weight;

    // Blend between original and blurred based on intensity
    let original = textureSample(input_texture, input_sampler, input.uv);
    color = mix(original, color, uniforms.intensity);

    return color;
}
