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
  // TODO: turn the position into something real
  var out: VertexOutput;
  out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  out.color = rect.color; // Optional, for texturing
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return in.color; // Use per-rectangle color
}
