struct Uniforms {
  width: f32,
  height: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
  @location(0) color: vec4<f32>, // r,g,b,a
  @location(1) rect: vec4<f32>,  // x, y, width, height
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>, // r,g,b,a
};

@vertex
fn vs_main(@builtin(instance_index) instance_idx: u32, rect: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  let x = rect.rect.x;
  let y = rect.rect.y;
  let w = rect.rect.z;
  let h = rect.rect.w;

  // Select the correct corner based on vertex index (0-3)
  let pos = array<vec2<f32>, 4>(
    vec2<f32>(x, y),          // Top-left
    vec2<f32>(x + w, y),      // Top-right
    vec2<f32>(x, y + h),      // Bottom-left
    vec2<f32>(x + w, y + h)   // Bottom-right
  );

  let ndc_x = (pos[instance_idx].x / uniforms.width) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pos[instance_idx].y / uniforms.height) * 2.0;

  out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
  out.color = rect.color; // Optional, for texturing
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return in.color; // Use per-rectangle color
}
