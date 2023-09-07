#include <metal_stdlib>

using namespace metal;

// Vertex layout
struct vertex_t {
    float3 position;
    float3 normal;
    float4 tangent;
    float4 color;
    float2 uv0;
    float2 uv1; 
};

// Constant buffers
struct const_buffer_t {
    float4x4 model_matrix;
    float4x4 view_matrix;
    float4x4 proj_matrix;
};

// Data that's passed from the vertex shader to the fragment shader
struct vertex_shader_output_t {
    float4 position [[position]];
    float4 color;
    float2 uv0;
};

// Vertex shader function
vertex vertex_shader_output_t hello_triangle_vertex(
    const device vertex_t* vertex_array [[buffer(0)]], 
    const constant const_buffer_t* const_buffer [[buffer(1)]],
    uint vertex_index [[vertex_id]]
) {
    vertex_shader_output_t out;
    const device vertex_t& vtx = vertex_array[vertex_index];
    float shading = dot(vtx.normal, normalize(float3(0.3, 0.3, 0.3)));
    shading = clamp(shading, 0.0, 1.0);
    out.color = float4(shading, shading, shading, 1.0);
    out.position = float4(vtx.position.x, vtx.position.y, vtx.position.z, 1.0);
    out.position *= const_buffer->model_matrix;
    out.position *= const_buffer->view_matrix;
    out.position *= const_buffer->proj_matrix;
    out.uv0 = float2(vtx.uv0.x, vtx.uv0.y);
    return out;
}

// Fragment shader function
fragment float4 hello_triangle_fragment(
    vertex_shader_output_t in [[stage_in]],
    texture2d<float> tex_color [[texture(0)]]
) {
    constexpr sampler texture_sampler (mag_filter::nearest, min_filter::nearest);
    return in.color * tex_color.sample(texture_sampler, in.uv0);
}