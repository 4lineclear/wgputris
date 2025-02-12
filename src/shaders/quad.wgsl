struct Uniforms {
  bounds: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
  @location(0) colour: vec4<f32>, // r,g,b,a
  @location(1) pos: vec2<u32>,  // x, y
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) colour: vec4<f32>, // r,g,b,a
};

@vertex
fn vs_main(rect: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  let ndc = ((vec2<f32>(rect.pos) / vec2<f32>(uniforms.bounds)) * 2.0) - vec2<f32>(1.0, 1.0);

  out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
  out.colour = rect.colour;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return in.colour; // Use per-rectangle colour
}
