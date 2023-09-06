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

// Data that's passed from the vertex shader to the fragment shader
struct vertex_shader_output_t {
    float4 position [[position]];
    float4 color;
};

// Vertex shader function
vertex vertex_shader_output_t hello_triangle_vertex(
    const device vertex_t* vertex_array [[buffer(0)]], 
    uint vertex_index [[vertex_id]]) {
    vertex_shader_output_t out;
    const device vertex_t& vtx = vertex_array[vertex_index];
    out.color = float4(vtx.normal.r, vtx.normal.g, vtx.normal.b, 1.0);
    out.position = float4(vtx.position.x, vtx.position.y, 0.0, 1.0);
    return out;
}

// Fragment shader function
fragment float4 hello_triangle_fragment(vertex_shader_output_t in [[stage_in]]) {
    return in.color;
}