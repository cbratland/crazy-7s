#import bevy_pbr::forward_io::VertexOutput

@group(1) @binding(0)
var image_texture: texture_2d<f32>;
@group(1) @binding(1)
var image_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var tiled_uv: vec2<f32>;
    tiled_uv = in.uv;
    var tiled_uv_x: f32;
    var tiled_uv_y: f32;
    tiled_uv_x = fract(in.uv.x * 3.0);
    tiled_uv_y = fract(in.uv.y * 3.0);
    tiled_uv = vec2(tiled_uv_x,tiled_uv_y);
    return textureSample(image_texture, image_sampler, tiled_uv);
}
